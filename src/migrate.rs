//! Migration logic: detect foreign schemas, build conversion plans, apply transformations.
//!
//! Architecture: two-pass approach.
//! Pass 1: scan all files, detect schema, assign IDs, build slug→id mapping.
//! Pass 2: rewrite each file with new frontmatter + remapped references.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

// --- Schema Detection ---

/// Detected format of a ticket corpus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectedFormat {
    Tkt,
    Tk,
    Unknown,
}

impl DetectedFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            DetectedFormat::Tkt => "tkt",
            DetectedFormat::Tk => "tk",
            DetectedFormat::Unknown => "unknown",
        }
    }
}

/// Result of schema detection with confidence.
#[derive(Debug)]
pub struct Detection {
    pub format: DetectedFormat,
    /// 0.0–1.0 confidence score
    pub confidence: f64,
    /// Signals that contributed to the detection
    pub signals: Vec<String>,
}

/// Detect the format of tickets in a directory.
pub fn detect(dir: &Path) -> Detection {
    let files = ticket_files(dir);
    if files.is_empty() {
        return Detection {
            format: DetectedFormat::Unknown,
            confidence: 0.0,
            signals: vec!["no .md files found".into()],
        };
    }

    let mut tkt_score: f64 = 0.0;
    let mut tk_score: f64 = 0.0;
    let mut signals = Vec::new();

    // Check filename patterns
    let numeric_prefix_count = files
        .iter()
        .filter(|f| {
            f.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
        })
        .count();

    let slug_only_count = files.len() - numeric_prefix_count;

    if numeric_prefix_count > slug_only_count {
        tkt_score += 0.3;
        signals.push(format!(
            "{}/{} files have numeric prefix (tkt pattern)",
            numeric_prefix_count,
            files.len()
        ));
    } else if slug_only_count > numeric_prefix_count {
        tk_score += 0.3;
        signals.push(format!(
            "{}/{} files are slug-only (tk pattern)",
            slug_only_count,
            files.len()
        ));
    }

    // Sample files for field detection
    let sample_size = files.len().min(10);
    let mut has_blocked_by = 0;
    let mut has_deps = 0;
    let mut has_id_field = 0;
    let mut has_title_field = 0;
    let mut has_status_closed = 0;
    let mut has_status_done = 0;

    for path in files.iter().take(sample_size) {
        if let Ok(content) = std::fs::read_to_string(path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("blocked_by:") {
                    has_blocked_by += 1;
                    break;
                }
                if trimmed.starts_with("deps:") {
                    has_deps += 1;
                    break;
                }
                if trimmed.starts_with("id:") {
                    has_id_field += 1;
                }
                if trimmed.starts_with("title:") {
                    has_title_field += 1;
                }
                if trimmed == "status: closed" {
                    has_status_closed += 1;
                }
                if trimmed == "status: done" {
                    has_status_done += 1;
                }
                // Stop at end of frontmatter
                if trimmed == "---" && !line.starts_with("---") {
                    break;
                }
            }
        }
    }

    if has_blocked_by > 0 {
        tkt_score += 0.3;
        signals.push(format!("{} files use blocked_by (tkt)", has_blocked_by));
    }
    if has_deps > 0 {
        tk_score += 0.3;
        signals.push(format!("{} files use deps (tk)", has_deps));
    }
    if has_id_field > 0 {
        tkt_score += 0.2;
        signals.push(format!("{} files have id: field (tkt)", has_id_field));
    }
    if has_title_field > 0 {
        tkt_score += 0.1;
    }
    if has_status_closed > 0 {
        tk_score += 0.2;
        signals.push(format!(
            "{} files use status:closed (tk)",
            has_status_closed
        ));
    }
    if has_status_done > 0 {
        tkt_score += 0.1;
    }

    let (format, confidence) = if tk_score > tkt_score && tk_score >= 0.3 {
        (DetectedFormat::Tk, tk_score.min(1.0))
    } else if tkt_score > tk_score && tkt_score >= 0.3 {
        (DetectedFormat::Tkt, tkt_score.min(1.0))
    } else {
        (DetectedFormat::Unknown, 0.0)
    };

    Detection {
        format,
        confidence,
        signals,
    }
}

// --- Migration Plan ---

/// A planned migration: maps old slugs to new IDs, tracks field transformations.
#[derive(Debug)]
pub struct MigrationPlan {
    /// Old slug → new numeric ID
    pub id_map: HashMap<String, String>,
    /// Files to process, in order
    pub entries: Vec<PlanEntry>,
    /// Total files
    pub total: usize,
}

/// One file's planned transformation.
#[derive(Debug)]
pub struct PlanEntry {
    source_snapshot: String,
    pub source_path: PathBuf,
    pub target_path: PathBuf,
    pub old_slug: String,
    pub new_id: String,
    pub extracted_title: String,
    pub status_mapped: String,
    pub priority_mapped: Option<String>,
    pub deps_remapped: Vec<String>,
}

/// Build a migration plan for tk → tkt conversion.
pub fn plan_tk(dir: &Path) -> MigrationPlan {
    let files = ticket_files(dir);

    // Sort by filename for stable ordering
    let mut files = files;
    files.sort_by(|a, b| {
        a.file_name()
            .unwrap_or_default()
            .cmp(b.file_name().unwrap_or_default())
    });

    // Pass 1: assign IDs and build mapping
    let mut id_map: HashMap<String, String> = HashMap::new();
    let mut entries: Vec<PlanEntry> = Vec::new();

    for (i, path) in files.iter().enumerate() {
        let new_id = format!("{:02}", i + 1);
        let slug = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        id_map.insert(slug.clone(), new_id.clone());
    }

    // Pass 2: parse each file and plan the transformation
    for path in &files {
        let slug = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let new_id = id_map.get(&slug).cloned().unwrap_or_default();

        let content = std::fs::read_to_string(path).unwrap_or_default();
        let parsed = parse_tk_file(&content);

        // Map deps slugs to new IDs
        let deps_remapped: Vec<String> = parsed
            .deps
            .iter()
            .filter_map(|dep| id_map.get(dep.as_str()).cloned())
            .collect();

        // Map status
        let status_mapped = match parsed.status.as_str() {
            "closed" => "done".to_string(),
            "wip" | "in-progress" | "in_progress" => "in_progress".to_string(),
            "backlog" => "backlog".to_string(),
            _ => "open".to_string(),
        };

        // Map priority
        let priority_mapped = parsed.priority.as_ref().and_then(|p| match p.as_str() {
            "1" => Some("urgent".to_string()),
            "2" => Some("high".to_string()),
            "3" => Some("medium".to_string()),
            "4" => Some("low".to_string()),
            other if ["urgent", "high", "medium", "low"].contains(&other) => {
                Some(other.to_string())
            }
            _ => None,
        });

        let target_filename = format!("{}-{}.md", new_id, slug);
        let target_path = dir.join(&target_filename);

        entries.push(PlanEntry {
            source_snapshot: content,
            source_path: path.clone(),
            target_path,
            old_slug: slug,
            new_id,
            extracted_title: parsed.title,
            status_mapped,
            priority_mapped,
            deps_remapped,
        });
    }

    let total = entries.len();
    MigrationPlan {
        id_map,
        entries,
        total,
    }
}

// --- Prepare and apply ---

pub struct ApplyResult {
    pub files_written: usize,
    pub files_renamed: usize,
    pub orphaned_deps: Vec<(String, String)>,
}

pub struct PreparedMigration {
    backup_dir: PathBuf,
    files: Vec<PreparedTicket>,
    orphaned_deps: Vec<(String, String)>,
}

struct PreparedTicket {
    source: PathBuf,
    destination: PathBuf,
    original: String,
    converted: String,
}

pub fn prepare(dir: &Path, plan: &MigrationPlan) -> anyhow::Result<PreparedMigration> {
    let root = dir.canonicalize()?;
    let mut files = Vec::new();
    let mut orphaned_deps = Vec::new();
    for entry in &plan.entries {
        if entry.source_path.parent().unwrap_or(dir).canonicalize()? != root
            || entry.target_path.parent().unwrap_or(dir).canonicalize()? != root
        {
            return Err(crate::DomainError::new(
                crate::ErrorKind::Validation,
                "migration paths must remain in the selected ticket directory".into(),
            )
            .into());
        }
        let source = root.join(entry.source_path.file_name().unwrap_or_default());
        let destination = root.join(entry.target_path.file_name().unwrap_or_default());
        let content = regular_text(&source)?.ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("missing migration source {}", source.display()),
            )
        })?;
        if content != entry.source_snapshot {
            return Err(crate::DomainError::new(
                crate::ErrorKind::Conflict,
                format!(
                    "migration source changed after planning: {}",
                    source.display()
                ),
            )
            .into());
        }
        let parsed = parse_tk_file(&content);
        for dep in &parsed.deps {
            if !plan.id_map.contains_key(dep.as_str()) {
                orphaned_deps.push((entry.old_slug.clone(), dep.clone()));
            }
        }
        // Build new frontmatter
        let mut fm_lines = Vec::new();
        fm_lines.push(format!("id: \"{}\"", entry.new_id));
        fm_lines.push(format!(
            "title: \"{}\"",
            entry.extracted_title.replace('"', "\\\"")
        ));
        fm_lines.push(format!("status: {}", entry.status_mapped));

        if entry.deps_remapped.is_empty() {
            fm_lines.push("blocked_by: []".to_string());
        } else {
            let deps_str: Vec<String> = entry
                .deps_remapped
                .iter()
                .map(|d| format!("\"{}\"", d))
                .collect();
            fm_lines.push(format!("blocked_by: [{}]", deps_str.join(", ")));
        }

        if let Some(ref prio) = entry.priority_mapped {
            fm_lines.push(format!("priority: {}", prio));
        }

        // Build body (strip old frontmatter, keep content)
        let body = &parsed.body;

        let new_content = format!("---\n{}\n---\n\n{}\n", fm_lines.join("\n"), body.trim());

        files.push(PreparedTicket {
            source,
            destination,
            original: content,
            converted: new_content,
        });
    }
    let prepared = PreparedMigration {
        backup_dir: dir.parent().unwrap_or(dir).join(".tickets.bak"),
        files,
        orphaned_deps,
    };
    prepared.verify_sources_and_destinations()?;
    Ok(prepared)
}

fn regular_text(path: &Path) -> anyhow::Result<Option<String>> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) => {
            let linked = metadata.file_type().is_symlink();
            #[cfg(windows)]
            let linked = {
                use std::os::windows::fs::MetadataExt;
                linked || metadata.file_attributes() & 0x400 != 0
            };
            if linked || !metadata.is_file() {
                return Err(crate::DomainError::new(
                    crate::ErrorKind::Conflict,
                    format!(
                        "migration needs a regular, unlinked file: {}",
                        path.display()
                    ),
                )
                .into());
            }
            Ok(Some(std::fs::read_to_string(path)?))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

impl PreparedMigration {
    pub fn orphaned_deps(&self) -> &[(String, String)] {
        &self.orphaned_deps
    }

    fn verify_source(file: &PreparedTicket) -> anyhow::Result<()> {
        if regular_text(&file.source)?.as_deref() != Some(file.original.as_str()) {
            return Err(crate::DomainError::new(
                crate::ErrorKind::Conflict,
                format!(
                    "migration source changed; preserved it and any published copy at {}",
                    file.destination.display()
                ),
            )
            .into());
        }
        Ok(())
    }

    fn verify_sources_and_destinations(&self) -> anyhow::Result<()> {
        if let Ok(metadata) = std::fs::symlink_metadata(&self.backup_dir) {
            let linked = metadata.file_type().is_symlink();
            #[cfg(windows)]
            let linked = {
                use std::os::windows::fs::MetadataExt;
                linked || metadata.file_attributes() & 0x400 != 0
            };
            if linked || !metadata.is_dir() {
                return Err(crate::DomainError::new(
                    crate::ErrorKind::Conflict,
                    "migration backup root must be a regular directory".into(),
                )
                .into());
            }
        }
        for file in &self.files {
            Self::verify_source(file)?;
            if file.source != file.destination
                && std::fs::symlink_metadata(&file.destination).is_ok()
            {
                return Err(crate::DomainError::new(
                    crate::ErrorKind::Conflict,
                    format!(
                        "migration destination exists; preserved it: {}",
                        file.destination.display()
                    ),
                )
                .into());
            }
            let backup = self
                .backup_dir
                .join(file.source.file_name().unwrap_or_default());
            if regular_text(&backup)?.is_some_and(|text| text != file.original) {
                return Err(crate::DomainError::new(
                    crate::ErrorKind::Conflict,
                    format!(
                        "different backup already exists; preserved it: {}",
                        backup.display()
                    ),
                )
                .into());
            }
        }
        Ok(())
    }

    pub fn apply(&self) -> anyhow::Result<ApplyResult> {
        self.verify_sources_and_destinations()?;
        std::fs::create_dir_all(&self.backup_dir)?;
        for file in &self.files {
            let backup = self
                .backup_dir
                .join(file.source.file_name().unwrap_or_default());
            match regular_text(&backup)? {
                Some(existing) if existing == file.original => {}
                Some(_) => {
                    return Err(crate::DomainError::new(
                        crate::ErrorKind::Conflict,
                        format!(
                            "different backup already exists; preserved it: {}",
                            backup.display()
                        ),
                    )
                    .into())
                }
                None => crate::core::atomic_create(&backup, &file.original, false)?,
            }
        }
        let mut files_written = 0;
        let mut files_renamed = 0;
        for file in &self.files {
            Self::verify_source(file)?;
            if file.source == file.destination {
                crate::core::atomic_write(&file.destination, &file.converted)?;
            } else {
                crate::core::atomic_create(&file.destination, &file.converted, false)?;
                Self::verify_source(file)?;
                std::fs::remove_file(&file.source)?;
                files_renamed += 1;
            }
            files_written += 1;
        }
        Ok(ApplyResult {
            files_written,
            files_renamed,
            orphaned_deps: self.orphaned_deps.clone(),
        })
    }
}

// --- Parsing helpers ---

/// Parsed fields from a tk-format ticket.
struct TkParsed {
    title: String,
    status: String,
    priority: Option<String>,
    deps: Vec<String>,
    body: String,
}

/// Parse a tk-format ticket file.
fn parse_tk_file(content: &str) -> TkParsed {
    let mut status = String::new();
    let mut priority = None;
    let mut deps: Vec<String> = Vec::new();
    let mut in_frontmatter = false;
    let mut frontmatter_ended = false;
    let mut body_lines: Vec<&str> = Vec::new();
    let mut title = String::new();

    for line in content.lines() {
        if !in_frontmatter && !frontmatter_ended && line.trim() == "---" {
            in_frontmatter = true;
            continue;
        }
        if in_frontmatter && line.trim() == "---" {
            in_frontmatter = false;
            frontmatter_ended = true;
            continue;
        }
        if in_frontmatter {
            let trimmed = line.trim();
            if let Some(val) = trimmed.strip_prefix("status:") {
                status = val.trim().to_string();
            } else if let Some(val) = trimmed.strip_prefix("priority:") {
                priority = Some(val.trim().to_string());
            } else if let Some(val) = trimmed.strip_prefix("deps:") {
                // Parse deps: [slug1, slug2] or deps: slug1, slug2
                let val = val.trim().trim_start_matches('[').trim_end_matches(']');
                for dep in val.split(',') {
                    let d = dep.trim().trim_matches('"').trim_matches('\'').trim();
                    if !d.is_empty() {
                        deps.push(d.to_string());
                    }
                }
            }
            // Skip other frontmatter fields (they'll be dropped)
            continue;
        }
        if frontmatter_ended {
            // Extract title from first H1
            if title.is_empty() && line.starts_with("# ") {
                title = line[2..].trim().to_string();
            }
            body_lines.push(line);
        }
    }

    // Fallback: if no frontmatter, treat whole file as body
    if !frontmatter_ended {
        body_lines = content.lines().collect();
        for line in &body_lines {
            if title.is_empty() && line.starts_with("# ") {
                title = line[2..].trim().to_string();
                break;
            }
        }
    }

    // Fallback title from filename would be set by caller
    if title.is_empty() {
        title = "Untitled".to_string();
    }

    TkParsed {
        title,
        status,
        priority,
        deps,
        body: body_lines.join("\n"),
    }
}

/// List .md files in the tickets directory (non-recursive).
fn ticket_files(dir: &Path) -> Vec<PathBuf> {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|ext| ext == "md"))
                .filter(|p| {
                    // Skip config.toml and other non-ticket files
                    p.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .ne("config.toml")
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_tk_file_basic() {
        let content = "---\nstatus: closed\npriority: 2\ndeps: [auth-system, api-layer]\n---\n\n# Deploy Pipeline\n\nDeploy to staging.\n";
        let parsed = parse_tk_file(content);
        assert_eq!(parsed.title, "Deploy Pipeline");
        assert_eq!(parsed.status, "closed");
        assert_eq!(parsed.priority, Some("2".to_string()));
        assert_eq!(parsed.deps, vec!["auth-system", "api-layer"]);
        assert!(parsed.body.contains("Deploy to staging."));
    }

    #[test]
    fn parse_tk_file_no_deps() {
        let content = "---\nstatus: open\n---\n\n# Simple Task\n\nDo the thing.\n";
        let parsed = parse_tk_file(content);
        assert_eq!(parsed.title, "Simple Task");
        assert_eq!(parsed.status, "open");
        assert!(parsed.deps.is_empty());
    }

    #[test]
    fn parse_tk_file_no_frontmatter() {
        let content = "# Just a heading\n\nSome body text.\n";
        let parsed = parse_tk_file(content);
        assert_eq!(parsed.title, "Just a heading");
        assert!(parsed.status.is_empty());
    }

    #[test]
    fn detect_tk_format() {
        let dir = tempfile::tempdir().unwrap();
        // Create tk-style files
        std::fs::write(
            dir.path().join("auth-system.md"),
            "---\nstatus: closed\ndeps: []\n---\n\n# Auth System\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("api-layer.md"),
            "---\nstatus: open\ndeps: [auth-system]\n---\n\n# API Layer\n",
        )
        .unwrap();

        let result = detect(dir.path());
        assert_eq!(result.format, DetectedFormat::Tk);
        assert!(result.confidence >= 0.3);
    }

    #[test]
    fn detect_tkt_format() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("01-auth.md"),
            "---\nid: \"01\"\ntitle: \"Auth\"\nstatus: done\nblocked_by: []\n---\n\n# Auth\n",
        )
        .unwrap();

        let result = detect(dir.path());
        assert_eq!(result.format, DetectedFormat::Tkt);
    }

    #[test]
    fn plan_tk_basic() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("auth-system.md"),
            "---\nstatus: closed\npriority: 1\ndeps: []\n---\n\n# Auth System\n\nImplement auth.\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("deploy-pipeline.md"),
            "---\nstatus: open\npriority: 2\ndeps: [auth-system]\n---\n\n# Deploy Pipeline\n\nDeploy.\n",
        )
        .unwrap();

        let plan = plan_tk(dir.path());
        assert_eq!(plan.total, 2);
        assert_eq!(plan.id_map.get("auth-system"), Some(&"01".to_string()));
        assert_eq!(plan.id_map.get("deploy-pipeline"), Some(&"02".to_string()));

        let auth = &plan.entries[0];
        assert_eq!(auth.new_id, "01");
        assert_eq!(auth.extracted_title, "Auth System");
        assert_eq!(auth.status_mapped, "done");
        assert_eq!(auth.priority_mapped, Some("urgent".to_string()));

        let deploy = &plan.entries[1];
        assert_eq!(deploy.new_id, "02");
        assert_eq!(deploy.deps_remapped, vec!["01"]);
        assert_eq!(deploy.priority_mapped, Some("high".to_string()));
    }

    #[test]
    fn apply_creates_correct_files() {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path().join(".tickets");
        std::fs::create_dir(&dir).unwrap();
        std::fs::write(
            dir.as_path().join("auth-system.md"),
            "---\nstatus: closed\ndeps: []\n---\n\n# Auth System\n\nImplement auth.\n",
        )
        .unwrap();
        std::fs::write(
            dir.as_path().join("deploy.md"),
            "---\nstatus: open\ndeps: [auth-system]\n---\n\n# Deploy\n\nDeploy it.\n",
        )
        .unwrap();

        let plan = plan_tk(dir.as_path());
        let result = prepare(dir.as_path(), &plan).unwrap().apply().unwrap();

        assert_eq!(result.files_written, 2);
        assert!(dir.as_path().join("01-auth-system.md").exists());
        assert!(dir.as_path().join("02-deploy.md").exists());
        assert!(!dir.as_path().join("auth-system.md").exists());
        assert!(!dir.as_path().join("deploy.md").exists());

        // Check content
        let auth_content =
            std::fs::read_to_string(dir.as_path().join("01-auth-system.md")).unwrap();
        assert!(auth_content.contains("id: \"01\""));
        assert!(auth_content.contains("title: \"Auth System\""));
        assert!(auth_content.contains("status: done"));
        assert!(auth_content.contains("blocked_by: []"));

        let deploy_content = std::fs::read_to_string(dir.as_path().join("02-deploy.md")).unwrap();
        assert!(deploy_content.contains("id: \"02\""));
        assert!(deploy_content.contains("blocked_by: [\"01\"]"));

        // Backup exists
        assert!(dir
            .as_path()
            .parent()
            .unwrap()
            .join(".tickets.bak")
            .join("auth-system.md")
            .exists());
    }
    #[test]
    fn migration_preserves_source_and_unmanaged_destination_on_collision() {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path().join(".tickets");
        std::fs::create_dir(&dir).unwrap();
        let source = dir.join("auth.md");
        let original = "---\nstatus: open\ndeps: []\n---\n\n# Auth\n\nOriginal specification.\n";
        std::fs::write(&source, original).unwrap();
        let plan = plan_tk(&dir);
        let prepared = prepare(&dir, &plan).unwrap();
        let destination = &plan.entries[0].target_path;
        std::fs::write(destination, "foreign arrival").unwrap();
        assert!(
            prepared.apply().is_err(),
            "must refuse a destination created after planning"
        );
        assert_eq!(std::fs::read_to_string(&source).unwrap(), original);
        assert_eq!(
            std::fs::read_to_string(destination).unwrap(),
            "foreign arrival"
        );
    }

    #[test]
    #[cfg(windows)]
    fn failed_migration_publication_keeps_the_original_source() {
        use std::os::windows::fs::OpenOptionsExt;
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path().join(".tickets");
        std::fs::create_dir(&dir).unwrap();
        let source = dir.join("auth.md");
        let original = "---\nstatus: open\ndeps: []\n---\n\n# Auth\n\nOriginal specification.\n";
        std::fs::write(&source, original).unwrap();
        let plan = plan_tk(&dir);
        let prepared = prepare(&dir, &plan).unwrap();
        let destination = &plan.entries[0].target_path;
        std::fs::write(destination, "locked foreign arrival").unwrap();
        let held = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(destination)
            .unwrap();
        assert!(prepared.apply().is_err());
        assert!(
            source.exists(),
            "failed migration removed the original before publishing output"
        );
        assert_eq!(std::fs::read_to_string(&source).unwrap(), original);
        assert_eq!(
            std::fs::read_to_string(destination).unwrap(),
            "locked foreign arrival"
        );
        drop(held);
    }
    #[test]
    fn preparation_rejects_unreadable_text_without_creating_backups() {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path().join(".tickets");
        std::fs::create_dir(&dir).unwrap();
        let source = dir.join("invalid.md");
        std::fs::write(&source, [255, 254]).unwrap();
        let plan = plan_tk(&dir);
        assert!(prepare(&dir, &plan).is_err());
        assert_eq!(std::fs::read(&source).unwrap(), [255, 254]);
        assert!(!temp.path().join(".tickets.bak").exists());
    }

    #[test]
    fn migration_preserves_conflicting_backup_and_changed_source() {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path().join(".tickets");
        std::fs::create_dir(&dir).unwrap();
        let source = dir.join("auth.md");
        let original = "---\nstatus: open\ndeps: []\n---\n\n# Auth\n\nOriginal.\n";
        std::fs::write(&source, original).unwrap();
        let prepared = prepare(&dir, &plan_tk(&dir)).unwrap();
        std::fs::write(&source, "concurrent human edit").unwrap();
        assert!(prepared.apply().is_err());
        assert_eq!(
            std::fs::read_to_string(&source).unwrap(),
            "concurrent human edit"
        );
        assert!(!temp.path().join(".tickets.bak").exists());
        std::fs::write(&source, original).unwrap();
        let backup = temp.path().join(".tickets.bak/auth.md");
        std::fs::create_dir_all(backup.parent().unwrap()).unwrap();
        std::fs::write(&backup, "older preserved backup").unwrap();
        assert!(prepared.apply().is_err());
        assert_eq!(
            std::fs::read_to_string(&backup).unwrap(),
            "older preserved backup"
        );
        assert_eq!(std::fs::read_to_string(&source).unwrap(), original);
        assert!(!dir.join("01-auth.md").exists());
    }
}

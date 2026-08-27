use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use anyhow::{Context, Result, bail};
use regex::Regex;

// --- Constants ---

pub const STATUS_VALUES: &[&str] = &["open", "in_progress", "done", "backlog"];
pub const ENV_VALUES: &[&str] = &["corp", "personal", "either"];

// --- Compiled regex patterns ---

static RE_FM_KEY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([A-Za-z_][A-Za-z0-9_-]*)\s*:(.*)$").unwrap());
static RE_NUMERIC_PREFIX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d+)(.*)$").unwrap());
static RE_FILENAME_ID: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d+)-").unwrap());

// --- Enums ---

/// Ticket lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Backlog,
    Open,
    InProgress,
    Done,
}

impl Status {
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "backlog" => Ok(Status::Backlog),
            "open" => Ok(Status::Open),
            "in_progress" => Ok(Status::InProgress),
            "done" => Ok(Status::Done),
            other => bail!(
                "invalid status {:?} (expected backlog/open/in_progress/done)",
                other
            ),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Status::Backlog => "backlog",
            Status::Open => "open",
            Status::InProgress => "in_progress",
            Status::Done => "done",
        }
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Environment filter for tickets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Env {
    Corp,
    Personal,
    Either,
}

impl Env {
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "corp" => Ok(Env::Corp),
            "personal" => Ok(Env::Personal),
            "either" => Ok(Env::Either),
            other => bail!("invalid env {:?} (expected corp/personal/either)", other),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Env::Corp => "corp",
            Env::Personal => "personal",
            Env::Either => "either",
        }
    }
}

impl std::fmt::Display for Env {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Ticket priority level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    Urgent,
    High,
    Medium,
    Low,
}

impl Priority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Priority::Urgent => "urgent",
            Priority::High => "high",
            Priority::Medium => "medium",
            Priority::Low => "low",
        }
    }

    /// Numeric sort key: lower = higher priority.
    pub fn sort_key(&self) -> u8 {
        match self {
            Priority::Urgent => 0,
            Priority::High => 1,
            Priority::Medium => 2,
            Priority::Low => 3,
        }
    }

    /// Parse a priority string. Returns None for unknown values (lenient).
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().trim_matches('"') {
            "urgent" => Some(Priority::Urgent),
            "high" => Some(Priority::High),
            "medium" => Some(Priority::Medium),
            "low" => Some(Priority::Low),
            _ => None,
        }
    }
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// --- AC types ---

/// Specifies which acceptance criteria boxes to check.
pub enum AcSelection<'a> {
    /// Check all unchecked boxes.
    All,
    /// Check specific 1-based indices.
    Indices(&'a [u32]),
}

/// Stats about acceptance criteria state after a check operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AcStats {
    pub checked: usize,
    pub unchecked: usize,
    pub total: usize,
}

// --- Compiled regex for AC manipulation ---

static RE_UNCHECKED_AC: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"- \[ \]").unwrap());
static RE_CHECKED_AC: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"- \[x\]").unwrap());

// --- TicketFile ---

/// Raw frontmatter editor. Preserves field ordering and unknown fields for surgical edits.
#[derive(Debug, Clone)]
pub struct TicketFile {
    pub path: PathBuf,
    /// Ordered frontmatter entries: (key, raw_value) — key="" for blank lines.
    pub fm: Vec<(String, String)>,
    /// Everything after the closing --- fence.
    pub body: String,
}

impl TicketFile {
    /// Parse a ticket file from disk.
    pub fn parse(path: &Path) -> Result<Self> {
        let content =
            std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        Self::parse_str(&content, path)
    }

    /// Parse from a string (for testing).
    pub fn parse_str(content: &str, path: &Path) -> Result<Self> {
        // Strip a leading UTF-8 BOM (YAML 1.2 §5.2: a conforming parser strips it).
        // Some Windows editors add it, which would otherwise hide the opening fence.
        let content = content.strip_prefix('\u{FEFF}').unwrap_or(content);
        let lines: Vec<&str> = content.split('\n').collect();

        if lines.is_empty() || !is_fence(lines[0]) {
            bail!("{}: no opening frontmatter fence on line 1", path.display());
        }

        let key_re = &*RE_FM_KEY;
        let mut fm: Vec<(String, String)> = Vec::new();
        let mut close_idx = None;

        for (i, line) in lines.iter().enumerate().skip(1) {
            if is_fence(line) {
                close_idx = Some(i);
                break;
            }
            if let Some(caps) = key_re.captures(line) {
                fm.push((caps[1].to_string(), caps[2].to_string()));
            } else if (line.starts_with(' ') || line.starts_with('\t')) && !fm.is_empty() {
                let last = fm.last_mut().unwrap();
                last.1.push('\n');
                last.1.push_str(line);
            } else if line.trim_start().starts_with('#') {
                // Comment line (YAML 1.2 §6.6): tolerate so the ticket isn't ejected.
                // Stored as an empty-key passthrough (like blank lines) — preserved by
                // serialize, dropped by lint's canonical rewrite (comments are non-data).
                fm.push((String::new(), (*line).to_string()));
            } else if line.trim().is_empty() {
                fm.push((String::new(), (*line).to_string()));
            } else {
                bail!(
                    "{}: unparseable frontmatter line {}: {:?}",
                    path.display(),
                    i + 1,
                    line
                );
            }
        }

        let close_idx = close_idx
            .ok_or_else(|| anyhow::anyhow!("{}: no closing frontmatter fence", path.display()))?;

        for req in &["id", "title", "status", "blocked_by"] {
            if !fm.iter().any(|(k, _)| k == req) {
                bail!("{}: missing required field: {}", path.display(), req);
            }
        }

        let body = lines[close_idx + 1..].join("\n");

        Ok(TicketFile {
            path: path.to_owned(),
            fm,
            body,
        })
    }

    /// Get a field's trimmed value.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.fm
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.trim())
    }

    /// Set a field value (raw text). Replaces if exists, appends if not.
    pub fn set_field(&mut self, key: &str, value: &str) {
        for (k, v) in self.fm.iter_mut() {
            if k == key {
                *v = format!(" {}", value);
                return;
            }
        }
        self.fm.push((key.to_string(), format!(" {}", value)));
    }

    /// Remove a field entirely (for clearing optional fields).
    pub fn remove_field(&mut self, key: &str) {
        self.fm.retain(|(k, _)| k != key);
    }

    // --- Typed mutation methods ---

    /// Set the ticket status (typed — no raw strings in callers).
    pub fn set_status(&mut self, status: Status) {
        self.set_field("status", status.as_str());
    }

    /// Set the blocked_by array (handles YAML formatting internally).
    pub fn set_blocked_by(&mut self, ids: &[impl AsRef<str>]) {
        let formatted = ids
            .iter()
            .map(|d| format!("\"{}\"", yaml_scalar_escape(d.as_ref())))
            .collect::<Vec<_>>()
            .join(", ");
        self.set_field("blocked_by", &format!("[{}]", formatted));
    }

    /// Set or clear the priority field.
    pub fn set_priority(&mut self, priority: Option<Priority>) {
        match priority {
            Some(p) => self.set_field("priority", p.as_str()),
            None => self.remove_field("priority"),
        }
    }

    /// Set or clear the env field.
    pub fn set_env(&mut self, env: Option<Env>) {
        match env {
            Some(e) => self.set_field("env", e.as_str()),
            None => self.remove_field("env"),
        }
    }

    /// Set or clear the validation_criteria field (multi-line YAML list format).
    pub fn set_validation_criteria(&mut self, criteria: &[impl AsRef<str>]) {
        if criteria.is_empty() {
            self.remove_field("validation_criteria");
            return;
        }
        let lines = criteria
            .iter()
            .map(|c| format!("\n  - \"{}\"", yaml_scalar_escape(c.as_ref())))
            .collect::<String>();
        self.set_field("validation_criteria", &lines);
    }

    /// Append a resolution section to the body (idempotent — skips if already present).
    pub fn append_resolution(&mut self, date: &str, note: &str, spike_branch: Option<&str>) {
        if self.body.contains("## Resolution") {
            return;
        }
        let branch_note = spike_branch
            .map(|b| format!("\n\nSpike branch: {}", b))
            .unwrap_or_default();
        self.body = format!(
            "{}\n\n## Resolution ({})\n\n{}{}\n",
            self.body.trim_end(),
            date,
            note,
            branch_note
        );
    }

    /// Check acceptance criteria boxes. Returns stats about the final state.
    pub fn check_acs(&mut self, selection: AcSelection) -> AcStats {
        let range = match crate::core::ac_section_range(&self.body) {
            Some(r) => r,
            None => {
                return AcStats {
                    checked: 0,
                    unchecked: 0,
                    total: 0,
                };
            }
        };

        match selection {
            AcSelection::All => {
                let section = self.body[range.clone()].replace("- [ ]", "- [x]");
                self.body.replace_range(range.clone(), &section);
            }
            AcSelection::Indices(indices) => {
                let section = &self.body[range.clone()];
                let offsets: Vec<(usize, usize)> = RE_UNCHECKED_AC
                    .find_iter(section)
                    .map(|m| (range.start + m.start(), range.start + m.end()))
                    .collect();
                // Apply in reverse order to preserve offsets
                for &idx in indices.iter().rev() {
                    let i = (idx as usize).saturating_sub(1);
                    if i < offsets.len() {
                        let (abs_start, abs_end) = offsets[i];
                        self.body.replace_range(abs_start..abs_end, "- [x]");
                    }
                }
            }
        }

        // Compute final stats
        self.ac_stats()
    }

    /// Get current acceptance criteria stats without modifying anything.
    pub fn ac_stats(&self) -> AcStats {
        let range = match crate::core::ac_section_range(&self.body) {
            Some(r) => r,
            None => {
                return AcStats {
                    checked: 0,
                    unchecked: 0,
                    total: 0,
                };
            }
        };
        let section = &self.body[range];
        let unchecked = RE_UNCHECKED_AC.find_iter(section).count();
        let checked = RE_CHECKED_AC.find_iter(section).count();
        AcStats {
            checked,
            unchecked,
            total: checked + unchecked,
        }
    }

    /// Serialize preserving raw frontmatter (surgical writes).
    pub fn serialize(&self) -> String {
        let mut parts = vec!["---".to_string()];
        for (k, v) in &self.fm {
            if k.is_empty() {
                parts.push(v.clone());
            } else {
                parts.push(format!("{}:{}", k, v));
            }
        }
        parts.push("---".to_string());
        let header = parts.join("\n");
        format!("{}\n{}", header, self.body)
    }

    /// Write the ticket file back to disk.
    pub fn write(&self) -> Result<()> {
        std::fs::write(&self.path, self.serialize())
            .with_context(|| format!("writing {}", self.path.display()))
    }
}

// --- Ticket ---

/// A parsed, validated ticket with typed fields. Constructed from a TicketFile.
/// Field access is zero-cost (&str borrows on owned Strings).
#[derive(Debug, Clone)]
pub struct Ticket {
    pub id: String,
    pub title: String,
    pub status: Status,
    pub blocked_by: Vec<String>,
    pub env: Env,
    pub priority: Option<Priority>,
    pub spec: Option<String>,
    pub validation_criteria: Vec<String>,
    pub tags: Vec<String>,
    /// Machine capability requirements. Ticket only appears on frontier
    /// if the machine's capabilities are a superset of this list.
    pub requires: Vec<String>,
    pub path: PathBuf,
    pub body: String,
    /// The underlying raw file for mutations.
    pub file: TicketFile,
}

impl Ticket {
    /// Parse and validate a ticket from disk.
    pub fn parse(path: &Path) -> Result<Self> {
        let file = TicketFile::parse(path)?;
        Self::from_file(file)
    }

    /// Parse and validate from a string (for testing).
    #[allow(dead_code)]
    pub fn parse_str(content: &str, path: &Path) -> Result<Self> {
        let file = TicketFile::parse_str(content, path)?;
        Self::from_file(file)
    }

    /// Construct a validated Ticket from a TicketFile.
    fn from_file(file: TicketFile) -> Result<Self> {
        let path_display = file.path.display().to_string();

        let raw_id = file.get("id").unwrap_or("");
        let id = yaml_scalar_unescape(raw_id.trim_matches('"').trim_matches('\''));

        let raw_title = file.get("title").unwrap_or("");
        let title = yaml_scalar_unescape(raw_title.trim_matches('"').trim_matches('\''));

        let raw_status = file.get("status").unwrap_or("");
        let status =
            Status::parse(raw_status).with_context(|| format!("{}: bad status", path_display))?;

        let blocked_by = parse_blocked_by(file.get("blocked_by").unwrap_or(""));

        let raw_env = file
            .get("env")
            .map(|v| v.trim_matches('"'))
            .unwrap_or("either");
        let env = Env::parse(raw_env).with_context(|| format!("{}: bad env", path_display))?;

        let priority = match file.get("priority").map(|v| v.trim_matches('"')) {
            Some(s) => Priority::parse(s),
            None => None,
        };

        let spec = file
            .get("spec")
            .map(|v| yaml_scalar_unescape(v.trim_matches('"').trim_matches('\'')));

        let validation_criteria =
            parse_validation_criteria(file.get("validation_criteria").unwrap_or(""));

        let tags = parse_tags(file.get("tags").unwrap_or(""));

        // Parse requires field (machine capability requirements)
        // Backward compat: if env is set and requires is empty, synthesize from env
        let mut requires = parse_tags(file.get("requires").unwrap_or(""));
        if requires.is_empty() && env != Env::Either {
            requires.push(env.as_str().to_string());
        }

        Ok(Ticket {
            id,
            title,
            status,
            blocked_by,
            env,
            priority,
            spec,
            validation_criteria,
            tags,
            requires,
            path: file.path.clone(),
            body: file.body.clone(),
            file,
        })
    }

    /// Numeric sort key: (numeric prefix, remainder).
    pub fn numeric_key(&self) -> (u64, String) {
        let re = &*RE_NUMERIC_PREFIX;
        match re.captures(&self.id) {
            Some(caps) => (caps[1].parse().unwrap_or(u64::MAX), caps[2].to_string()),
            None => (u64::MAX, self.id.clone()),
        }
    }

    /// Whether this ticket has high or urgent priority.
    pub fn is_high_priority(&self) -> bool {
        matches!(self.priority, Some(Priority::Urgent) | Some(Priority::High))
    }

    /// Sort key for priority: urgent(0) > high(1) > default/medium(2) > low(3).
    pub fn priority_sort_key(&self) -> u8 {
        self.priority.map(|p| p.sort_key()).unwrap_or(2) // None = medium/default
    }
}

// --- Corpus ---

/// Load all tickets from a .tickets/ directory.
pub fn load_corpus(dir: &Path) -> Result<Vec<Ticket>> {
    let mut tickets = Vec::new();
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .with_context(|| format!("reading {}", dir.display()))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        match Ticket::parse(&entry.path()) {
            Ok(ticket) => tickets.push(ticket),
            Err(e) => {
                eprintln!("⚠ skipping {}: {}", entry.file_name().to_string_lossy(), e);
            }
        }
    }
    Ok(tickets)
}

/// Compute the frontier: open tickets with all deps done, env-filtered, priority-sorted.
pub fn frontier(corpus: &[Ticket]) -> Vec<&Ticket> {
    frontier_with_default_env(corpus, "")
}

/// Compute the frontier with a fallback default_env (from project config).
/// CREW_ENV env var takes priority over the default.
pub fn frontier_with_default_env<'a>(corpus: &'a [Ticket], default_env: &str) -> Vec<&'a Ticket> {
    // Build machine capabilities: CREW_ENV (legacy) + config machine.capabilities
    let crew_env = std::env::var("CREW_ENV").unwrap_or_default();
    let effective_env = if crew_env.is_empty() {
        default_env.to_string()
    } else {
        crew_env
    };

    // Machine capabilities come from config (loaded by caller and passed via default_env for legacy).
    // For requires matching, we build a simple set from CREW_ENV as a capability.
    // The full machine.capabilities set is checked in ready.rs which has access to ProjectConfig.
    // Here we handle the legacy path: CREW_ENV as a single capability.
    let legacy_caps: Vec<&str> = if effective_env.is_empty() {
        vec![]
    } else {
        vec![effective_env.as_str()]
    };

    let done: std::collections::HashSet<&str> = corpus
        .iter()
        .filter(|t| t.status == Status::Done)
        .map(|t| t.id.as_str())
        .collect();

    let mut pool: Vec<&Ticket> = corpus
        .iter()
        .filter(|t| {
            if t.status != Status::Open {
                return false;
            }
            if !t.blocked_by.iter().all(|dep| done.contains(dep.as_str())) {
                return false;
            }
            // Legacy env filter: only applies to tickets using the old env field.
            // Tickets with explicit `requires` (env == Either) are filtered by
            // machine.capabilities in ready.rs, not here.
            if t.env != Env::Either
                && !legacy_caps.is_empty()
                && !t.requires.iter().all(|r| legacy_caps.contains(&r.as_str()))
            {
                return false;
            }
            // If machine declares no capabilities (legacy_caps empty) and ticket has requires,
            // we still show it — the full capability check happens in ready.rs with ProjectConfig
            true
        })
        .collect();

    pool.sort_by_key(|t| (t.priority_sort_key(), t.numeric_key()));
    pool
}

/// Find the maximum numeric id in a list of filenames.
pub fn max_id(names: &[String]) -> u64 {
    names
        .iter()
        .filter_map(|n| {
            RE_FILENAME_ID
                .captures(n)
                .map(|c| c[1].parse::<u64>().unwrap_or(0))
        })
        .max()
        .unwrap_or(0)
}

/// Determine the id zero-padding width from existing filenames.
pub fn id_width(names: &[String]) -> usize {
    names
        .iter()
        .filter_map(|n| RE_FILENAME_ID.captures(n).map(|c| c[1].len()))
        .max()
        .unwrap_or(2)
}

/// Find a ticket by id in the corpus.
pub fn find_ticket<'a>(corpus: &'a [Ticket], id: &str) -> Result<&'a Ticket> {
    corpus
        .iter()
        .find(|t| t.id == id)
        .ok_or_else(|| anyhow::anyhow!("no ticket with id {}", id))
}

// --- Helpers ---

fn is_fence(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed == "---"
}

/// Parse a blocked_by field value into a Vec of IDs.
/// Supports inline array `["01", "03"]`, block-style YAML list, and bare scalars.
fn parse_blocked_by(raw: &str) -> Vec<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    // Inline array format: ["01", "03"]
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        let inner = &trimmed[1..trimmed.len() - 1];
        return inner
            .split(',')
            .map(|s| yaml_scalar_unescape(s.trim().trim_matches('"').trim_matches('\'')))
            .filter(|s| !s.is_empty())
            .collect();
    }
    // Multi-line YAML list format:
    //   - "01"
    //   - "03"
    if raw.contains('\n') {
        return raw
            .split('\n')
            .map(|line| line.trim())
            .filter(|line| line.starts_with('-'))
            .map(|line| {
                let val = line[1..].trim();
                yaml_scalar_unescape(val.trim_matches('"').trim_matches('\''))
            })
            .filter(|s| !s.is_empty())
            .collect();
    }
    // Bare scalar: "01, 04" or "01"
    trimmed
        .split(',')
        .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Parse validation_criteria from frontmatter.
/// Supports both inline array `["a", "b"]` and multi-line YAML list:
/// ```yaml
/// validation_criteria:
///   - "cargo test passes"
///   - "login returns JWT"
/// ```
fn parse_validation_criteria(raw: &str) -> Vec<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    // Inline array format: ["a", "b"]
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        let inner = &trimmed[1..trimmed.len() - 1];
        return inner
            .split(',')
            .map(|s| yaml_scalar_unescape(s.trim().trim_matches('"').trim_matches('\'')))
            .filter(|s| !s.is_empty())
            .collect();
    }
    // Multi-line YAML list format (continuation lines joined with newlines):
    //   - "criterion one"
    //   - "criterion two"
    raw.split('\n')
        .map(|line| line.trim())
        .filter(|line| line.starts_with('-'))
        .map(|line| {
            let val = line[1..].trim();
            yaml_scalar_unescape(val.trim_matches('"').trim_matches('\''))
        })
        .filter(|s| !s.is_empty())
        .collect()
}

/// Parse tags from frontmatter. Same format as validation_criteria.
fn parse_tags(raw: &str) -> Vec<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        let inner = &trimmed[1..trimmed.len() - 1];
        return inner
            .split(',')
            .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }
    raw.split('\n')
        .map(|line| line.trim())
        .filter(|line| line.starts_with('-'))
        .map(|line| {
            line[1..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string()
        })
        .filter(|s| !s.is_empty())
        .collect()
}

// --- YAML/JSON escaping ---

/// Escape a string for use inside YAML double-quoted scalars.
pub fn yaml_scalar_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\0' => out.push_str("\\0"),
            c if c.is_control() => {
                out.push_str(&format!("\\x{:02X}", c as u32));
            }
            _ => out.push(c),
        }
    }
    out
}

/// Decode YAML double-quoted scalar escapes when reading values.
pub fn yaml_scalar_unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('\\') => out.push('\\'),
                Some('"') => out.push('"'),
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some('0') => out.push('\0'),
                Some('x') => {
                    let hex: String = chars.by_ref().take(2).collect();
                    if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                        out.push(byte as char);
                    } else {
                        out.push_str("\\x");
                        out.push_str(&hex);
                    }
                }
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Escape a string for use inside a JSON string value (between quotes).
pub fn json_string_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {
                out.push_str(&format!("\\u{:04X}", c as u32));
            }
            _ => out.push(c),
        }
    }
    out
}

/// Parameters for creating a new ticket file.
pub struct NewTicketParams<'a> {
    pub id: &'a str,
    pub title: &'a str,
    pub blocked_by: &'a [String],
    pub env: Option<&'a str>,
    pub spec: Option<&'a str>,
    pub priority: Option<&'a str>,
    pub status: Option<&'a str>,
    pub validation_criteria: &'a [String],
    pub tags: &'a [String],
    pub requires: &'a [String],
}

/// Generate the text for a new ticket file.
pub fn new_ticket_text(p: &NewTicketParams) -> String {
    let status_val = p.status.unwrap_or("open");
    let mut fm_lines = vec![
        format!("id: \"{}\"", yaml_scalar_escape(p.id)),
        format!("title: \"{}\"", yaml_scalar_escape(p.title)),
        format!("status: {}", status_val),
    ];
    let deps = p
        .blocked_by
        .iter()
        .map(|d| format!("\"{}\"", yaml_scalar_escape(d)))
        .collect::<Vec<_>>()
        .join(", ");
    fm_lines.push(format!("blocked_by: [{}]", deps));
    if let Some(e) = p.env {
        fm_lines.push(format!("env: {}", e));
    }
    if let Some(s) = p.spec {
        fm_lines.push(format!("spec: \"{}\"", yaml_scalar_escape(s)));
    }
    if let Some(prio) = p.priority {
        fm_lines.push(format!("priority: {}", prio));
    }
    if !p.validation_criteria.is_empty() {
        fm_lines.push("validation_criteria:".to_string());
        for vc in p.validation_criteria {
            fm_lines.push(format!("  - \"{}\"", yaml_scalar_escape(vc)));
        }
    }
    if !p.tags.is_empty() {
        let tags_str = p
            .tags
            .iter()
            .map(|t| format!("\"{}\"", yaml_scalar_escape(t)))
            .collect::<Vec<_>>()
            .join(", ");
        fm_lines.push(format!("tags: [{}]", tags_str));
    }
    if !p.requires.is_empty() {
        let req_str = p
            .requires
            .iter()
            .map(|r| format!("\"{}\"", yaml_scalar_escape(r)))
            .collect::<Vec<_>>()
            .join(", ");
        fm_lines.push(format!("requires: [{}]", req_str));
    }
    let body = format!(
        "\n# {}\n\n## What to build\n\nTBD\n\n## Acceptance criteria\n\n- [ ] TBD\n",
        p.title
    );
    format!("---\n{}\n---\n{}", fm_lines.join("\n"), body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn parse_basic_ticket() {
        let mut f = NamedTempFile::new().unwrap();
        write!(f, "---\nid: \"01\"\ntitle: \"Test ticket\"\nstatus: open\nblocked_by: []\n---\n\n# Test\n\n- [ ] AC1\n").unwrap();
        let t = Ticket::parse(f.path()).unwrap();
        assert_eq!(t.id, "01");
        assert_eq!(t.title, "Test ticket");
        assert_eq!(t.status, Status::Open);
        assert!(t.blocked_by.is_empty());
    }

    #[test]
    fn parse_ticket_with_deps() {
        let mut f = NamedTempFile::new().unwrap();
        write!(f, "---\nid: \"05\"\ntitle: \"Depends on others\"\nstatus: open\nblocked_by: [\"01\", \"03\"]\npriority: high\nenv: corp\nspec: \"my-spec\"\n---\n\n# Body\n").unwrap();
        let t = Ticket::parse(f.path()).unwrap();
        assert_eq!(t.blocked_by, vec!["01", "03"]);
        assert!(t.is_high_priority());
        assert_eq!(t.env, Env::Corp);
        assert_eq!(t.spec.as_deref(), Some("my-spec"));
    }

    #[test]
    fn parse_blocked_by_block_style() {
        let content = "---\nid: \"05\"\ntitle: \"Block deps\"\nstatus: open\nblocked_by:\n  - \"01\"\n  - \"03\"\n---\n\n# Body\n";
        let t = Ticket::parse_str(content, Path::new("t.md")).unwrap();
        assert_eq!(t.blocked_by, vec!["01", "03"]);
    }

    #[test]
    fn parse_blocked_by_bare_scalar() {
        let content =
            "---\nid: \"05\"\ntitle: \"Bare\"\nstatus: open\nblocked_by: 01, 04\n---\n\n# Body\n";
        let t = Ticket::parse_str(content, Path::new("t.md")).unwrap();
        assert_eq!(t.blocked_by, vec!["01", "04"]);
    }

    #[test]
    fn parse_blocked_by_single_bare() {
        let content =
            "---\nid: \"05\"\ntitle: \"Bare\"\nstatus: open\nblocked_by: 01\n---\n\n# Body\n";
        let t = Ticket::parse_str(content, Path::new("t.md")).unwrap();
        assert_eq!(t.blocked_by, vec!["01"]);
    }

    #[test]
    fn parse_tolerates_utf8_bom() {
        let content = "\u{FEFF}---\nid: \"05\"\ntitle: \"BOM\"\nstatus: open\nblocked_by: []\n---\n\n# Body\n";
        let t = Ticket::parse_str(content, Path::new("t.md")).unwrap();
        assert_eq!(t.id, "05");
        assert_eq!(t.title, "BOM");
    }

    #[test]
    fn parse_tolerates_comment_lines() {
        let content = "---\n# this is a note\nid: \"05\"\ntitle: \"Commented\"\n  # indented note\nstatus: open\nblocked_by: []\n---\n\n# Body\n";
        let t = Ticket::parse_str(content, Path::new("t.md")).unwrap();
        assert_eq!(t.id, "05");
        assert_eq!(t.status, Status::Open);
    }

    #[test]
    fn parse_tolerates_space_before_colon() {
        let content =
            "---\nid : \"05\"\ntitle : \"Spaced\"\nstatus : open\nblocked_by : []\n---\n\n# Body\n";
        let t = Ticket::parse_str(content, Path::new("t.md")).unwrap();
        assert_eq!(t.id, "05");
        assert_eq!(t.title, "Spaced");
        assert_eq!(t.status, Status::Open);
    }

    #[test]
    fn parse_still_bails_on_missing_opening_fence() {
        let content = "id: \"05\"\ntitle: \"No fence\"\nstatus: open\nblocked_by: []\n";
        assert!(Ticket::parse_str(content, Path::new("t.md")).is_err());
    }

    #[test]
    fn parse_still_bails_on_missing_closing_fence() {
        let content = "---\nid: \"05\"\ntitle: \"No close\"\nstatus: open\nblocked_by: []\n\n# Body never closed\n";
        assert!(Ticket::parse_str(content, Path::new("t.md")).is_err());
    }

    #[test]
    fn parse_still_bails_on_garbage_line() {
        let content = "---\nid: \"05\"\ntitle: \"Garbage\"\nthis is not valid frontmatter\nstatus: open\nblocked_by: []\n---\n\n# Body\n";
        assert!(Ticket::parse_str(content, Path::new("t.md")).is_err());
    }

    #[test]
    fn frontier_filters_correctly() {
        let content_done = "---\nid: \"01\"\ntitle: \"Done\"\nstatus: done\nblocked_by: []\n---\n";
        let content_open =
            "---\nid: \"02\"\ntitle: \"Open\"\nstatus: open\nblocked_by: [\"01\"]\n---\n";
        let content_blocked =
            "---\nid: \"03\"\ntitle: \"Blocked\"\nstatus: open\nblocked_by: [\"99\"]\n---\n";

        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("01-done.md"), content_done).unwrap();
        std::fs::write(dir.path().join("02-open.md"), content_open).unwrap();
        std::fs::write(dir.path().join("03-blocked.md"), content_blocked).unwrap();

        let corpus = load_corpus(dir.path()).unwrap();
        let front = frontier(&corpus);
        assert_eq!(front.len(), 1);
        assert_eq!(front[0].id, "02");
    }

    #[test]
    fn frontier_excludes_backlog() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("01-open.md"),
            "---\nid: \"01\"\ntitle: \"Open\"\nstatus: open\nblocked_by: []\n---\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("02-backlog.md"),
            "---\nid: \"02\"\ntitle: \"Backlog\"\nstatus: backlog\nblocked_by: []\n---\n",
        )
        .unwrap();

        let corpus = load_corpus(dir.path()).unwrap();
        let front = frontier(&corpus);
        assert_eq!(front.len(), 1);
        assert_eq!(front[0].id, "01");
    }

    #[test]
    fn frontier_sorts_by_priority_buckets() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("01-low.md"),
            "---\nid: \"01\"\ntitle: \"Low\"\nstatus: open\nblocked_by: []\npriority: low\n---\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("02-none.md"),
            "---\nid: \"02\"\ntitle: \"Default\"\nstatus: open\nblocked_by: []\n---\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("03-urgent.md"),
            "---\nid: \"03\"\ntitle: \"Urgent\"\nstatus: open\nblocked_by: []\npriority: urgent\n---\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("04-high.md"),
            "---\nid: \"04\"\ntitle: \"High\"\nstatus: open\nblocked_by: []\npriority: high\n---\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("05-medium.md"),
            "---\nid: \"05\"\ntitle: \"Medium\"\nstatus: open\nblocked_by: []\npriority: medium\n---\n",
        )
        .unwrap();

        let corpus = load_corpus(dir.path()).unwrap();
        let front = frontier(&corpus);
        let ids: Vec<&str> = front.iter().map(|t| t.id.as_str()).collect();
        // urgent(03) → high(04) → default/medium(02,05) → low(01)
        assert_eq!(ids, vec!["03", "04", "02", "05", "01"]);
    }

    #[test]
    fn surgical_edit_preserves_unknown_fields() {
        let content = "---\nid: \"01\"\ntitle: \"Test\"\nstatus: open\nblocked_by: []\ncustom_field: hello\n---\n\n# Body\n";
        let mut t = Ticket::parse_str(content, Path::new("test.md")).unwrap();
        t.file.set_field("status", "done");
        let out = t.file.serialize();
        assert!(out.contains("status: done"));
        assert!(out.contains("custom_field: hello"));
    }

    #[test]
    fn yaml_escape_handles_special_chars() {
        assert_eq!(yaml_scalar_escape("hello"), "hello");
        assert_eq!(yaml_scalar_escape(""), "");
        assert_eq!(yaml_scalar_escape(r#"Fix "ready""#), r#"Fix \"ready\""#);
        assert_eq!(yaml_scalar_escape("back\\slash"), "back\\\\slash");
        assert_eq!(yaml_scalar_escape("line\nbreak"), "line\\nbreak");
        assert_eq!(yaml_scalar_escape("tab\there"), "tab\\there");
        assert_eq!(yaml_scalar_escape("cr\rhere"), "cr\\rhere");
        assert_eq!(yaml_scalar_escape("null\0byte"), "null\\0byte");
        assert_eq!(yaml_scalar_escape("unicode: café"), "unicode: café");
        assert_eq!(yaml_scalar_escape("a\"b\\c\nd"), "a\\\"b\\\\c\\nd");
    }

    #[test]
    fn json_escape_handles_special_chars() {
        assert_eq!(json_string_escape("hello"), "hello");
        assert_eq!(json_string_escape(""), "");
        assert_eq!(json_string_escape(r#"has "quotes""#), r#"has \"quotes\""#);
        assert_eq!(json_string_escape("back\\slash"), "back\\\\slash");
        assert_eq!(json_string_escape("line\nbreak"), "line\\nbreak");
        assert_eq!(json_string_escape("tab\there"), "tab\\there");
        assert_eq!(json_string_escape("cr\rhere"), "cr\\rhere");
        assert_eq!(json_string_escape("unicode: café"), "unicode: café");
        let with_ctrl = format!("ctrl{}here", '\x01');
        assert_eq!(json_string_escape(&with_ctrl), "ctrl\\u0001here");
        assert_eq!(json_string_escape("a\"b\\c\nd"), "a\\\"b\\\\c\\nd");
    }

    #[test]
    fn new_ticket_text_escapes_title() {
        let text = new_ticket_text(&NewTicketParams {
            id: "01",
            title: "Fix \"ready\" command",
            blocked_by: &[],
            env: None,
            spec: None,
            priority: None,
            status: None,
            validation_criteria: &[],
            tags: &[],
            requires: &[],
        });
        assert!(text.contains(r#"title: "Fix \"ready\" command""#));
        let t = Ticket::parse_str(&text, Path::new("test.md")).unwrap();
        assert!(t.title.contains("ready"));
    }

    #[test]
    fn status_enum_parse() {
        assert_eq!(Status::parse("open").unwrap(), Status::Open);
        assert_eq!(Status::parse("in_progress").unwrap(), Status::InProgress);
        assert_eq!(Status::parse("done").unwrap(), Status::Done);
        assert!(Status::parse("invalid").is_err());
    }

    #[test]
    fn env_enum_parse() {
        assert_eq!(Env::parse("corp").unwrap(), Env::Corp);
        assert_eq!(Env::parse("personal").unwrap(), Env::Personal);
        assert_eq!(Env::parse("either").unwrap(), Env::Either);
        assert!(Env::parse("invalid").is_err());
    }

    #[test]
    fn invalid_status_rejected_at_parse() {
        let content =
            "---\nid: \"01\"\ntitle: \"Bad\"\nstatus: invalid\nblocked_by: []\n---\n\n# Bad\n";
        let result = Ticket::parse_str(content, Path::new("test.md"));
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("status") || msg.contains("invalid"));
    }

    #[test]
    fn invalid_env_rejected_at_parse() {
        let content = "---\nid: \"01\"\ntitle: \"Bad\"\nstatus: open\nblocked_by: []\nenv: bogus\n---\n\n# Bad\n";
        let result = Ticket::parse_str(content, Path::new("test.md"));
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("env") || msg.contains("bogus"));
    }

    // --- Typed mutation method tests ---

    #[test]
    fn set_status_typed() {
        let content =
            "---\nid: \"01\"\ntitle: \"Test\"\nstatus: open\nblocked_by: []\n---\n\n# Body\n";
        let mut file = TicketFile::parse_str(content, Path::new("t.md")).unwrap();
        file.set_status(Status::Done);
        let out = file.serialize();
        assert!(out.contains("status: done"));
        assert!(!out.contains("status: open"));
    }

    #[test]
    fn set_status_preserves_other_fields() {
        let content = "---\nid: \"01\"\ntitle: \"Test\"\nstatus: open\nblocked_by: [\"02\"]\npriority: high\n---\n\n# Body\n";
        let mut file = TicketFile::parse_str(content, Path::new("t.md")).unwrap();
        file.set_status(Status::InProgress);
        let out = file.serialize();
        assert!(out.contains("status: in_progress"));
        assert!(out.contains("priority: high"));
        assert!(out.contains("blocked_by: [\"02\"]"));
    }

    #[test]
    fn set_blocked_by_empty() {
        let content = "---\nid: \"01\"\ntitle: \"Test\"\nstatus: open\nblocked_by: [\"02\", \"03\"]\n---\n\n# Body\n";
        let mut file = TicketFile::parse_str(content, Path::new("t.md")).unwrap();
        let empty: Vec<&str> = vec![];
        file.set_blocked_by(&empty);
        let out = file.serialize();
        assert!(out.contains("blocked_by: []"));
    }

    #[test]
    fn set_blocked_by_multiple() {
        let content =
            "---\nid: \"01\"\ntitle: \"Test\"\nstatus: open\nblocked_by: []\n---\n\n# Body\n";
        let mut file = TicketFile::parse_str(content, Path::new("t.md")).unwrap();
        file.set_blocked_by(&["05", "10"]);
        let out = file.serialize();
        assert!(out.contains("blocked_by: [\"05\", \"10\"]"));
    }

    #[test]
    fn set_blocked_by_escapes_values() {
        let content =
            "---\nid: \"01\"\ntitle: \"Test\"\nstatus: open\nblocked_by: []\n---\n\n# Body\n";
        let mut file = TicketFile::parse_str(content, Path::new("t.md")).unwrap();
        file.set_blocked_by(&["01"]);
        let out = file.serialize();
        assert!(out.contains("blocked_by: [\"01\"]"));
    }

    #[test]
    fn set_priority_some() {
        let content =
            "---\nid: \"01\"\ntitle: \"Test\"\nstatus: open\nblocked_by: []\n---\n\n# Body\n";
        let mut file = TicketFile::parse_str(content, Path::new("t.md")).unwrap();
        file.set_priority(Some(Priority::Urgent));
        let out = file.serialize();
        assert!(out.contains("priority: urgent"));
    }

    #[test]
    fn set_priority_none_removes_field() {
        let content = "---\nid: \"01\"\ntitle: \"Test\"\nstatus: open\nblocked_by: []\npriority: high\n---\n\n# Body\n";
        let mut file = TicketFile::parse_str(content, Path::new("t.md")).unwrap();
        file.set_priority(None);
        let out = file.serialize();
        assert!(!out.contains("priority"));
    }

    #[test]
    fn set_env_some() {
        let content =
            "---\nid: \"01\"\ntitle: \"Test\"\nstatus: open\nblocked_by: []\n---\n\n# Body\n";
        let mut file = TicketFile::parse_str(content, Path::new("t.md")).unwrap();
        file.set_env(Some(Env::Corp));
        let out = file.serialize();
        assert!(out.contains("env: corp"));
    }

    #[test]
    fn set_env_none_removes_field() {
        let content = "---\nid: \"01\"\ntitle: \"Test\"\nstatus: open\nblocked_by: []\nenv: personal\n---\n\n# Body\n";
        let mut file = TicketFile::parse_str(content, Path::new("t.md")).unwrap();
        file.set_env(None);
        let out = file.serialize();
        assert!(!out.contains("env"));
    }

    #[test]
    fn append_resolution_basic() {
        let content = "---\nid: \"01\"\ntitle: \"Test\"\nstatus: open\nblocked_by: []\n---\n\n# Test\n\n## Acceptance criteria\n\n- [x] Done\n";
        let mut file = TicketFile::parse_str(content, Path::new("t.md")).unwrap();
        file.append_resolution("2026-08-10", "Shipped it", None);
        assert!(file.body.contains("## Resolution (2026-08-10)"));
        assert!(file.body.contains("Shipped it"));
        assert!(!file.body.contains("Spike branch"));
    }

    #[test]
    fn append_resolution_with_spike() {
        let content =
            "---\nid: \"01\"\ntitle: \"Test\"\nstatus: open\nblocked_by: []\n---\n\n# Test\n";
        let mut file = TicketFile::parse_str(content, Path::new("t.md")).unwrap();
        file.append_resolution("2026-08-10", "Validated", Some("spike/experiment"));
        assert!(file.body.contains("## Resolution (2026-08-10)"));
        assert!(file.body.contains("Validated"));
        assert!(file.body.contains("Spike branch: spike/experiment"));
    }

    #[test]
    fn append_resolution_idempotent() {
        let content = "---\nid: \"01\"\ntitle: \"Test\"\nstatus: open\nblocked_by: []\n---\n\n# Test\n\n## Resolution (2026-01-01)\n\nAlready here\n";
        let mut file = TicketFile::parse_str(content, Path::new("t.md")).unwrap();
        let before = file.body.clone();
        file.append_resolution("2026-08-10", "New note", None);
        assert_eq!(file.body, before);
    }

    #[test]
    fn check_acs_all() {
        let content = "---\nid: \"01\"\ntitle: \"Test\"\nstatus: open\nblocked_by: []\n---\n\n# Test\n\n## Acceptance criteria\n\n- [ ] First\n- [ ] Second\n- [x] Already done\n";
        let mut file = TicketFile::parse_str(content, Path::new("t.md")).unwrap();
        let stats = file.check_acs(AcSelection::All);
        assert_eq!(stats.total, 3);
        assert_eq!(stats.checked, 3);
        assert_eq!(stats.unchecked, 0);
        assert!(!file.body.contains("- [ ]"));
    }

    #[test]
    fn check_acs_indices() {
        let content = "---\nid: \"01\"\ntitle: \"Test\"\nstatus: open\nblocked_by: []\n---\n\n# Test\n\n## Acceptance criteria\n\n- [ ] First\n- [ ] Second\n- [ ] Third\n";
        let mut file = TicketFile::parse_str(content, Path::new("t.md")).unwrap();
        let stats = file.check_acs(AcSelection::Indices(&[1, 3]));
        assert_eq!(stats.total, 3);
        assert_eq!(stats.checked, 2);
        assert_eq!(stats.unchecked, 1);
        // Second item should still be unchecked
        assert!(file.body.contains("- [x] First"));
        assert!(file.body.contains("- [ ] Second"));
        assert!(file.body.contains("- [x] Third"));
    }

    #[test]
    fn check_acs_no_section() {
        let content = "---\nid: \"01\"\ntitle: \"Test\"\nstatus: open\nblocked_by: []\n---\n\n# Test\n\nNo AC section here.\n";
        let mut file = TicketFile::parse_str(content, Path::new("t.md")).unwrap();
        let stats = file.check_acs(AcSelection::All);
        assert_eq!(stats.total, 0);
    }

    #[test]
    fn ac_stats_without_mutation() {
        let content = "---\nid: \"01\"\ntitle: \"Test\"\nstatus: open\nblocked_by: []\n---\n\n# Test\n\n## Acceptance criteria\n\n- [x] Done\n- [ ] Not done\n- [ ] Also not\n";
        let file = TicketFile::parse_str(content, Path::new("t.md")).unwrap();
        let stats = file.ac_stats();
        assert_eq!(stats.total, 3);
        assert_eq!(stats.checked, 1);
        assert_eq!(stats.unchecked, 2);
    }

    #[test]
    fn parse_validation_criteria_multiline() {
        let content = "---\nid: \"01\"\ntitle: \"Test\"\nstatus: open\nblocked_by: []\nvalidation_criteria:\n  - \"cargo test passes\"\n  - \"clippy zero warnings\"\n  - \"integration test covers flow\"\n---\n\n# Test\n";
        let ticket = Ticket::parse_str(content, Path::new("t.md")).unwrap();
        assert_eq!(ticket.validation_criteria.len(), 3);
        assert_eq!(ticket.validation_criteria[0], "cargo test passes");
        assert_eq!(ticket.validation_criteria[1], "clippy zero warnings");
        assert_eq!(
            ticket.validation_criteria[2],
            "integration test covers flow"
        );
    }

    #[test]
    fn parse_validation_criteria_inline_array() {
        let content = "---\nid: \"01\"\ntitle: \"Test\"\nstatus: open\nblocked_by: []\nvalidation_criteria: [\"test passes\", \"lint clean\"]\n---\n\n# Test\n";
        let ticket = Ticket::parse_str(content, Path::new("t.md")).unwrap();
        assert_eq!(ticket.validation_criteria.len(), 2);
        assert_eq!(ticket.validation_criteria[0], "test passes");
        assert_eq!(ticket.validation_criteria[1], "lint clean");
    }

    #[test]
    fn parse_validation_criteria_empty_when_absent() {
        let content =
            "---\nid: \"01\"\ntitle: \"Test\"\nstatus: open\nblocked_by: []\n---\n\n# Test\n";
        let ticket = Ticket::parse_str(content, Path::new("t.md")).unwrap();
        assert!(ticket.validation_criteria.is_empty());
    }

    #[test]
    fn set_validation_criteria_roundtrip() {
        let content =
            "---\nid: \"01\"\ntitle: \"Test\"\nstatus: open\nblocked_by: []\n---\n\n# Test\n";
        let mut file = TicketFile::parse_str(content, Path::new("t.md")).unwrap();
        file.set_validation_criteria(&["cargo test passes", "clippy clean"]);
        let serialized = file.serialize();
        let reparsed = Ticket::parse_str(&serialized, Path::new("t.md")).unwrap();
        assert_eq!(reparsed.validation_criteria.len(), 2);
        assert_eq!(reparsed.validation_criteria[0], "cargo test passes");
        assert_eq!(reparsed.validation_criteria[1], "clippy clean");
    }
}

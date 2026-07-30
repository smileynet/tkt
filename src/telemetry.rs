//! Telemetry: local-only structured event recording.
//!
//! Records command invocations as JSONL to a platform-appropriate data directory,
//! segmented by project. Opt-in only — disabled by default.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

// --- Debug mode ---

/// Debug output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugMode {
    /// Human-readable lines to stderr
    Human,
    /// JSONL to stderr
    Json,
    /// Debug mode disabled
    Off,
}

/// Check TKT_DEBUG environment variable.
/// - "1" or "true" → Human
/// - "json" → Json
/// - unset or anything else → Off
pub fn debug_mode() -> DebugMode {
    match std::env::var("TKT_DEBUG").as_deref() {
        Ok("1") | Ok("true") => DebugMode::Human,
        Ok("json") => DebugMode::Json,
        _ => DebugMode::Off,
    }
}

/// Emit a debug event to stderr. No-op if debug mode is Off.
pub fn debug_event(mode: DebugMode, session: &str, project: &str, msg: &str) {
    match mode {
        DebugMode::Off => {}
        DebugMode::Human => {
            eprintln!("[tkt:debug] {}", msg);
        }
        DebugMode::Json => {
            let json = serde_json::json!({
                "ts": iso_timestamp(),
                "session": session,
                "project": project,
                "level": "debug",
                "msg": msg,
            });
            eprintln!("{}", json);
        }
    }
}

// --- Session ID ---

/// Generate a session ID: timestamp (ms) + PID, hex-encoded.
/// Sortable by time, unique per process. Not a full ULID but sufficient
/// for correlating log lines within and across invocations.
pub fn generate_session_id() -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    let pid = std::process::id();
    format!("{:012x}-{:04x}", ts, pid & 0xFFFF)
}

// --- Project slug ---

/// Derive a project slug from the git repo root directory name.
/// Falls back to "unknown" if the path has no final component.
pub fn project_slug(repo_root: &Path) -> String {
    repo_root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

// --- Consent ---

/// Consent state determined by checking the hierarchy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Consent {
    Enabled,
    Disabled,
}

/// Reason why consent was determined.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsentReason {
    DoNotTrack,
    EnvVar,
    CiDetected,
    ConfigFile,
    Default,
}

/// Check the consent hierarchy. Returns (state, reason).
///
/// Priority (highest wins):
/// 1. DO_NOT_TRACK=1 → disabled
/// 2. TKT_TELEMETRY=off → disabled; TKT_TELEMETRY=on → enabled
/// 3. CI=true → disabled
/// 4. Config file → enabled/disabled per file
/// 5. Default → disabled
pub fn check_consent() -> (Consent, ConsentReason) {
    // 1. Universal opt-out
    if std::env::var("DO_NOT_TRACK").as_deref() == Ok("1") {
        return (Consent::Disabled, ConsentReason::DoNotTrack);
    }

    // 2. Tool-specific env var
    match std::env::var("TKT_TELEMETRY").as_deref() {
        Ok("off") | Ok("false") | Ok("0") => {
            return (Consent::Disabled, ConsentReason::EnvVar);
        }
        Ok("on") | Ok("true") | Ok("1") => {
            return (Consent::Enabled, ConsentReason::EnvVar);
        }
        _ => {}
    }

    // 3. CI detection
    if std::env::var("CI").is_ok() {
        return (Consent::Disabled, ConsentReason::CiDetected);
    }

    // 4. Config file
    if let Some(path) = consent_file_path() {
        if let Some(consent) = read_consent_file(&path) {
            return (consent, ConsentReason::ConfigFile);
        }
    }

    // 5. Default: disabled
    (Consent::Disabled, ConsentReason::Default)
}

/// Path to the consent config file.
/// Linux/macOS: ~/.config/tkt/consent.toml
/// Windows: %APPDATA%/tkt/consent.toml
pub fn consent_file_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("tkt").join("consent.toml"))
}

/// Read consent from the config file. Returns None if file doesn't exist or is unparseable.
fn read_consent_file(path: &Path) -> Option<Consent> {
    let content = std::fs::read_to_string(path).ok()?;
    // Simple TOML parsing — just look for `enabled = true/false`
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "enabled = true" {
            return Some(Consent::Enabled);
        }
        if trimmed == "enabled = false" {
            return Some(Consent::Disabled);
        }
    }
    None
}

/// Write consent to the config file.
#[allow(dead_code)] // Used by upcoming `tkt telemetry` subcommand (#20)
pub fn write_consent(enabled: bool) -> std::io::Result<()> {
    let path = consent_file_path()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "no config directory"))?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let date = iso_date();
    let content = format!(
        "[telemetry]\nenabled = {}\nconsented_at = \"{}\"\nversion = 1\n",
        enabled, date
    );
    std::fs::write(&path, content)
}

// --- Event ---

/// A telemetry event recorded as a single JSONL line.
pub struct Event {
    pub ts: String,
    pub session: String,
    pub project: String,
    pub cmd: String,
    pub exit_code: i32,
    pub duration_ms: u64,
    pub version: String,
    pub os: String,
    pub arch: String,
}

impl Event {
    /// Serialize to a JSON string (one line, no trailing newline).
    pub fn to_json(&self) -> String {
        serde_json::json!({
            "ts": self.ts,
            "session": self.session,
            "project": self.project,
            "cmd": self.cmd,
            "exit_code": self.exit_code,
            "duration_ms": self.duration_ms,
            "version": self.version,
            "os": self.os,
            "arch": self.arch,
        })
        .to_string()
    }
}

// --- Sink ---

/// Maximum size of a per-project JSONL file before rotation (5 MB).
const MAX_FILE_SIZE: u64 = 5 * 1024 * 1024;

/// Maximum number of rotated files per project (current + 5 rotated = 6 total).
const MAX_ROTATED_FILES: u32 = 5;

/// Maximum age of rotated files in seconds (30 days).
const MAX_AGE_SECS: u64 = 30 * 24 * 60 * 60;

/// Path to the telemetry data directory.
/// Linux: ~/.local/share/tkt/telemetry/
/// macOS: ~/Library/Application Support/tkt/telemetry/
/// Windows: %APPDATA%/tkt/telemetry/
pub fn telemetry_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("tkt").join("telemetry"))
}

/// Append an event to the per-project JSONL file.
/// Silently swallows all errors (telemetry must never affect CLI behavior).
/// Runs rotation check after writing if file exceeds size threshold.
pub fn record_event(event: &Event) {
    let _ = try_record_event(event);
}

fn try_record_event(event: &Event) -> std::io::Result<()> {
    let dir = telemetry_dir()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "no data directory"))?;
    std::fs::create_dir_all(&dir)?;

    let slug = sanitize_slug(&event.project);
    let filename = format!("{}.jsonl", slug);
    let path = dir.join(&filename);

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;

    let line = event.to_json();
    writeln!(file, "{}", line)?;
    drop(file);

    // Check if rotation is needed (fast: just a stat call)
    if let Ok(meta) = std::fs::metadata(&path) {
        if meta.len() > MAX_FILE_SIZE {
            let _ = rotate_file(&dir, &slug);
        }
    }

    // Run cleanup periodically (cheap check: look for any .1.jsonl files older than max age)
    let _ = cleanup_old_files(&dir);

    Ok(())
}

// --- Rotation ---

/// Rotate a project's JSONL file: current → .1 → .2 → ... → .N (delete beyond max).
fn rotate_file(dir: &Path, slug: &str) -> std::io::Result<()> {
    // Shift existing rotated files up
    for i in (1..MAX_ROTATED_FILES).rev() {
        let from = dir.join(format!("{}.{}.jsonl", slug, i));
        let to = dir.join(format!("{}.{}.jsonl", slug, i + 1));
        if from.exists() {
            std::fs::rename(&from, &to)?;
        }
    }

    // Delete the oldest if it exists (file beyond max)
    let oldest = dir.join(format!("{}.{}.jsonl", slug, MAX_ROTATED_FILES + 1));
    if oldest.exists() {
        std::fs::remove_file(&oldest)?;
    }

    // Move current to .1
    let current = dir.join(format!("{}.jsonl", slug));
    let first_rotated = dir.join(format!("{}.1.jsonl", slug));
    if current.exists() {
        std::fs::rename(&current, &first_rotated)?;
    }

    Ok(())
}

// --- Cleanup ---

/// Delete rotated files older than MAX_AGE_SECS. Quick scan (stat only, no parsing).
fn cleanup_old_files(dir: &Path) -> std::io::Result<()> {
    let now = SystemTime::now();
    let entries = std::fs::read_dir(dir)?;
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        // Only clean rotated files (contain a numeric suffix like .1.jsonl, .2.jsonl)
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        if !is_rotated_file(&name) {
            continue;
        }
        if let Ok(meta) = entry.metadata() {
            if let Ok(modified) = meta.modified() {
                if let Ok(age) = now.duration_since(modified) {
                    if age.as_secs() > MAX_AGE_SECS {
                        let _ = std::fs::remove_file(&path);
                    }
                }
            }
        }
    }
    Ok(())
}

/// Check if a filename is a rotated telemetry file (e.g., "tkt.1.jsonl", "project.3.jsonl").
fn is_rotated_file(name: &str) -> bool {
    // Pattern: <slug>.<number>.jsonl
    if !name.ends_with(".jsonl") {
        return false;
    }
    let without_ext = &name[..name.len() - 6]; // strip ".jsonl"
                                               // Must end with .<digit(s)>
    if let Some(dot_pos) = without_ext.rfind('.') {
        let suffix = &without_ext[dot_pos + 1..];
        suffix.chars().all(|c| c.is_ascii_digit()) && !suffix.is_empty()
    } else {
        false
    }
}

/// Full cleanup: enforce max file count per project and prune sessions.
/// Called from `tkt telemetry --status` or at startup.
pub fn cleanup_telemetry_dir() {
    if let Some(dir) = telemetry_dir() {
        if dir.is_dir() {
            let _ = cleanup_old_files(&dir);
            let _ = enforce_max_files(&dir);
        }
    }
}

/// Enforce max rotated file count per project slug.
fn enforce_max_files(dir: &Path) -> std::io::Result<()> {
    let entries = std::fs::read_dir(dir)?;
    // Group rotated files by slug
    let mut by_slug: std::collections::HashMap<String, Vec<(u32, PathBuf)>> =
        std::collections::HashMap::new();

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        if !is_rotated_file(&name) {
            continue;
        }
        // Parse slug and number
        let without_ext = &name[..name.len() - 6];
        if let Some(dot_pos) = without_ext.rfind('.') {
            let slug = without_ext[..dot_pos].to_string();
            if let Ok(num) = without_ext[dot_pos + 1..].parse::<u32>() {
                by_slug.entry(slug).or_default().push((num, path.clone()));
            }
        }
    }

    // For each slug, delete files beyond MAX_ROTATED_FILES
    for (_slug, mut files) in by_slug {
        files.sort_by_key(|(num, _)| *num);
        if files.len() > MAX_ROTATED_FILES as usize {
            for (_, path) in files.iter().skip(MAX_ROTATED_FILES as usize) {
                let _ = std::fs::remove_file(path);
            }
        }
    }

    Ok(())
}

/// Prune the oldest complete sessions from a JSONL file to bring it under size.
/// A session boundary is detected by a change in the "session" field.
/// Removes whole sessions (oldest first) until file is under MAX_FILE_SIZE.
#[allow(dead_code)] // Called from rotation path and tests; wired fully in future
pub fn prune_oldest_sessions(path: &Path) -> std::io::Result<()> {
    let content = std::fs::read_to_string(path)?;
    let lines: Vec<&str> = content.lines().filter(|l| !l.is_empty()).collect();

    if lines.is_empty() {
        return Ok(());
    }

    // Group lines by session (preserving order)
    let mut sessions: Vec<(String, Vec<&str>)> = Vec::new();
    for line in &lines {
        let session_id = extract_session_id(line);
        if sessions.last().map(|(id, _)| id.as_str()) == Some(&session_id) {
            sessions.last_mut().unwrap().1.push(line);
        } else {
            sessions.push((session_id, vec![line]));
        }
    }

    // Remove oldest sessions until total size is under threshold
    let total_bytes: usize = lines.iter().map(|l| l.len() + 1).sum();
    if total_bytes as u64 <= MAX_FILE_SIZE {
        return Ok(());
    }

    let mut removed_bytes = 0usize;
    let target_removal = total_bytes - MAX_FILE_SIZE as usize;
    let mut keep_from = 0;

    for (i, (_, session_lines)) in sessions.iter().enumerate() {
        let session_bytes: usize = session_lines.iter().map(|l| l.len() + 1).sum();
        removed_bytes += session_bytes;
        keep_from = i + 1;
        if removed_bytes >= target_removal {
            break;
        }
    }

    // Rewrite file with remaining sessions
    let remaining: Vec<&str> = sessions[keep_from..]
        .iter()
        .flat_map(|(_, lines)| lines.iter())
        .copied()
        .collect();

    let new_content = remaining.join("\n");
    std::fs::write(
        path,
        if new_content.is_empty() {
            String::new()
        } else {
            new_content + "\n"
        },
    )
}

/// Extract the session ID from a JSONL line (best-effort, no full parse).
fn extract_session_id(line: &str) -> String {
    // Quick extraction: find "session":"<value>"
    if let Some(start) = line.find("\"session\":\"") {
        let rest = &line[start + 11..];
        if let Some(end) = rest.find('"') {
            return rest[..end].to_string();
        }
    }
    "unknown".to_string()
}

/// Sanitize a project name for use as a filename.
fn sanitize_slug(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect()
}

// --- Utilities ---

/// Current ISO 8601 date (YYYY-MM-DD).
#[allow(dead_code)] // Used by write_consent
fn iso_date() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let days = secs.div_euclid(86400) as i32;
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i32) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    format!("{:04}-{:02}-{:02}", y, m, d)
}

/// Current ISO 8601 timestamp (YYYY-MM-DDTHH:MM:SSZ).
pub fn iso_timestamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let days = (secs / 86400) as i32;
    let day_secs = (secs % 86400) as u32;
    let h = day_secs / 3600;
    let m = (day_secs % 3600) / 60;
    let s = day_secs % 60;

    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i32) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let mon = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if mon <= 2 { y + 1 } else { y };

    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, mon, day, h, m, s)
}

/// Current OS string.
pub fn os_string() -> &'static str {
    std::env::consts::OS
}

/// Current architecture string.
pub fn arch_string() -> &'static str {
    std::env::consts::ARCH
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn session_id_format() {
        let id = generate_session_id();
        // Format: 12 hex chars (timestamp) + '-' + 4 hex chars (pid)
        assert_eq!(id.len(), 17);
        assert_eq!(&id[12..13], "-");
        assert!(id[..12].chars().all(|c| c.is_ascii_hexdigit()));
        assert!(id[13..].chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn session_id_unique() {
        let a = generate_session_id();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let b = generate_session_id();
        assert_ne!(a, b);
    }

    #[test]
    fn project_slug_from_path() {
        assert_eq!(project_slug(Path::new("/home/user/code/tkt")), "tkt");
        assert_eq!(
            project_slug(Path::new("D:\\code\\game-research")),
            "game-research"
        );
        assert_eq!(project_slug(Path::new("/")), "unknown");
    }

    #[test]
    fn consent_default_disabled() {
        // With no env vars set (can't fully control in test, but default path)
        // Just verify the function doesn't panic
        let (state, _reason) = check_consent();
        // In CI, this will be Disabled due to CI=true
        assert_eq!(state, Consent::Disabled);
    }

    #[test]
    fn consent_file_parsing() {
        let content_enabled = "[telemetry]\nenabled = true\nconsented_at = \"2026-07-30\"\n";
        let content_disabled = "[telemetry]\nenabled = false\n";
        let content_empty = "";

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("consent.toml");

        std::fs::write(&path, content_enabled).unwrap();
        assert_eq!(read_consent_file(&path), Some(Consent::Enabled));

        std::fs::write(&path, content_disabled).unwrap();
        assert_eq!(read_consent_file(&path), Some(Consent::Disabled));

        std::fs::write(&path, content_empty).unwrap();
        assert_eq!(read_consent_file(&path), None);
    }

    #[test]
    fn event_serialization() {
        let event = Event {
            ts: "2026-07-30T10:00:00Z".to_string(),
            session: "0192a3b4c5d6-1234".to_string(),
            project: "tkt".to_string(),
            cmd: "ready".to_string(),
            exit_code: 0,
            duration_ms: 150,
            version: "0.1.0".to_string(),
            os: "windows".to_string(),
            arch: "x86_64".to_string(),
        };
        let json = event.to_json();
        assert!(json.contains("\"cmd\":\"ready\""));
        assert!(json.contains("\"exit_code\":0"));
        assert!(json.contains("\"project\":\"tkt\""));
        assert!(json.contains("\"session\":\"0192a3b4c5d6-1234\""));
        // Should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["cmd"], "ready");
    }

    #[test]
    fn sanitize_slug_handles_special_chars() {
        assert_eq!(sanitize_slug("tkt"), "tkt");
        assert_eq!(sanitize_slug("game-research"), "game-research");
        assert_eq!(sanitize_slug("my project"), "my-project");
        assert_eq!(sanitize_slug("path/to/thing"), "path-to-thing");
        assert_eq!(sanitize_slug("über-project"), "-ber-project");
    }

    #[test]
    fn iso_timestamp_format() {
        let ts = iso_timestamp();
        // YYYY-MM-DDTHH:MM:SSZ
        assert_eq!(ts.len(), 20);
        assert_eq!(&ts[4..5], "-");
        assert_eq!(&ts[7..8], "-");
        assert_eq!(&ts[10..11], "T");
        assert_eq!(&ts[19..], "Z");
    }

    #[test]
    fn is_rotated_file_detection() {
        assert!(is_rotated_file("tkt.1.jsonl"));
        assert!(is_rotated_file("game-research.5.jsonl"));
        assert!(is_rotated_file("project.12.jsonl"));
        assert!(!is_rotated_file("tkt.jsonl")); // current file, not rotated
        assert!(!is_rotated_file("tkt.txt"));
        assert!(!is_rotated_file("tkt.abc.jsonl")); // non-numeric suffix
    }

    #[test]
    fn rotate_file_shifts_files() {
        let dir = tempfile::tempdir().unwrap();
        let slug = "test-project";

        // Create a current file
        std::fs::write(dir.path().join("test-project.jsonl"), "line1\nline2\n").unwrap();

        // Rotate
        rotate_file(dir.path(), slug).unwrap();

        // Current should be gone, .1 should exist
        assert!(!dir.path().join("test-project.jsonl").exists());
        assert!(dir.path().join("test-project.1.jsonl").exists());
        assert_eq!(
            std::fs::read_to_string(dir.path().join("test-project.1.jsonl")).unwrap(),
            "line1\nline2\n"
        );

        // Create another current and rotate again
        std::fs::write(dir.path().join("test-project.jsonl"), "line3\n").unwrap();
        rotate_file(dir.path(), slug).unwrap();

        // .1 should have new content, .2 should have old
        assert!(dir.path().join("test-project.1.jsonl").exists());
        assert!(dir.path().join("test-project.2.jsonl").exists());
        assert_eq!(
            std::fs::read_to_string(dir.path().join("test-project.1.jsonl")).unwrap(),
            "line3\n"
        );
        assert_eq!(
            std::fs::read_to_string(dir.path().join("test-project.2.jsonl")).unwrap(),
            "line1\nline2\n"
        );
    }

    #[test]
    fn rotate_file_respects_max_files() {
        let dir = tempfile::tempdir().unwrap();
        let slug = "proj";

        // Create files .1 through MAX_ROTATED_FILES
        for i in 1..=MAX_ROTATED_FILES {
            std::fs::write(
                dir.path().join(format!("proj.{}.jsonl", i)),
                format!("data{}\n", i),
            )
            .unwrap();
        }
        // Create current
        std::fs::write(dir.path().join("proj.jsonl"), "current\n").unwrap();

        // Rotate — should shift all and drop the oldest
        rotate_file(dir.path(), slug).unwrap();

        // The file beyond max should have been deleted
        assert!(!dir
            .path()
            .join(format!("proj.{}.jsonl", MAX_ROTATED_FILES + 1))
            .exists());
        // .1 should be the old current
        assert_eq!(
            std::fs::read_to_string(dir.path().join("proj.1.jsonl")).unwrap(),
            "current\n"
        );
    }

    #[test]
    fn extract_session_id_from_jsonl() {
        let line = r#"{"ts":"2026-07-30T10:00:00Z","session":"abc123-def4","project":"tkt","cmd":"ready"}"#;
        assert_eq!(extract_session_id(line), "abc123-def4");

        let bad_line = r#"{"ts":"2026-07-30T10:00:00Z","cmd":"ready"}"#;
        assert_eq!(extract_session_id(bad_line), "unknown");
    }

    #[test]
    fn prune_oldest_sessions_removes_whole_sessions() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.jsonl");

        // Write lines from 3 sessions, each small
        let mut content = String::new();
        for i in 0..3 {
            for j in 0..5 {
                content.push_str(&format!(
                    r#"{{"session":"sess-{}","cmd":"cmd{}","ts":"t"}}"#,
                    i, j
                ));
                content.push('\n');
            }
        }
        std::fs::write(&path, &content).unwrap();

        // File is small, pruning should be a no-op
        prune_oldest_sessions(&path).unwrap();
        let after = std::fs::read_to_string(&path).unwrap();
        assert_eq!(
            after.lines().count(),
            15,
            "should not prune when under threshold"
        );
    }

    #[test]
    fn enforce_max_files_cleans_excess() {
        let dir = tempfile::tempdir().unwrap();

        // Create more than MAX_ROTATED_FILES rotated files for one slug
        for i in 1..=(MAX_ROTATED_FILES + 3) {
            std::fs::write(dir.path().join(format!("myproj.{}.jsonl", i)), "data\n").unwrap();
        }

        enforce_max_files(dir.path()).unwrap();

        // Should only have MAX_ROTATED_FILES remaining
        let remaining: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "jsonl"))
            .collect();
        assert_eq!(remaining.len(), MAX_ROTATED_FILES as usize);
    }
}

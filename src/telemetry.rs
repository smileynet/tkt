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

/// Path to the telemetry data directory.
/// Linux: ~/.local/share/tkt/telemetry/
/// macOS: ~/Library/Application Support/tkt/telemetry/
/// Windows: %APPDATA%/tkt/telemetry/
pub fn telemetry_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("tkt").join("telemetry"))
}

/// Append an event to the per-project JSONL file.
/// Silently swallows all errors (telemetry must never affect CLI behavior).
pub fn record_event(event: &Event) {
    let _ = try_record_event(event);
}

fn try_record_event(event: &Event) -> std::io::Result<()> {
    let dir = telemetry_dir()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "no data directory"))?;
    std::fs::create_dir_all(&dir)?;

    let filename = format!("{}.jsonl", sanitize_slug(&event.project));
    let path = dir.join(filename);

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;

    let line = event.to_json();
    writeln!(file, "{}", line)?;
    Ok(())
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
}

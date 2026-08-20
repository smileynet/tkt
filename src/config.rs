//! Unified configuration with cascade: CLI flag > env var > project > user > default.
//!
//! User config: ~/.config/tkt/config.toml
//! Project config: .tickets/config.toml
//!
//! Both files use the same [section] key = value format.
//! The user config is created on first `tkt config --set`, not on install.
//! The project config is optional — missing file means all defaults.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// All known configuration keys and their built-in defaults.
const PROJECT_KEYS: &[(&str, &str)] = &[
    ("close.require_resolution", "false"),
    ("close.require_checked_acs", "true"),
    ("close.require_validation_criteria", "false"),
    ("close.require_validation_evidence", "warn"),
    ("close.allow_force", "true"),
    ("validate.strict", "false"),
    ("ready.default_env", ""),
    ("priority.warn_unknown", "true"),
    ("new.default_priority", ""),
    ("push.enabled", "true"),
];

/// User-only keys (debug settings, not applicable at project level).
const USER_ONLY_KEYS: &[(&str, &str)] = &[("debug", "false"), ("debug.format", "human")];

// ============================================================
// Project-level configuration (.tickets/config.toml)
// ============================================================

/// Project-level configuration with all fields defaulted.
#[derive(Debug, Clone)]
pub struct ProjectConfig {
    pub close_require_resolution: bool,
    pub close_require_checked_acs: bool,
    pub close_require_validation_criteria: bool,
    /// "false" | "warn" | "true"
    pub close_require_validation_evidence: String,
    pub close_allow_force: bool,
    pub validate_strict: bool,
    pub ready_default_env: String,
    pub priority_warn_unknown: bool,
    pub new_default_priority: String,
    pub push_enabled: bool,
    /// Unknown keys found in the config file (for warning).
    pub unknown_keys: Vec<String>,
    /// Track which keys came from which source for --show.
    sources: BTreeMap<String, Source>,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            close_require_resolution: false,
            close_require_checked_acs: true,
            close_require_validation_criteria: false,
            close_require_validation_evidence: "warn".to_string(),
            close_allow_force: true,
            validate_strict: false,
            ready_default_env: String::new(),
            priority_warn_unknown: true,
            new_default_priority: String::new(),
            push_enabled: true,
            unknown_keys: Vec::new(),
            sources: BTreeMap::new(),
        }
    }
}

impl ProjectConfig {
    /// Load project config with cascade: project config > user config > default.
    /// Missing files are not errors — they just don't contribute values.
    /// Set TKT_NO_USER_CONFIG=1 to skip user config (for testing).
    pub fn load(tickets_dir: &Path) -> Self {
        let project_path = tickets_dir.join("config.toml");
        let project_values = read_sectioned_config(&project_path);
        let skip_user = std::env::var("TKT_NO_USER_CONFIG").as_deref() == Ok("1");
        let user_values = if skip_user {
            None
        } else {
            config_file_path().and_then(|p| read_sectioned_config(&p))
        };

        let mut cfg = Self::default();
        let mut unknown = Vec::new();
        let mut sources = BTreeMap::new();

        // Initialize all sources as default
        for &(key, _) in PROJECT_KEYS {
            sources.insert(key.to_string(), Source::Default);
        }

        // Layer 1: user config (lowest priority)
        if let Some(ref user_vals) = user_values {
            for (key, value) in user_vals {
                if apply_value(&mut cfg, key, value) {
                    sources.insert(key.clone(), Source::User);
                }
            }
        }

        // Layer 2: project config (overrides user)
        if let Some(ref proj_vals) = project_values {
            for (key, value) in proj_vals {
                if apply_value(&mut cfg, key, value) {
                    sources.insert(key.clone(), Source::ProjectConfig);
                } else {
                    unknown.push(key.clone());
                }
            }
        }

        // Layer 3: env vars (overrides project)
        for &(key, _) in PROJECT_KEYS {
            let env_name = format!("TKT_{}", key.replace('.', "_").to_uppercase());
            if let Ok(val) = std::env::var(&env_name) {
                apply_value(&mut cfg, key, &val);
                sources.insert(key.to_string(), Source::Env);
            }
        }

        cfg.unknown_keys = unknown;
        cfg.sources = sources;
        cfg
    }

    /// List all project settings with their resolved sources.
    pub fn list(&self) -> Vec<ConfigEntry> {
        let mut entries = Vec::new();
        let fields: Vec<(&str, String)> = vec![
            (
                "close.require_resolution",
                self.close_require_resolution.to_string(),
            ),
            (
                "close.require_checked_acs",
                self.close_require_checked_acs.to_string(),
            ),
            (
                "close.require_validation_criteria",
                self.close_require_validation_criteria.to_string(),
            ),
            (
                "close.require_validation_evidence",
                self.close_require_validation_evidence.clone(),
            ),
            ("close.allow_force", self.close_allow_force.to_string()),
            ("validate.strict", self.validate_strict.to_string()),
            ("ready.default_env", self.ready_default_env.clone()),
            (
                "priority.warn_unknown",
                self.priority_warn_unknown.to_string(),
            ),
            ("new.default_priority", self.new_default_priority.clone()),
            ("push.enabled", self.push_enabled.to_string()),
        ];

        for (key, value) in fields {
            let source = self.sources.get(key).cloned().unwrap_or(Source::Default);
            entries.push(ConfigEntry {
                key: key.to_string(),
                value,
                source,
            });
        }
        entries
    }
}

/// Apply a key-value pair to the config struct. Returns true if the key was recognized.
fn apply_value(cfg: &mut ProjectConfig, key: &str, value: &str) -> bool {
    match key {
        "close.require_resolution" => cfg.close_require_resolution = is_truthy(value),
        "close.require_checked_acs" => cfg.close_require_checked_acs = is_truthy(value),
        "close.require_validation_criteria" => {
            cfg.close_require_validation_criteria = is_truthy(value)
        }
        "close.require_validation_evidence" => {
            cfg.close_require_validation_evidence = value.to_string()
        }
        "close.allow_force" => cfg.close_allow_force = is_truthy(value),
        "validate.strict" => cfg.validate_strict = is_truthy(value),
        "ready.default_env" => cfg.ready_default_env = value.to_string(),
        "priority.warn_unknown" => cfg.priority_warn_unknown = is_truthy(value),
        "new.default_priority" => cfg.new_default_priority = value.to_string(),
        "push.enabled" => cfg.push_enabled = is_truthy(value),
        _ => return false,
    }
    true
}

fn is_truthy(s: &str) -> bool {
    matches!(s, "true" | "1" | "yes")
}

/// Parse a TOML-like config with [sections].
/// Converts `[section]\nkey = value` to `section.key = value` in the map.
fn read_sectioned_config(path: &Path) -> Option<BTreeMap<String, String>> {
    let content = std::fs::read_to_string(path).ok()?;
    let mut map = BTreeMap::new();
    let mut current_section = String::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            current_section = trimmed[1..trimmed.len() - 1].trim().to_string();
            continue;
        }
        if let Some((key, value)) = trimmed.split_once('=') {
            let key = key.trim();
            let value = value.trim().trim_matches('"');
            let full_key = if current_section.is_empty() {
                key.to_string()
            } else {
                format!("{}.{}", current_section, key)
            };
            map.insert(full_key, value.to_string());
        }
    }
    Some(map)
}

// ============================================================
// User-level configuration (~/.config/tkt/config.toml)
// ============================================================

/// User configuration state (debug settings + shared project defaults).
#[derive(Debug)]
pub struct Config {
    values: BTreeMap<String, String>,
}

impl Config {
    /// Load config from the platform-appropriate path.
    /// Missing file is not an error (returns all defaults).
    /// Respects TKT_NO_USER_CONFIG=1 to skip loading (for testing).
    pub fn load() -> Self {
        if std::env::var("TKT_NO_USER_CONFIG").as_deref() == Ok("1") {
            return Config {
                values: BTreeMap::new(),
            };
        }
        let values = config_file_path()
            .and_then(|p| read_sectioned_config(&p))
            .unwrap_or_default();
        Config { values }
    }

    /// Get a value with precedence: env > config > default.
    /// Env var name: TKT_{KEY_UPPER} (dots become underscores).
    pub fn get(&self, key: &str) -> String {
        // Check env override
        let env_name = format!("TKT_{}", key.replace('.', "_").to_uppercase());
        if let Ok(val) = std::env::var(&env_name) {
            return val;
        }
        // Check config file
        if let Some(val) = self.values.get(key) {
            return val.clone();
        }
        // Default
        default_for(key).to_string()
    }

    /// Get a value as bool (true/1/yes are truthy).
    pub fn get_bool(&self, key: &str) -> bool {
        matches!(self.get(key).as_str(), "true" | "1" | "yes")
    }

    /// Set a key in the user config file. Creates the file if it doesn't exist.
    pub fn set(key: &str, value: &str) -> std::io::Result<()> {
        let path = config_file_path().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "no config directory")
        })?;

        let mut values = read_sectioned_config(&path).unwrap_or_default();
        values.insert(key.to_string(), value.to_string());
        write_config_file(&path, &values)
    }

    /// Remove a key from the user config file (revert to default).
    pub fn unset(key: &str) -> std::io::Result<bool> {
        let path = config_file_path().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "no config directory")
        })?;

        let mut values = read_sectioned_config(&path).unwrap_or_default();
        let existed = values.remove(key).is_some();
        write_config_file(&path, &values)?;
        Ok(existed)
    }

    /// List all user settings: known keys with their effective values and sources.
    pub fn list(&self) -> Vec<ConfigEntry> {
        let mut entries = Vec::new();

        // User-only keys (debug)
        for &(key, default) in USER_ONLY_KEYS {
            let env_name = format!("TKT_{}", key.replace('.', "_").to_uppercase());
            let (value, source) = if let Ok(val) = std::env::var(&env_name) {
                (val, Source::Env)
            } else if let Some(val) = self.values.get(key) {
                (val.clone(), Source::ConfigFile)
            } else {
                (default.to_string(), Source::Default)
            };
            entries.push(ConfigEntry {
                key: key.to_string(),
                value,
                source,
            });
        }

        // Project keys that have user-level overrides
        for &(key, _default) in PROJECT_KEYS {
            if let Some(val) = self.values.get(key) {
                entries.push(ConfigEntry {
                    key: key.to_string(),
                    value: val.clone(),
                    source: Source::ConfigFile,
                });
            } else {
                // Only show if env var set
                let env_name = format!("TKT_{}", key.replace('.', "_").to_uppercase());
                if let Ok(val) = std::env::var(&env_name) {
                    entries.push(ConfigEntry {
                        key: key.to_string(),
                        value: val,
                        source: Source::Env,
                    });
                }
            }
        }

        // Include any extra keys in config file not in known lists
        for (key, val) in &self.values {
            if !USER_ONLY_KEYS.iter().any(|(k, _)| k == key)
                && !PROJECT_KEYS.iter().any(|(k, _)| k == key)
            {
                entries.push(ConfigEntry {
                    key: key.clone(),
                    value: val.clone(),
                    source: Source::ConfigFile,
                });
            }
        }
        entries
    }
}

#[derive(Debug, Clone)]
pub struct ConfigEntry {
    pub key: String,
    pub value: String,
    pub source: Source,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    Env,
    ConfigFile,
    User,
    ProjectConfig,
    Default,
}

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Source::Env => write!(f, "env"),
            Source::ConfigFile => write!(f, "config"),
            Source::User => write!(f, "user"),
            Source::ProjectConfig => write!(f, "project"),
            Source::Default => write!(f, "default"),
        }
    }
}

/// Platform-appropriate config file path.
/// Respects XDG_CONFIG_HOME if set (for testing and Linux convention).
/// Otherwise: macOS ~/Library/Application Support/tkt/config.toml,
/// Linux ~/.config/tkt/config.toml, Windows %APPDATA%/tkt/config.toml.
pub fn config_file_path() -> Option<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(xdg).join("tkt").join("config.toml"));
    }
    dirs::config_dir().map(|d| d.join("tkt").join("config.toml"))
}

/// Get the default value for a key (user-only or project keys).
fn default_for(key: &str) -> &str {
    USER_ONLY_KEYS
        .iter()
        .chain(PROJECT_KEYS.iter())
        .find(|(k, _)| *k == key)
        .map(|(_, v)| *v)
        .unwrap_or("")
}

/// Write config file with sections. Groups keys by prefix (before the dot).
fn write_config_file(path: &Path, values: &BTreeMap<String, String>) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut content = String::from(
        "# tkt user configuration\n# Cascade: env > project config > this file > default\n\n",
    );

    // Group by section
    let mut sections: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    for (key, value) in values {
        if let Some((section, field)) = key.split_once('.') {
            sections
                .entry(section.to_string())
                .or_default()
                .push((field.to_string(), value.clone()));
        } else {
            sections
                .entry(String::new())
                .or_default()
                .push((key.clone(), value.clone()));
        }
    }

    // Write unsectioned keys first
    if let Some(flat_keys) = sections.remove("") {
        for (key, value) in &flat_keys {
            content.push_str(&format!("{} = \"{}\"\n", key, value));
        }
        if !flat_keys.is_empty() {
            content.push('\n');
        }
    }

    // Write sectioned keys
    for (section, fields) in &sections {
        content.push_str(&format!("[{}]\n", section));
        for (field, value) in fields {
            content.push_str(&format!("{} = \"{}\"\n", field, value));
        }
        content.push('\n');
    }

    std::fs::write(path, content)
}

/// Set a key in the project config file.
#[allow(dead_code)]
pub fn set_project_config(tickets_dir: &Path, key: &str, value: &str) -> std::io::Result<()> {
    let path = tickets_dir.join("config.toml");
    let mut values = read_sectioned_config(&path).unwrap_or_default();
    values.insert(key.to_string(), value.to_string());
    write_config_file(&path, &values)
}

/// Remove a key from the project config file.
#[allow(dead_code)]
pub fn unset_project_config(tickets_dir: &Path, key: &str) -> std::io::Result<bool> {
    let path = tickets_dir.join("config.toml");
    let mut values = read_sectioned_config(&path).unwrap_or_default();
    let existed = values.remove(key).is_some();
    write_config_file(&path, &values)?;
    Ok(existed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_read_config_file_parses_values() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "# comment").unwrap();
        writeln!(f, "debug = \"true\"").unwrap();
        writeln!(f, "debug.format = \"json\"").unwrap();

        let map = read_sectioned_config(&path).unwrap();
        assert_eq!(map.get("debug").unwrap(), "true");
        assert_eq!(map.get("debug.format").unwrap(), "json");
    }

    #[test]
    fn test_read_config_file_missing_returns_none() {
        let path = Path::new("/nonexistent/config.toml");
        assert!(read_sectioned_config(path).is_none());
    }

    #[test]
    fn test_write_then_read_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tkt").join("config.toml");

        let mut values = BTreeMap::new();
        values.insert("debug".to_string(), "true".to_string());
        values.insert("debug.format".to_string(), "json".to_string());

        write_config_file(&path, &values).unwrap();
        let read_back = read_sectioned_config(&path).unwrap();
        assert_eq!(read_back, values);
    }

    #[test]
    fn test_default_for_known_keys() {
        assert_eq!(default_for("debug"), "false");
        assert_eq!(default_for("debug.format"), "human");
        assert_eq!(default_for("close.allow_force"), "true");
        assert_eq!(default_for("unknown"), "");
    }

    #[test]
    fn test_project_config_defaults() {
        // Prevent ambient user config from affecting defaults
        std::env::set_var("TKT_NO_USER_CONFIG", "1");
        let dir = tempfile::tempdir().unwrap();
        let cfg = ProjectConfig::load(dir.path());
        assert!(!cfg.close_require_resolution);
        assert!(cfg.close_require_checked_acs);
        assert!(!cfg.validate_strict);
        assert!(cfg.ready_default_env.is_empty());
        assert!(cfg.priority_warn_unknown);
        assert!(cfg.new_default_priority.is_empty());
        assert!(cfg.push_enabled);
        assert!(cfg.unknown_keys.is_empty());
        std::env::remove_var("TKT_NO_USER_CONFIG");
    }

    #[test]
    fn test_project_config_parses_sections() {
        let dir = tempfile::tempdir().unwrap();
        let config_content = r#"
[close]
require_resolution = true

[push]
enabled = false

[ready]
default_env = "corp"

[mystery]
foo = "bar"
"#;
        std::fs::write(dir.path().join("config.toml"), config_content).unwrap();
        let cfg = ProjectConfig::load(dir.path());
        assert!(cfg.close_require_resolution);
        assert!(!cfg.push_enabled);
        assert_eq!(cfg.ready_default_env, "corp");
        assert_eq!(cfg.unknown_keys, vec!["mystery.foo"]);
    }

    #[test]
    fn test_user_config_cascades_to_project() {
        // User config sets allow_force = false
        let user_dir = tempfile::tempdir().unwrap();
        let user_path = user_dir.path().join("config.toml");
        std::fs::write(
            &user_path,
            "[close]\nallow_force = false\nrequire_resolution = true\n",
        )
        .unwrap();

        // Project config only overrides require_resolution
        let proj_dir = tempfile::tempdir().unwrap();
        std::fs::write(
            proj_dir.path().join("config.toml"),
            "[close]\nrequire_resolution = false\n",
        )
        .unwrap();

        // Load with user config — we need to test the cascade logic directly
        let user_vals = read_sectioned_config(&user_path).unwrap();
        let proj_vals = read_sectioned_config(&proj_dir.path().join("config.toml")).unwrap();

        let mut cfg = ProjectConfig::default();
        // Apply user first
        for (key, value) in &user_vals {
            apply_value(&mut cfg, key, value);
        }
        // Then project overrides
        for (key, value) in &proj_vals {
            apply_value(&mut cfg, key, value);
        }

        // allow_force from user (not overridden by project)
        assert!(!cfg.close_allow_force);
        // require_resolution overridden by project (false beats user's true)
        assert!(!cfg.close_require_resolution);
    }

    #[test]
    fn test_write_sectioned_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");

        let mut values = BTreeMap::new();
        values.insert("close.allow_force".to_string(), "false".to_string());
        values.insert("close.require_resolution".to_string(), "true".to_string());
        values.insert("push.enabled".to_string(), "false".to_string());

        write_config_file(&path, &values).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("[close]"));
        assert!(content.contains("allow_force = \"false\""));
        assert!(content.contains("[push]"));

        // Roundtrip
        let read_back = read_sectioned_config(&path).unwrap();
        assert_eq!(read_back, values);
    }
}

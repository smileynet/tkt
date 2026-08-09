//! User-level and project-level configuration.
//!
//! User config: ~/.config/tkt/config.toml (precedence: env > config > default)
//! Project config: .tickets/config.toml (precedence: CLI flag > config > default)
//!
//! The user config is created on first `tkt config set`, not on install.
//! The project config is optional — missing file means all defaults.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Known user configuration keys and their defaults.
const KNOWN_KEYS: &[(&str, &str)] = &[("debug", "false"), ("debug.format", "human")];

// ============================================================
// Project-level configuration (.tickets/config.toml)
// ============================================================

/// Project-level configuration with all fields defaulted.
#[derive(Debug, Clone)]
pub struct ProjectConfig {
    pub close_require_resolution: bool,
    pub close_require_checked_acs: bool,
    pub validate_strict: bool,
    pub ready_default_env: String,
    pub priority_warn_unknown: bool,
    pub new_default_priority: String,
    pub push_enabled: bool,
    /// Unknown keys found in the config file (for warning).
    pub unknown_keys: Vec<String>,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            close_require_resolution: false,
            close_require_checked_acs: true,
            validate_strict: false,
            ready_default_env: String::new(),
            priority_warn_unknown: true,
            new_default_priority: String::new(),
            push_enabled: true,
            unknown_keys: Vec::new(),
        }
    }
}

impl ProjectConfig {
    /// Load project config from `.tickets/config.toml` relative to the given tickets dir.
    /// Missing file returns all defaults. Unknown keys are collected for warnings.
    pub fn load(tickets_dir: &Path) -> Self {
        let path = tickets_dir.join("config.toml");
        let values = match read_project_config(&path) {
            Some(v) => v,
            None => return Self::default(),
        };

        let mut cfg = Self::default();
        let mut unknown = Vec::new();

        for (key, value) in &values {
            match key.as_str() {
                "close.require_resolution" => cfg.close_require_resolution = is_truthy(value),
                "close.require_checked_acs" => cfg.close_require_checked_acs = is_truthy(value),
                "validate.strict" => cfg.validate_strict = is_truthy(value),
                "ready.default_env" => cfg.ready_default_env = value.clone(),
                "priority.warn_unknown" => cfg.priority_warn_unknown = is_truthy(value),
                "new.default_priority" => cfg.new_default_priority = value.clone(),
                "push.enabled" => cfg.push_enabled = is_truthy(value),
                _ => unknown.push(key.clone()),
            }
        }
        cfg.unknown_keys = unknown;
        cfg
    }

    /// List all project settings with their sources.
    pub fn list(&self) -> Vec<ConfigEntry> {
        vec![
            entry(
                "close.require_resolution",
                &self.close_require_resolution.to_string(),
                "false",
            ),
            entry(
                "close.require_checked_acs",
                &self.close_require_checked_acs.to_string(),
                "true",
            ),
            entry(
                "validate.strict",
                &self.validate_strict.to_string(),
                "false",
            ),
            entry("ready.default_env", &self.ready_default_env, ""),
            entry(
                "priority.warn_unknown",
                &self.priority_warn_unknown.to_string(),
                "true",
            ),
            entry("new.default_priority", &self.new_default_priority, ""),
            entry("push.enabled", &self.push_enabled.to_string(), "true"),
        ]
    }
}

fn entry(key: &str, value: &str, default: &str) -> ConfigEntry {
    let source = if value == default {
        Source::Default
    } else {
        Source::ProjectConfig
    };
    ConfigEntry {
        key: key.to_string(),
        value: value.to_string(),
        source,
    }
}

fn is_truthy(s: &str) -> bool {
    matches!(s, "true" | "1" | "yes")
}

/// Parse a TOML-like project config with [sections].
/// Converts `[section]\nkey = value` to `section.key = value` in the map.
fn read_project_config(path: &Path) -> Option<BTreeMap<String, String>> {
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

/// Resolved configuration state.
#[derive(Debug)]
pub struct Config {
    values: BTreeMap<String, String>,
}

impl Config {
    /// Load config from the platform-appropriate path.
    /// Missing file is not an error (returns all defaults).
    pub fn load() -> Self {
        let values = config_file_path()
            .and_then(|p| read_config_file(&p))
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

    /// Set a key in the config file. Creates the file if it doesn't exist.
    pub fn set(key: &str, value: &str) -> std::io::Result<()> {
        let path = config_file_path().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "no config directory")
        })?;

        let mut values = read_config_file(&path).unwrap_or_default();
        values.insert(key.to_string(), value.to_string());
        write_config_file(&path, &values)
    }

    /// Remove a key from the config file (revert to default).
    pub fn unset(key: &str) -> std::io::Result<bool> {
        let path = config_file_path().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "no config directory")
        })?;

        let mut values = read_config_file(&path).unwrap_or_default();
        let existed = values.remove(key).is_some();
        write_config_file(&path, &values)?;
        Ok(existed)
    }

    /// List all settings: known keys with their effective values and sources.
    pub fn list(&self) -> Vec<ConfigEntry> {
        let mut entries = Vec::new();
        for &(key, default) in KNOWN_KEYS {
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
        // Include any extra keys in config file not in KNOWN_KEYS
        for (key, val) in &self.values {
            if !KNOWN_KEYS.iter().any(|(k, _)| k == key) {
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

#[derive(Debug)]
pub struct ConfigEntry {
    pub key: String,
    pub value: String,
    pub source: Source,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Source {
    Env,
    ConfigFile,
    ProjectConfig,
    Default,
}

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Source::Env => write!(f, "env"),
            Source::ConfigFile => write!(f, "config"),
            Source::ProjectConfig => write!(f, "project"),
            Source::Default => write!(f, "default"),
        }
    }
}

/// Platform-appropriate config file path.
/// Linux: ~/.config/tkt/config.toml
/// macOS: ~/Library/Application Support/tkt/config.toml
/// Windows: %APPDATA%/tkt/config.toml
pub fn config_file_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("tkt").join("config.toml"))
}

/// Get the default value for a key.
fn default_for(key: &str) -> &str {
    KNOWN_KEYS
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| *v)
        .unwrap_or("")
}

/// Parse a simple TOML-like config (flat key = value pairs, supports # comments).
fn read_config_file(path: &Path) -> Option<BTreeMap<String, String>> {
    let content = std::fs::read_to_string(path).ok()?;
    let mut map = BTreeMap::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = trimmed.split_once('=') {
            let key = key.trim().to_string();
            let value = value.trim().trim_matches('"').to_string();
            map.insert(key, value);
        }
    }
    Some(map)
}

/// Write config file (flat key = "value" format).
fn write_config_file(path: &Path, values: &BTreeMap<String, String>) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut content = String::from("# tkt user configuration\n# See: tkt config list\n\n");
    for (key, value) in values {
        content.push_str(&format!("{} = \"{}\"\n", key, value));
    }
    std::fs::write(path, content)
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

        let map = read_config_file(&path).unwrap();
        assert_eq!(map.get("debug").unwrap(), "true");
        assert_eq!(map.get("debug.format").unwrap(), "json");
    }

    #[test]
    fn test_read_config_file_missing_returns_none() {
        let path = Path::new("/nonexistent/config.toml");
        assert!(read_config_file(path).is_none());
    }

    #[test]
    fn test_write_then_read_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tkt").join("config.toml");

        let mut values = BTreeMap::new();
        values.insert("debug".to_string(), "true".to_string());
        values.insert("debug.format".to_string(), "json".to_string());

        write_config_file(&path, &values).unwrap();
        let read_back = read_config_file(&path).unwrap();
        assert_eq!(read_back, values);
    }

    #[test]
    fn test_default_for_known_keys() {
        assert_eq!(default_for("debug"), "false");
        assert_eq!(default_for("debug.format"), "human");
        assert_eq!(default_for("unknown"), "");
    }

    #[test]
    fn test_project_config_defaults() {
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
}

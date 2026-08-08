//! User-level configuration (~/.config/tkt/config.toml).
//!
//! Precedence: env var > config file > default.
//! The config file is created on first `tkt config set`, not on install.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Known configuration keys and their defaults.
const KNOWN_KEYS: &[(&str, &str)] = &[("debug", "false"), ("debug.format", "human")];

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
    Default,
}

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Source::Env => write!(f, "env"),
            Source::ConfigFile => write!(f, "config"),
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
}

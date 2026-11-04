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
    ("validate.require_source_reference", "false"),
    ("validate.missing_resolution_before_id", ""),
    ("ready.default_env", ""),
    ("priority.warn_unknown", "true"),
    ("new.default_priority", ""),
    ("push.enabled", "true"),
    ("commit.hooks", "skip"),
    ("hooks.on_new", "[]"),
    ("hooks.on_claim", "[]"),
    ("hooks.on_close", "[]"),
    ("machine.capabilities", ""),
    ("audit.resolution_baseline_id", ""),
];

/// User-only keys (debug settings, not applicable at project level).
const USER_ONLY_KEYS: &[(&str, &str)] = &[
    ("debug", "false"),
    ("debug.format", "log"),
    ("update_check.interval", "24h"),
];

// ============================================================
// Project-level configuration (.tickets/config.toml)
// ============================================================

/// Project-level configuration with all fields defaulted.
#[derive(Debug, Clone)]
pub struct ProjectConfig {
    pub close_require_resolution: bool,
    pub close_require_checked_acs: bool,
    pub close_require_validation_criteria: bool,
    pub close_require_validation_evidence: EvidencePolicy,
    pub close_allow_force: bool,
    pub validate_strict: bool,
    pub validate_require_source_reference: bool,
    /// Suppress missing-resolution findings for numeric IDs below this project ratchet.
    pub validate_missing_resolution_before_id: Option<u64>,
    pub ready_default_env: Option<crate::core::Env>,
    pub priority_warn_unknown: bool,
    pub new_default_priority: Option<crate::core::Priority>,
    pub push_enabled: bool,
    pub commit_hooks: CommitHookPolicy,
    pub lifecycle_hooks: [Vec<String>; 3],
    /// Machine capabilities declared by this workstation (for requires matching).
    pub machine_capabilities: Vec<String>,
    /// Grandfather cutoff for audit resolution-quality rules: done tickets with a
    /// numeric ID below this value are exempt from resolution-quality audit rules.
    pub audit_resolution_baseline_id: Option<u64>,
    pub label_namespaces: BTreeMap<String, LabelNamespace>,
    pub label_namespace_errors: Vec<String>,
    pub config_errors: Vec<String>,
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
            close_require_validation_evidence: EvidencePolicy::Warn,
            close_allow_force: true,
            validate_strict: false,
            validate_require_source_reference: false,
            validate_missing_resolution_before_id: None,
            ready_default_env: None,
            priority_warn_unknown: true,
            new_default_priority: None,
            push_enabled: true,
            commit_hooks: CommitHookPolicy::Skip,
            lifecycle_hooks: Default::default(),
            machine_capabilities: Vec::new(),
            audit_resolution_baseline_id: None,
            label_namespaces: BTreeMap::new(),
            label_namespace_errors: Vec::new(),
            config_errors: Vec::new(),
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
        let skip_user = std::env::var("TKT_NO_USER_CONFIG").as_deref() == Ok("1");
        Self::load_inner(tickets_dir, skip_user)
    }

    /// `load` with the user-config decision passed in, so tests needn't mutate the
    /// process-global environment (which races across parallel tests).
    fn load_inner(tickets_dir: &Path, skip_user: bool) -> Self {
        let project_path = tickets_dir.join("config.toml");
        let project_values = read_sectioned_config(&project_path);
        let (label_namespaces, label_namespace_errors) = project_values
            .as_ref()
            .map(parse_label_namespaces)
            .unwrap_or_default();
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

        if let Some(ref proj_vals) = project_values {
            for key in proj_vals.keys() {
                if !key.starts_with("labels.")
                    && !PROJECT_KEYS.iter().any(|(known, _)| known == key)
                {
                    unknown.push(key.clone());
                }
            }
        }

        for &(key, _) in PROJECT_KEYS {
            let env_name = format!("TKT_{}", key.replace('.', "_").to_uppercase());
            let (value, source) = if let Ok(value) = std::env::var(&env_name) {
                (Some(value), Source::Env)
            } else if let Some(value) = project_values.as_ref().and_then(|values| values.get(key)) {
                (Some(value.clone()), Source::ProjectConfig)
            } else if let Some(value) = user_values.as_ref().and_then(|values| values.get(key)) {
                (Some(value.clone()), Source::User)
            } else {
                (None, Source::Default)
            };
            if let Some(value) = value {
                match apply_value(&mut cfg, key, &value) {
                    Ok(true) => {
                        sources.insert(key.to_string(), source);
                    }
                    Ok(false) => unreachable!("PROJECT_KEYS contains only known policy keys"),
                    Err(reason) => cfg.config_errors.push(config_value_error(
                        key,
                        &value,
                        &config_source_description(&source, key, &project_path),
                        &reason,
                    )),
                }
            }
        }

        if let Some(ref user_vals) = user_values {
            if std::env::var_os("TKT_DEBUG").is_none() {
                cfg.config_errors
                    .extend(validate_user_debug_values(user_vals));
            }
        }

        cfg.unknown_keys = unknown;
        cfg.label_namespaces = label_namespaces;
        cfg.label_namespace_errors = label_namespace_errors;
        cfg.sources = sources;
        cfg
    }

    pub fn ready_capabilities(&self) -> Vec<String> {
        let legacy_env = std::env::var("CREW_ENV")
            .ok()
            .filter(|v| !v.is_empty())
            .or_else(|| self.ready_default_env.map(|v| v.as_str().to_string()))
            .unwrap_or_default();
        let mut capabilities = self.machine_capabilities.clone();
        if !legacy_env.is_empty() && !capabilities.contains(&legacy_env) {
            capabilities.push(legacy_env);
        }
        capabilities
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
                self.close_require_validation_evidence.as_str().to_string(),
            ),
            ("close.allow_force", self.close_allow_force.to_string()),
            ("validate.strict", self.validate_strict.to_string()),
            (
                "validate.require_source_reference",
                self.validate_require_source_reference.to_string(),
            ),
            (
                "validate.missing_resolution_before_id",
                self.validate_missing_resolution_before_id
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
            ),
            (
                "ready.default_env",
                self.ready_default_env
                    .map(|value| value.as_str().to_string())
                    .unwrap_or_default(),
            ),
            (
                "priority.warn_unknown",
                self.priority_warn_unknown.to_string(),
            ),
            (
                "new.default_priority",
                self.new_default_priority
                    .map(|value| value.as_str().to_string())
                    .unwrap_or_default(),
            ),
            ("push.enabled", self.push_enabled.to_string()),
            ("commit.hooks", self.commit_hooks.as_str().to_string()),
            (
                "hooks.on_new",
                serde_json::to_string(&self.lifecycle_hooks[0]).unwrap(),
            ),
            (
                "hooks.on_claim",
                serde_json::to_string(&self.lifecycle_hooks[1]).unwrap(),
            ),
            (
                "hooks.on_close",
                serde_json::to_string(&self.lifecycle_hooks[2]).unwrap(),
            ),
            ("machine.capabilities", self.machine_capabilities.join(",")),
            (
                "audit.resolution_baseline_id",
                self.audit_resolution_baseline_id
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
            ),
        ];

        for (key, value) in fields {
            let source = self.sources.get(key).cloned().unwrap_or(Source::Default);
            entries.push(ConfigEntry {
                key: key.to_string(),
                value,
                source,
            });
        }
        for (name, namespace) in &self.label_namespaces {
            entries.push(ConfigEntry {
                key: format!("labels.{name}.single_select"),
                value: namespace.single_select.to_string(),
                source: Source::ProjectConfig,
            });
            entries.push(ConfigEntry {
                key: format!("labels.{name}.values"),
                value: namespace.values.join(","),
                source: Source::ProjectConfig,
            });
        }
        entries
    }

    pub fn validate(&self) -> Result<(), crate::DomainError> {
        let errors: Vec<_> = self
            .config_errors
            .iter()
            .chain(self.label_namespace_errors.iter())
            .collect();
        if errors.is_empty() {
            return Ok(());
        }
        let description = if self.label_namespace_errors.is_empty() {
            "invalid configuration"
        } else {
            "invalid .tickets/config.toml label namespace configuration"
        };
        Err(crate::DomainError::new(
            crate::ErrorKind::Validation,
            format!(
                "{description}: {}",
                errors
                    .into_iter()
                    .map(|error| error.as_str())
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
        ))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LabelNamespace {
    pub single_select: bool,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidencePolicy {
    Disabled,
    Warn,
    Required,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommitHookPolicy {
    Skip,
    Run,
}

impl CommitHookPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Skip => "skip",
            Self::Run => "run",
        }
    }
}

impl EvidencePolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "false",
            Self::Warn => "warn",
            Self::Required => "true",
        }
    }
}

fn parse_label_namespaces(
    values: &BTreeMap<String, String>,
) -> (BTreeMap<String, LabelNamespace>, Vec<String>) {
    let mut namespaces: BTreeMap<String, LabelNamespace> = BTreeMap::new();
    let mut errors = Vec::new();
    for (key, value) in values.iter().filter(|(key, _)| key.starts_with("labels.")) {
        let Some((namespace, field)) = key[7..].split_once('.') else {
            errors.push(format!(
                "{key}={value:?}: expected [labels.<namespace>] field"
            ));
            continue;
        };
        if namespace.is_empty() || field.contains('.') {
            errors.push(format!("{key}={value:?}: malformed namespace field"));
            continue;
        }
        let namespace = namespaces.entry(namespace.to_string()).or_default();
        match field {
            "single_select" => match value.as_str() {
                "true" => namespace.single_select = true,
                "false" => namespace.single_select = false,
                _ => errors.push(format!("{key}={value:?}: expected true or false")),
            },
            "values" => match serde_json::from_str::<Vec<String>>(value) {
                Ok(parsed) => namespace.values = parsed,
                Err(_) => errors.push(format!(
                    "{key}={value:?}: expected a list of quoted strings"
                )),
            },
            _ => errors.push(format!("{key}={value:?}: unknown label namespace field")),
        }
    }
    (namespaces, errors)
}

/// Apply a key-value pair to the config struct. Returns true if the key was recognized.
fn apply_value(cfg: &mut ProjectConfig, key: &str, value: &str) -> Result<bool, String> {
    let boolean =
        || parse_bool(value).ok_or_else(|| "expected true, false, 1, 0, yes, or no".to_string());
    match key {
        "close.require_resolution" => cfg.close_require_resolution = boolean()?,
        "close.require_checked_acs" => cfg.close_require_checked_acs = boolean()?,
        "close.require_validation_criteria" => cfg.close_require_validation_criteria = boolean()?,
        "close.require_validation_evidence" => {
            cfg.close_require_validation_evidence = match value {
                "false" => EvidencePolicy::Disabled,
                "warn" => EvidencePolicy::Warn,
                "true" => EvidencePolicy::Required,
                _ => return Err("expected true, warn, or false".to_string()),
            }
        }
        "close.allow_force" => cfg.close_allow_force = boolean()?,
        "validate.strict" => cfg.validate_strict = boolean()?,
        "validate.require_source_reference" => cfg.validate_require_source_reference = boolean()?,
        "validate.missing_resolution_before_id" => {
            cfg.validate_missing_resolution_before_id = parse_optional_id(value)?
        }
        "ready.default_env" => {
            cfg.ready_default_env = if value.is_empty() {
                None
            } else {
                Some(
                    crate::core::Env::parse(value)
                        .map_err(|_| "expected corp, personal, or either".to_string())?,
                )
            }
        }
        "priority.warn_unknown" => cfg.priority_warn_unknown = boolean()?,
        "new.default_priority" => {
            cfg.new_default_priority = if value.is_empty() {
                None
            } else {
                Some(
                    crate::core::Priority::parse(value)
                        .ok_or_else(|| "expected urgent, high, medium, or low".to_string())?,
                )
            }
        }
        "push.enabled" => cfg.push_enabled = boolean()?,
        "commit.hooks" => {
            cfg.commit_hooks = match value {
                "skip" => CommitHookPolicy::Skip,
                "run" => CommitHookPolicy::Run,
                _ => return Err("expected skip or run".to_string()),
            }
        }
        "hooks.on_new" | "hooks.on_claim" | "hooks.on_close" => {
            if value.len() > 8192 {
                return Err("expected at most 8192 bytes".into());
            }
            let argv: Vec<String> = serde_json::from_str(value).map_err(|_| {
                "expected a JSON array of executable and argument strings".to_string()
            })?;
            if argv.len() > 16
                || argv.iter().any(|arg| arg.contains('\0'))
                || argv.first().is_some_and(|program| program.is_empty())
            {
                return Err(
                    "expected at most 16 arguments / 8192 bytes, a nonempty executable and no NUL"
                        .into(),
                );
            }
            let index = match key {
                "hooks.on_new" => 0,
                "hooks.on_claim" => 1,
                _ => 2,
            };
            cfg.lifecycle_hooks[index] = argv;
        }
        "machine.capabilities" => {
            cfg.machine_capabilities = value
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        "audit.resolution_baseline_id" => {
            cfg.audit_resolution_baseline_id = parse_optional_id(value)?
        }
        _ => return Ok(false),
    }
    Ok(true)
}

fn parse_bool(s: &str) -> Option<bool> {
    match s {
        "true" | "1" | "yes" => Some(true),
        "false" | "0" | "no" => Some(false),
        _ => None,
    }
}

pub fn parse_update_interval(value: &str) -> Result<Option<u64>, crate::DomainError> {
    let seconds = if value == "never" {
        return Ok(None);
    } else {
        value
            .strip_suffix('h')
            .and_then(|n| n.parse::<u64>().ok())
            .and_then(|n| n.checked_mul(3600))
            .or_else(|| {
                value
                    .strip_suffix('d')
                    .and_then(|n| n.parse::<u64>().ok())
                    .and_then(|n| n.checked_mul(86400))
            })
    };
    seconds.filter(|n| *n > 0).map(Some).ok_or_else(|| crate::DomainError::new(crate::ErrorKind::Validation,
        format!("invalid update_check.interval={value:?}: expected positive whole hours/days (1h, 24h, 7d) or never")))
}

fn parse_optional_id(value: &str) -> Result<Option<u64>, String> {
    if value.is_empty() {
        return Ok(None);
    }
    value
        .parse::<u64>()
        .map(Some)
        .map_err(|_| "expected a numeric ticket id or empty value".to_string())
}

fn config_source_description(source: &Source, key: &str, project_path: &Path) -> String {
    match source {
        Source::Env => format!(
            "environment variable TKT_{}",
            key.replace('.', "_").to_uppercase()
        ),
        Source::ProjectConfig => format!("project config {}", project_path.display()),
        Source::User | Source::ConfigFile => config_file_path()
            .map(|path| format!("user config {}", path.display()))
            .unwrap_or_else(|| "user configuration".to_string()),
        Source::Default => "built-in default".to_string(),
    }
}

fn config_value_error(key: &str, value: &str, source: &str, reason: &str) -> String {
    format!("invalid config {key}={value:?} in {source}: {reason}")
}

fn validate_user_debug_values(values: &BTreeMap<String, String>) -> Vec<String> {
    let mut errors = Vec::new();
    let debug_overridden = std::env::var_os("TKT_DEBUG").is_some();
    if !debug_overridden {
        if let Some(value) = values.get("debug") {
            if parse_bool(value).is_none() {
                errors.push(config_value_error(
                    "debug",
                    value,
                    &config_file_path()
                        .map(|p| format!("user config {}", p.display()))
                        .unwrap_or_else(|| "user configuration".to_string()),
                    "expected true, false, 1, 0, yes, or no",
                ));
            }
        }
    }
    if !debug_overridden {
        if let Some(value) = std::env::var("TKT_DEBUG_FORMAT")
            .ok()
            .or_else(|| values.get("debug.format").cloned())
        {
            if !matches!(value.as_str(), "log" | "file" | "human" | "stderr" | "json") {
                let source = if std::env::var_os("TKT_DEBUG_FORMAT").is_some() {
                    "environment variable TKT_DEBUG_FORMAT".to_string()
                } else {
                    config_file_path()
                        .map(|path| format!("user config {}", path.display()))
                        .unwrap_or_else(|| "user configuration".to_string())
                };
                errors.push(config_value_error(
                    "debug.format",
                    &value,
                    &source,
                    "expected log, file, human, stderr, or json",
                ));
            }
        }
    }
    errors
}

/// Parse a TOML-like config with [sections].
/// Converts `[section]\nkey = value` to `section.key = value` in the map.
fn read_sectioned_config(path: &Path) -> Option<BTreeMap<String, String>> {
    let content = std::fs::read_to_string(path).ok()?;
    Some(parse_sectioned_config(&content))
}

fn parse_sectioned_config(content: &str) -> BTreeMap<String, String> {
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
            let raw_value = value.trim();
            let value = if raw_value.starts_with('"') && raw_value.ends_with('"') {
                serde_json::from_str::<String>(raw_value)
                    .unwrap_or_else(|_| raw_value.trim_matches('"').to_string())
            } else {
                raw_value.to_string()
            };
            let full_key = if current_section.is_empty() {
                key.to_string()
            } else {
                format!("{}.{}", current_section, key)
            };
            map.insert(full_key, value);
        }
    }
    map
}

// ============================================================
// User-level configuration (~/.config/tkt/config.toml)
// ============================================================

/// User configuration state (debug settings + shared project defaults).
#[derive(Debug)]
pub struct Config {
    values: BTreeMap<String, String>,
    errors: Vec<String>,
}

impl Config {
    /// Load config from the platform-appropriate path.
    /// Missing file is not an error (returns all defaults).
    /// Respects TKT_NO_USER_CONFIG=1 to skip loading (for testing).
    pub fn load() -> Self {
        if std::env::var("TKT_NO_USER_CONFIG").as_deref() == Ok("1") {
            return Config {
                values: BTreeMap::new(),
                errors: Vec::new(),
            };
        }
        let values = config_file_path()
            .and_then(|p| read_sectioned_config(&p))
            .unwrap_or_default();
        let errors = if std::env::var_os("TKT_DEBUG").is_none() {
            validate_user_debug_values(&values)
        } else {
            Vec::new()
        };
        Config { values, errors }
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
        parse_bool(&self.get(key)).unwrap_or(false)
    }

    pub fn validate(&self) -> Result<(), crate::DomainError> {
        if self.errors.is_empty() {
            return Ok(());
        }
        Err(crate::DomainError::new(
            crate::ErrorKind::Validation,
            format!("invalid user configuration: {}", self.errors.join("; ")),
        ))
    }

    /// Set a key in the user config file. Creates the file if it doesn't exist.
    pub fn set(key: &str, value: &str) -> anyhow::Result<()> {
        if key == "update_check.interval" {
            parse_update_interval(value)?;
        }
        let path = config_file_path().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "no config directory")
        })?;

        let before = crate::effects::read_text(&path)?;
        let mut values = parse_sectioned_config(before.as_deref().unwrap_or_default());
        values.insert(key.to_string(), value.to_string());
        write_config_file(&path, &values, before)
    }

    /// Remove a key from the user config file (revert to default).
    pub fn unset(key: &str) -> anyhow::Result<bool> {
        let path = config_file_path().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "no config directory")
        })?;

        let before = crate::effects::read_text(&path)?;
        let mut values = parse_sectioned_config(before.as_deref().unwrap_or_default());
        let existed = values.remove(key).is_some();
        write_config_file(&path, &values, before)?;
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
fn write_config_file(
    path: &Path,
    values: &BTreeMap<String, String>,
    before: Option<String>,
) -> anyhow::Result<()> {
    let mut content = String::from("# tkt configuration\n\n");

    // Group by section
    let mut sections: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    for (key, value) in values {
        if let Some((section, field)) = key.rsplit_once('.') {
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
            content.push_str(&format!("{} = {}\n", key, config_literal(key, value)));
        }
        if !flat_keys.is_empty() {
            content.push('\n');
        }
    }

    // Write sectioned keys
    for (section, fields) in &sections {
        content.push_str(&format!("[{}]\n", section));
        for (field, value) in fields {
            let key = format!("{section}.{field}");
            content.push_str(&format!("{} = {}\n", field, config_literal(&key, value)));
        }
        content.push('\n');
    }

    let mut effects = Vec::new();
    if let Some(parent) = path.parent() {
        effects.push(crate::effects::FileEffect::Directory(parent.to_path_buf()));
    }
    effects.push(crate::effects::FileEffect::write(
        path.to_path_buf(),
        before,
        content,
    ));
    crate::effects::FilePlan(effects).execute(crate::effects::Mode::current())
}

fn config_literal(key: &str, value: &str) -> String {
    if key.starts_with("labels.")
        && key.ends_with(".values")
        && serde_json::from_str::<Vec<String>>(value).is_ok()
    {
        return value.to_string();
    }
    let boolean_key = matches!(
        key,
        "close.require_resolution"
            | "close.require_checked_acs"
            | "close.require_validation_criteria"
            | "close.require_validation_evidence"
            | "close.allow_force"
            | "validate.strict"
            | "validate.require_source_reference"
            | "priority.warn_unknown"
            | "push.enabled"
            | "debug"
    ) || (key.starts_with("labels.") && key.ends_with(".single_select"));
    if boolean_key && matches!(value, "true" | "false") {
        value.to_string()
    } else {
        format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
    }
}

/// Set a key in the project config file.
pub fn set_project_config(tickets_dir: &Path, key: &str, value: &str) -> anyhow::Result<()> {
    let path = tickets_dir.join("config.toml");
    let before = crate::effects::read_text(&path)?;
    let mut values = parse_sectioned_config(before.as_deref().unwrap_or_default());
    values.insert(key.to_string(), value.to_string());
    validate_project_values(&values, &path)?;
    write_config_file(&path, &values, before)
}

/// Remove a key from the project config file.
pub fn unset_project_config(tickets_dir: &Path, key: &str) -> anyhow::Result<bool> {
    let path = tickets_dir.join("config.toml");
    let before = crate::effects::read_text(&path)?;
    let mut values = parse_sectioned_config(before.as_deref().unwrap_or_default());
    let existed = values.remove(key).is_some();
    write_config_file(&path, &values, before)?;
    Ok(existed)
}

fn validate_project_values(values: &BTreeMap<String, String>, path: &Path) -> anyhow::Result<()> {
    let mut config = ProjectConfig::default();
    for (key, value) in values {
        if PROJECT_KEYS.iter().any(|(known, _)| known == key) {
            if let Err(reason) = apply_value(&mut config, key, value) {
                config.config_errors.push(config_value_error(
                    key,
                    value,
                    &format!("project config {}", path.display()),
                    &reason,
                ));
            }
        }
    }
    let (namespaces, errors) = parse_label_namespaces(values);
    config.label_namespaces = namespaces;
    config.label_namespace_errors = errors;
    config.validate()?;
    Ok(())
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

        write_config_file(&path, &values, None).unwrap();
        let read_back = read_sectioned_config(&path).unwrap();
        assert_eq!(read_back, values);
    }

    #[test]
    fn test_default_for_known_keys() {
        assert_eq!(default_for("debug"), "false");
        assert_eq!(default_for("debug.format"), "log");
        assert_eq!(default_for("close.allow_force"), "true");
        assert_eq!(default_for("unknown"), "");
    }

    #[test]
    fn test_project_config_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = ProjectConfig::load_inner(dir.path(), true);
        assert!(!cfg.close_require_resolution);
        assert!(cfg.close_require_checked_acs);
        assert!(!cfg.validate_strict);
        assert_eq!(cfg.ready_default_env, None);
        assert!(cfg.priority_warn_unknown);
        assert_eq!(cfg.new_default_priority, None);
        assert!(cfg.push_enabled);
        assert!(cfg.unknown_keys.is_empty());
    }

    #[test]
    fn known_boolean_spellings_are_strict_and_keep_legacy_aliases() {
        for (value, expected) in [
            ("true", true),
            ("1", true),
            ("yes", true),
            ("false", false),
            ("0", false),
            ("no", false),
        ] {
            assert_eq!(parse_bool(value), Some(expected));
        }
        for value in ["", "off", "treu", "maybe"] {
            assert_eq!(parse_bool(value), None);
        }
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
        let cfg = ProjectConfig::load_inner(dir.path(), true);
        assert!(cfg.close_require_resolution);
        assert!(!cfg.push_enabled);
        assert_eq!(cfg.ready_default_env, Some(crate::core::Env::Corp));
        assert_eq!(cfg.unknown_keys, vec!["mystery.foo"]);
    }

    #[test]
    fn test_audit_resolution_baseline_id_parsed() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("config.toml"),
            "[audit]\nresolution_baseline_id = \"36\"\n",
        )
        .unwrap();
        let cfg = ProjectConfig::load_inner(dir.path(), true);
        assert_eq!(cfg.audit_resolution_baseline_id, Some(36));
        assert!(cfg.unknown_keys.is_empty());
    }

    #[test]
    fn test_audit_resolution_baseline_id_default_empty() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = ProjectConfig::load_inner(dir.path(), true);
        assert_eq!(cfg.audit_resolution_baseline_id, None);
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
            apply_value(&mut cfg, key, value).unwrap();
        }
        // Then project overrides
        for (key, value) in &proj_vals {
            apply_value(&mut cfg, key, value).unwrap();
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

        write_config_file(&path, &values, None).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("[close]"));
        assert!(content.contains("allow_force = false"));
        assert!(content.contains("[push]"));

        // Roundtrip
        let read_back = read_sectioned_config(&path).unwrap();
        assert_eq!(read_back, values);
    }
}

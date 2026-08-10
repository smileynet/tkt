//! Shared helpers for command implementations.

use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::git;

/// Bail with a domain error (exit code 1).
macro_rules! domain_bail {
    ($($arg:tt)*) => {
        return Err($crate::DomainError(format!($($arg)*)).into())
    };
}
pub(crate) use domain_bail;

/// Global quiet flag — set once at startup, read by command functions.
pub(crate) fn is_quiet() -> bool {
    crate::QUIET.load(std::sync::atomic::Ordering::Relaxed)
}

/// Resolve the .tickets/ directory from cwd.
pub(crate) fn tickets_dir() -> Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    let root = git::repo_root(&cwd)?;
    let dir = root.join(".tickets");
    if !dir.is_dir() {
        domain_bail!("no .tickets/ directory in {}", root.display());
    }
    Ok(dir)
}

/// Load project-level config from .tickets/config.toml.
/// Warns on unknown keys. Returns defaults if file is missing.
pub(crate) fn project_config(tickets_dir: &Path) -> crate::config::ProjectConfig {
    let cfg = crate::config::ProjectConfig::load(tickets_dir);
    for key in &cfg.unknown_keys {
        eprintln!(
            "warning: unknown config key {:?} in .tickets/config.toml",
            key
        );
    }
    cfg
}

/// Format a success message consistently.
pub(crate) fn success_msg(verb: &str, id: &str, slug: &str, detail: &str) -> String {
    format!(
        "{} {} {} {} ({})",
        crate::color::sym_ok(),
        verb,
        id,
        slug,
        detail
    )
}

/// Extract slug from a ticket filename like "01-my-slug.md" → "my-slug"
pub(crate) fn slug_from_filename(path: &Path) -> String {
    let name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    // Strip leading digits and dash, strip .md
    if let Some(pos) = name.find('-') {
        name[pos + 1..].trim_end_matches(".md").to_string()
    } else {
        name.trim_end_matches(".md").to_string()
    }
}

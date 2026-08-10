//! Shared helpers for command implementations.

use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::core::{self, Status, Ticket};
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

pub(crate) fn has_remote(repo: &Path) -> bool {
    git::has_remote(repo).unwrap_or(false)
}

/// Preflight for mutation commands: resolves context, fetches, loads corpus.
/// Returns (repo, remote, corpus) ready for mutation.
pub(crate) fn preflight_mutation() -> Result<(PathBuf, bool, Vec<Ticket>)> {
    let dir = tickets_dir()?;
    let repo = git::repo_root(&dir)?;
    let remote = has_remote(&repo);

    let dbg = crate::telemetry::debug_mode();
    if remote {
        let fetch_start = std::time::Instant::now();
        git::fetch(&repo)?;
        crate::telemetry::debug_event(
            dbg,
            "",
            "",
            &format!(
                "git fetch origin ({:.1}s)",
                fetch_start.elapsed().as_secs_f64()
            ),
        );
    }
    let corpus = core::load_corpus(&dir)?;
    let open = corpus.iter().filter(|t| t.status == Status::Open).count();
    let wip = corpus
        .iter()
        .filter(|t| t.status == Status::InProgress)
        .count();
    let done = corpus.iter().filter(|t| t.status == Status::Done).count();
    crate::telemetry::debug_event(
        dbg,
        "",
        "",
        &format!(
            "corpus loaded: {} tickets ({} open, {} in_progress, {} done)",
            corpus.len(),
            open,
            wip,
            done
        ),
    );
    Ok((repo, remote, corpus))
}

/// Check the remote state of a ticket. Returns the remote ticket's status if available.
pub(crate) fn check_remote_status(repo: &Path, remote: bool, ticket: &Ticket) -> Option<String> {
    if !remote {
        return None;
    }
    let remote_path = format!(
        ".tickets/{}",
        ticket.path.file_name().unwrap().to_string_lossy()
    );
    if let Ok(content) = git::git(repo, &["show", &format!("origin/main:{}", remote_path)]) {
        if let Ok(remote_file) = core::TicketFile::parse_str(&content, &ticket.path) {
            return remote_file.get("status").map(|s| s.to_string());
        }
    }
    None
}

/// Commit and push a mutation. Handles local-only messaging.
/// Respects project config push.enabled — if false, skips push even when remote exists.
pub(crate) fn commit_and_publish(
    repo: &Path,
    remote: bool,
    paths: &[&str],
    message: &str,
) -> Result<()> {
    let dbg = crate::telemetry::debug_mode();
    git::add(repo, paths)?;
    crate::telemetry::debug_event(dbg, "", "", &format!("git add {:?}", paths));
    git::commit(repo, message)?;
    crate::telemetry::debug_event(dbg, "", "", &format!("git commit {:?}", message));

    // Check project config push.enabled
    let should_push = if remote {
        let dir = repo.join(".tickets");
        if dir.is_dir() {
            let pcfg = crate::config::ProjectConfig::load(&dir);
            pcfg.push_enabled
        } else {
            true
        }
    } else {
        false
    };

    if should_push {
        git::push_with_retry(repo)?;
        crate::telemetry::debug_event(dbg, "", "", "git push (success)");
    } else if remote {
        crate::telemetry::debug_event(dbg, "", "", "push skipped (push.enabled = false)");
    }
    Ok(())
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

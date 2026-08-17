//! `MutationContext` — shared lifecycle for commands that mutate existing tickets.
//!
//! Encapsulates: resolve .tickets/ → repo root → project config → detect remote →
//! fetch → load corpus. Provides `find_ticket`, `remote_status`, and `publish`.

use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::config::ProjectConfig;
use crate::core::{self, Ticket, TicketFile};
use crate::git;
use crate::DomainError;

/// Pre-resolved context for mutating existing tickets. Created via `MutationContext::open()`.
pub struct MutationContext {
    pub repo: PathBuf,
    pub tickets_dir: PathBuf,
    pub has_remote: bool,
    pub config: ProjectConfig,
    pub corpus: Vec<Ticket>,
}

impl MutationContext {
    /// Resolve the full mutation context: tickets dir, repo root, config, remote
    /// detection, fetch (if remote), and corpus load.
    pub fn open() -> Result<Self> {
        let cwd = std::env::current_dir()?;
        let repo = git::repo_root(&cwd)?;
        let tickets_dir = repo.join(".tickets");
        if !tickets_dir.is_dir() {
            return Err(DomainError::new(
                crate::ErrorKind::NotFound,
                format!("no .tickets/ directory in {}", repo.display()),
            )
            .into());
        }

        let config = load_project_config(&tickets_dir);
        let has_remote = git::has_remote(&repo).unwrap_or(false);

        let dbg = crate::telemetry::debug_mode();
        if has_remote {
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

        let corpus = core::load_corpus(&tickets_dir)?;
        let open = corpus
            .iter()
            .filter(|t| t.status == core::Status::Open)
            .count();
        let wip = corpus
            .iter()
            .filter(|t| t.status == core::Status::InProgress)
            .count();
        let done = corpus
            .iter()
            .filter(|t| t.status == core::Status::Done)
            .count();
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

        Ok(Self {
            repo,
            tickets_dir,
            has_remote,
            config,
            corpus,
        })
    }

    /// Find a ticket by ID in the loaded corpus. Returns a domain error if not found.
    pub fn find_ticket(&self, id: &str) -> Result<&Ticket> {
        core::find_ticket(&self.corpus, id).map_err(|_| {
            DomainError::new(
                crate::ErrorKind::NotFound,
                format!("no ticket with id {:?}", id),
            )
            .into()
        })
    }

    /// Check the remote status of a ticket. Returns `Some(status_string)` if remote
    /// has a different state, `None` if no remote or file not found on remote.
    pub fn remote_status(&self, ticket: &Ticket) -> Option<String> {
        if !self.has_remote {
            return None;
        }
        let remote_path = format!(
            ".tickets/{}",
            ticket.path.file_name().unwrap().to_string_lossy()
        );
        if let Ok(content) = git::git(
            &self.repo,
            &["show", &format!("origin/main:{}", remote_path)],
        ) {
            if let Ok(remote_file) = TicketFile::parse_str(&content, &ticket.path) {
                return remote_file.get("status").map(|s| s.to_string());
            }
        }
        None
    }

    /// Stage files, commit, and push (respecting push.enabled). This is the single
    /// push-gated path for existing-ticket mutations.
    pub fn publish(&self, paths: &[&str], message: &str) -> Result<()> {
        let dbg = crate::telemetry::debug_mode();
        git::add(&self.repo, paths)?;
        crate::telemetry::debug_event(dbg, "", "", &format!("git add {:?}", paths));
        git::commit(&self.repo, message)?;
        crate::telemetry::debug_event(dbg, "", "", &format!("git commit {:?}", message));

        if self.has_remote && self.config.push_enabled {
            git::push_with_retry(&self.repo)?;
            crate::telemetry::debug_event(dbg, "", "", "git push (success)");
        } else if self.has_remote {
            crate::telemetry::debug_event(dbg, "", "", "push skipped (push.enabled = false)");
        }
        Ok(())
    }

    /// Compute a relative path from repo root, normalized to forward slashes.
    pub fn rel_path(&self, path: &Path) -> String {
        path.strip_prefix(&self.repo)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/")
    }
}

/// Load project config with unknown-key warnings.
fn load_project_config(tickets_dir: &Path) -> ProjectConfig {
    let cfg = ProjectConfig::load(tickets_dir);
    for key in &cfg.unknown_keys {
        eprintln!(
            "warning: unknown config key {:?} in .tickets/config.toml",
            key
        );
    }
    cfg
}

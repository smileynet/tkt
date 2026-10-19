//! Git transaction management for atomic ticket allocation.
//!
//! Encapsulates: fetch → scan local+remote → allocate IDs → commit → push with bounded retry.

use std::path::{Path, PathBuf};

use anyhow::{bail, Result};

use crate::core;
use crate::git;

/// Outcome of publishing a commit to the remote.
pub enum PublishOutcome {
    /// Commit pushed successfully on first try.
    Published,
    /// No remote configured — committed locally only.
    LocalOnly,
    /// Push was rejected, recovered via rebase, and second push succeeded.
    Retried,
}

/// A git transaction context for ticket allocation operations.
///
/// Handles fetch, remote+local name scanning, and push-with-retry for
/// commands that create new ticket files (new, batch).
pub struct GitTransaction {
    pub repo: PathBuf,
    pub dir: PathBuf,
    pub remote: bool,
    pub upstream: Option<git::Upstream>,
}

impl GitTransaction {
    /// Create a new transaction: resolves repo root, detects remote, fetches if remote exists.
    pub fn new(dir: &Path) -> Result<Self> {
        let repo = git::repo_root(dir)?;
        let has_remote = git::has_remote(&repo)?;

        // Respect project config push.enabled
        let pcfg = crate::config::ProjectConfig::load(dir);
        let remote = has_remote && pcfg.push_enabled;
        if remote && git::current_branch(&repo)?.is_empty() {
            bail!(
                "cannot mutate remote-backed tickets from detached HEAD; switch to a branch or set push.enabled = false for local-only work"
            );
        }
        let upstream = if remote {
            git::resolve_upstream(&repo)?
        } else {
            None
        };

        if let Some(upstream) = &upstream {
            git::fetch(&repo, upstream)?;
        }

        Ok(Self {
            repo,
            dir: dir.to_path_buf(),
            remote,
            upstream,
        })
    }

    /// Scan ticket filenames from both the local directory and resolved upstream.
    /// Returns basenames like ["01-auth.md", "02-feature.md"].
    pub fn scan_names(&self) -> Result<Vec<String>> {
        let mut names = local_ticket_filenames(&self.dir);
        if let Some(upstream) = &self.upstream {
            let remote_names = git::remote_ticket_names(&self.repo, upstream)?;
            for rn in remote_names {
                if !names.contains(&rn) {
                    names.push(rn);
                }
            }
        }
        Ok(names)
    }

    /// Compute the next available ID and zero-padding width from a set of names.
    pub fn next_id(names: &[String]) -> (String, usize) {
        let next = core::max_id(names) + 1;
        let width = core::id_width(names);
        let tid = format!("{:0>width$}", next, width = width);
        (tid, width)
    }

    /// Attempt to push the current HEAD commit. On race rejection:
    /// hard-reset, pull-rebase, then return Err to signal the caller
    /// should reallocate and retry.
    ///
    /// Returns Ok(PublishOutcome) on success, or a retryable signal.
    pub fn try_push(&self) -> Result<PublishResult> {
        if !self.remote {
            return Ok(PublishResult::Done(PublishOutcome::LocalOnly));
        }

        let upstream = self
            .upstream
            .as_ref()
            .expect("remote transactions have an upstream");
        match git::push(&self.repo, upstream)? {
            git::PushResult::Success => Ok(PublishResult::Done(PublishOutcome::Published)),
            git::PushResult::Failed(stderr) => {
                bail!("push failed: {}", stderr);
            }
            git::PushResult::Rejected => {
                // Undo and rebase for retry
                git::undo_commit(&self.repo)?;
                git::pull_rebase(&self.repo, upstream)?;
                Ok(PublishResult::NeedsRetry)
            }
        }
    }

    /// Push after a retry (second attempt). Fails if rejected again.
    pub fn push_retry(&self) -> Result<PublishOutcome> {
        let upstream = self
            .upstream
            .as_ref()
            .expect("remote transactions have an upstream");
        match git::push(&self.repo, upstream)? {
            git::PushResult::Success => Ok(PublishOutcome::Retried),
            git::PushResult::Failed(stderr) => {
                bail!("push failed on retry: {}", stderr);
            }
            git::PushResult::Rejected => {
                bail!("allocation failed after 2 attempts (push repeatedly rejected)");
            }
        }
    }
}

/// Result of try_push — either done or needs caller to retry.
pub enum PublishResult {
    Done(PublishOutcome),
    NeedsRetry,
}

/// List .md filenames in the .tickets/ directory (basenames only).
fn local_ticket_filenames(dir: &Path) -> Vec<String> {
    std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect()
}

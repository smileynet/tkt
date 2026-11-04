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

pub enum PushAttempt {
    Done(PublishOutcome),
    Rejected,
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
    _allocation_lock: Option<crate::operation_lock::OperationLock>,
}

impl GitTransaction {
    pub fn commit_created<'a>(
        &'a self,
        paths: &'a [&'a str],
        message: &'a str,
        event: Option<(&'a crate::config::ProjectConfig, crate::lifecycle::Event)>,
    ) -> Result<Option<crate::lifecycle::PreparedEvent>> {
        let outcome = crate::effects::Phase::AllocationCommit(AllocationCommitPlan {
            transaction: self,
            paths,
            message,
            event,
        })
        .execute(crate::effects::Mode::current())?;
        match outcome {
            crate::effects::Outcome::AllocationCommit(event) => Ok(event),
            crate::effects::Outcome::Preview(_) => Ok(None),
            _ => bail!("allocation commit phase returned no commit result"),
        }
    }
    /// Create a new transaction: resolves repo root, detects remote, fetches if remote exists.
    pub fn new(dir: &Path) -> Result<Self> {
        Self::open(dir, true)
    }

    pub fn new_unfetched(dir: &Path) -> Result<Self> {
        Self::open(dir, false)
    }

    fn open(dir: &Path, fetch: bool) -> Result<Self> {
        let repo = git::repo_root(dir)?;
        let allocation_lock = crate::operation_lock::OperationLock::acquire(&repo, "allocation")?;
        let has_remote = git::has_remote(&repo)?;

        // Respect project config push.enabled
        let pcfg = crate::config::ProjectConfig::load(dir);
        pcfg.validate()?;
        let remote = has_remote
            && pcfg.push_enabled
            && !crate::NO_PUSH.load(std::sync::atomic::Ordering::Relaxed);
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

        if fetch {
            if let Some(upstream) = &upstream {
                git::fetch(&repo, upstream)?;
            }
        }

        Ok(Self {
            repo,
            dir: dir.to_path_buf(),
            remote,
            upstream,
            _allocation_lock: allocation_lock,
        })
    }

    pub fn fetch(&self) -> Result<()> {
        if let Some(upstream) = &self.upstream {
            git::fetch(&self.repo, upstream)?;
        }
        Ok(())
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
        let crate::effects::Outcome::Allocation(AllocationResult::Attempt(result)) =
            crate::effects::Phase::AllocationPublication(AllocationPlan::Attempt(self))
                .execute(crate::effects::Mode::current())?
        else {
            bail!("allocation publication was not applied");
        };
        Ok(result)
    }

    fn try_push_apply(&self) -> Result<PublishResult> {
        match self.push_once(None)? {
            PushAttempt::Done(outcome) => Ok(PublishResult::Done(outcome)),
            PushAttempt::Rejected => {
                git::undo_commit(&self.repo)?;
                git::pull_rebase(
                    &self.repo,
                    self.upstream
                        .as_ref()
                        .expect("rejected remote push has an upstream"),
                )?;
                Ok(PublishResult::NeedsRetry)
            }
        }
    }

    /// Expose rejection before recovery so journal owners can validate their attempt.
    pub fn push_once(&self, revision: Option<&str>) -> Result<PushAttempt> {
        let crate::effects::Outcome::Allocation(AllocationResult::Once(result)) =
            crate::effects::Phase::AllocationPublication(AllocationPlan::Once {
                transaction: self,
                revision,
            })
            .execute(crate::effects::Mode::current())?
        else {
            bail!("allocation publication was not applied");
        };
        Ok(result)
    }

    fn push_once_apply(&self, revision: Option<&str>) -> Result<PushAttempt> {
        if !self.remote {
            return Ok(PushAttempt::Done(PublishOutcome::LocalOnly));
        }

        let upstream = self
            .upstream
            .as_ref()
            .expect("remote transactions have an upstream");
        match git::push_revision(&self.repo, upstream, revision.unwrap_or("HEAD"))? {
            git::PushResult::Success => Ok(PushAttempt::Done(PublishOutcome::Published)),
            git::PushResult::Failed(stderr) => {
                Err(git::local_commit_push_error(&self.repo, &stderr))
            }
            git::PushResult::Rejected => Ok(PushAttempt::Rejected),
        }
    }

    /// Push after a retry (second attempt). Fails if rejected again.
    pub fn push_retry(&self) -> Result<PublishOutcome> {
        let crate::effects::Outcome::Allocation(AllocationResult::Retry(result)) =
            crate::effects::Phase::AllocationPublication(AllocationPlan::Retry(self))
                .execute(crate::effects::Mode::current())?
        else {
            bail!("allocation publication was not applied");
        };
        Ok(result)
    }

    fn push_retry_apply(&self) -> Result<PublishOutcome> {
        if !self.remote {
            return Ok(PublishOutcome::LocalOnly);
        }
        let upstream = self
            .upstream
            .as_ref()
            .expect("remote transactions have an upstream");
        match git::push(&self.repo, upstream)? {
            git::PushResult::Success => Ok(PublishOutcome::Retried),
            git::PushResult::Failed(stderr) => {
                Err(git::local_commit_push_error(&self.repo, &stderr))
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

pub(crate) enum AllocationPlan<'a> {
    Attempt(&'a GitTransaction),
    Once {
        transaction: &'a GitTransaction,
        revision: Option<&'a str>,
    },
    Retry(&'a GitTransaction),
}

pub(crate) struct AllocationCommitPlan<'a> {
    transaction: &'a GitTransaction,
    paths: &'a [&'a str],
    message: &'a str,
    event: Option<(&'a crate::config::ProjectConfig, crate::lifecycle::Event)>,
}

impl AllocationCommitPlan<'_> {
    pub(crate) fn summaries(&self) -> Vec<crate::effects::EffectSummary> {
        let mut effects = vec![crate::effects::EffectSummary {
            operation: "commit-allocated-tickets",
            paths: self
                .paths
                .iter()
                .map(|path| self.transaction.repo.join(path))
                .collect(),
            unresolved: true,
        }];
        effects.extend(git::commit_support_summaries(
            &self.transaction.repo,
            self.message,
        ));
        if let Some((config, event)) = self.event {
            if !config.lifecycle_hooks[event.index()].is_empty() {
                effects.push(crate::lifecycle::notification_summary(
                    &self.transaction.repo,
                    event,
                ));
            }
        }
        effects
    }

    pub(crate) fn apply(self) -> Result<Option<crate::lifecycle::PreparedEvent>> {
        for path in self.paths {
            git::add(&self.transaction.repo, &[*path])?;
        }
        let hook = self.event.and_then(|(config, event)| {
            crate::lifecycle::PreparedEvent::prepare(config, &self.transaction.repo, event)
        });
        git::commit(&self.transaction.repo, self.message, self.paths)?;
        Ok(hook)
    }
}

pub(crate) enum AllocationResult {
    Attempt(PublishResult),
    Once(PushAttempt),
    Retry(PublishOutcome),
}

impl AllocationPlan<'_> {
    pub(crate) fn summary(&self) -> crate::effects::EffectSummary {
        let (transaction, operation) = match self {
            Self::Attempt(transaction) => (*transaction, "publish-allocation-with-race-recovery"),
            Self::Once { transaction, .. } => (*transaction, "publish-recorded-allocation"),
            Self::Retry(transaction) => (*transaction, "publish-retried-allocation"),
        };
        crate::effects::EffectSummary {
            operation,
            paths: vec![transaction.repo.clone()],
            unresolved: transaction.remote,
        }
    }

    pub(crate) fn apply(self) -> Result<AllocationResult> {
        match self {
            Self::Attempt(transaction) => {
                transaction.try_push_apply().map(AllocationResult::Attempt)
            }
            Self::Once {
                transaction,
                revision,
            } => transaction
                .push_once_apply(revision)
                .map(AllocationResult::Once),
            Self::Retry(transaction) => transaction.push_retry_apply().map(AllocationResult::Retry),
        }
    }
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

#[cfg(test)]
mod phase_tests {
    use super::*;

    #[test]
    fn allocation_phase_previews_describe_publication_without_git_or_lock_changes() {
        use crate::effects::{Mode, Outcome, Phase};
        let temp = tempfile::tempdir().unwrap();
        let transaction = GitTransaction {
            repo: temp.path().into(),
            dir: temp.path().join(".tickets"),
            remote: true,
            upstream: Some(git::Upstream {
                remote: "never-contact-fixture".into(),
                reference: "never-contact-fixture/main".into(),
            }),
            _allocation_lock: None,
        };
        let paths = [".tickets/01-fixture.md"];
        for phase in [
            Phase::AllocationCommit(AllocationCommitPlan {
                transaction: &transaction,
                paths: &paths,
                message: "chore(tickets): new 01 fixture",
                event: None,
            }),
            Phase::AllocationPublication(AllocationPlan::Attempt(&transaction)),
            Phase::AllocationPublication(AllocationPlan::Once {
                transaction: &transaction,
                revision: None,
            }),
            Phase::AllocationPublication(AllocationPlan::Retry(&transaction)),
        ] {
            let Outcome::Preview(effects) = phase.execute(Mode::Preview).unwrap() else {
                panic!("preview applied allocation effects");
            };
            assert!(!effects.is_empty());
            if effects[0].operation == "commit-allocated-tickets" {
                assert!(effects
                    .iter()
                    .any(|effect| effect.operation == "temporary-tkt-undo-index-and-cleanup"));
            }
            assert!(effects[0].unresolved);
            assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 0);
        }
    }
}

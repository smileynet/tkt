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
    pub upstream: Option<git::Upstream>,
    pub config: ProjectConfig,
    pub corpus: Vec<Ticket>,
}

pub struct CommitExpectation<'a> {
    pub parent: &'a str,
    pub tree: &'a str,
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

        let config = load_project_config(&tickets_dir)?;
        let has_remote = git::has_remote(&repo)?;
        let push_enabled =
            config.push_enabled && !crate::NO_PUSH.load(std::sync::atomic::Ordering::Relaxed);
        if has_remote && push_enabled && git::current_branch(&repo)?.is_empty() {
            return Err(DomainError::with_hint(
                crate::ErrorKind::Conflict,
                "cannot mutate remote-backed tickets from detached HEAD".to_string(),
                "switch to a branch or set push.enabled = false for local-only work".to_string(),
            )
            .into());
        }
        let upstream = if has_remote && push_enabled {
            git::resolve_upstream(&repo)?
        } else {
            None
        };

        let dbg = crate::telemetry::debug_mode();
        if let Some(upstream) = &upstream {
            let fetch_start = std::time::Instant::now();
            git::fetch(&repo, upstream)?;
            if !crate::DRY_RUN.load(std::sync::atomic::Ordering::Relaxed) {
                crate::telemetry::debug_event(
                    dbg,
                    "",
                    "",
                    &format!(
                        "git fetch {} ({:.1}s)",
                        upstream.remote,
                        fetch_start.elapsed().as_secs_f64()
                    ),
                );
            }
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
            upstream,
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
        let Some(upstream) = &self.upstream else {
            return None;
        };
        let remote_path = format!(
            ".tickets/{}",
            ticket.path.file_name().unwrap().to_string_lossy()
        );
        if let Ok(content) = git::git(
            &self.repo,
            &["show", &format!("{}:{}", upstream.reference, remote_path)],
        ) {
            if let Ok(remote_file) = TicketFile::parse_str(&content, &ticket.path) {
                return remote_file.get("status").map(|s| s.to_string());
            }
        }
        None
    }

    /// Stage files, commit, and push (respecting push.enabled). This is the single
    /// push-gated path for existing-ticket mutations.
    pub fn publish(
        &self,
        paths: &[&str],
        message: &str,
        expected: Option<CommitExpectation<'_>>,
        event: Option<(crate::lifecycle::Event, &str)>,
    ) -> Result<()> {
        crate::effects::Phase::ExistingPublication(PublicationPlan {
            context: self,
            paths,
            message,
            expected,
            event,
        })
        .execute(crate::effects::Mode::current())
        .map(drop)
    }

    /// Compute a relative path from repo root, normalized to forward slashes.
    pub fn rel_path(&self, path: &Path) -> String {
        path.strip_prefix(&self.repo)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/")
    }
}

pub(crate) struct PublicationPlan<'a> {
    context: &'a MutationContext,
    paths: &'a [&'a str],
    message: &'a str,
    expected: Option<CommitExpectation<'a>>,
    event: Option<(crate::lifecycle::Event, &'a str)>,
}

impl PublicationPlan<'_> {
    pub(crate) fn summaries(&self) -> Vec<crate::effects::EffectSummary> {
        let mut effects = vec![crate::effects::EffectSummary {
            operation: "publish-existing-ticket",
            paths: self
                .paths
                .iter()
                .map(|path| self.context.repo.join(path))
                .collect(),
            unresolved: self.context.upstream.is_some(),
        }];
        effects.extend(git::commit_support_summaries(
            &self.context.repo,
            self.message,
        ));
        if let Some((event, _)) = self.event {
            if !self.context.config.lifecycle_hooks[event.index()].is_empty() {
                effects.push(crate::lifecycle::notification_summary(
                    &self.context.repo,
                    event,
                ));
            }
        }
        effects
    }

    pub(crate) fn apply(self) -> Result<()> {
        let Self {
            context,
            paths,
            message,
            expected,
            event,
        } = self;
        let dbg = crate::telemetry::debug_mode();
        let exact_inverse = expected.is_some();
        if let Some(expected) = expected.as_ref() {
            if git::git(&context.repo, &["rev-parse", "HEAD"])? != expected.parent
                || git::git(&context.repo, &["write-tree"])? != expected.tree
            {
                return Err(DomainError::new(
                    crate::ErrorKind::Conflict,
                    "prepared parent or index changed before commit".into(),
                )
                .into());
            }
        } else {
            git::add(&context.repo, paths)?;
            crate::telemetry::debug_event(dbg, "", "", &format!("git add {:?}", paths));
        }
        let hook = event.and_then(|(event, _)| {
            crate::lifecycle::PreparedEvent::prepare(&context.config, &context.repo, event)
        });
        git::commit(&context.repo, message, paths)?;
        crate::telemetry::debug_event(dbg, "", "", &format!("git commit {:?}", message));

        if let Some(expected) = expected {
            let parent = git::git(&context.repo, &["show", "-s", "--format=%P", "HEAD"])?;
            let tree = git::git(&context.repo, &["rev-parse", "HEAD^{tree}"])?;
            if parent != expected.parent || tree != expected.tree {
                return Err(DomainError::with_hint(
                    crate::ErrorKind::Conflict,
                    "created local commit differs from the prepared inverse; it was not pushed".into(),
                    "inspect HEAD and local history; preserve the commit and resolve unexpected hook or concurrent changes before publication".into(),
                ).into());
            }
        }

        let hook_commit = hook.and_then(|hook| hook.capture(&context.repo));
        let publication = (|| -> Result<()> {
            if let Some(upstream) = &context.upstream {
                // The commit above succeeded and is durable. If the push fails for a
                // non-race reason (auth, network, unreachable remote), we deliberately
                // do NOT roll back the commit — git separates commit (local, durable)
                // from push (network), the change is fully recoverable, and undoing a
                // valid commit under a transient failure is the wrong move (#180).
                // Surface a structured, actionable error instead of a bare crash.
                let pushed = if exact_inverse {
                    match git::push(&context.repo, upstream) {
                        Ok(git::PushResult::Success) => Ok(()),
                        Ok(git::PushResult::Rejected) => Err(anyhow::anyhow!(
                        "upstream advanced; exact undo publication does not rebase automatically"
                    )),
                        Ok(git::PushResult::Failed(stderr)) => Err(anyhow::anyhow!(stderr)),
                        Err(error) => Err(error),
                    }
                } else {
                    git::push_with_retry(&context.repo, upstream)
                };
                if let Err(e) = pushed {
                    crate::telemetry::debug_event(dbg, "", "", &format!("git push (failed): {e}"));
                    return Err(git::local_commit_push_error(
                        &context.repo,
                        &format!("{e:#}"),
                    ));
                }
                crate::telemetry::debug_event(dbg, "", "", "git push (success)");
            } else if context.has_remote {
                let reason = if crate::NO_PUSH.load(std::sync::atomic::Ordering::Relaxed) {
                    "--no-push"
                } else {
                    "push.enabled = false"
                };
                crate::telemetry::debug_event(dbg, "", "", &format!("push skipped ({reason})"));
                eprintln!(
                    "created local commit; not pushed ({reason}). Run `git push` when ready."
                );
            }
            Ok(())
        })();
        if let (Some((event, id)), Some(commit)) = (event, hook_commit) {
            crate::lifecycle::dispatch(&context.config, &context.repo, event, id, &commit);
        }
        publication
    }
}

/// Load project config with unknown-key warnings.
fn load_project_config(tickets_dir: &Path) -> Result<ProjectConfig> {
    let cfg = ProjectConfig::load(tickets_dir);
    cfg.validate()?;
    for key in &cfg.unknown_keys {
        eprintln!(
            "warning: unknown config key {:?} in .tickets/config.toml",
            key
        );
    }
    Ok(cfg)
}

#[cfg(test)]
mod phase_tests {
    use super::*;

    #[test]
    fn existing_publication_preview_retains_paths_without_staging_or_publishing() {
        use crate::effects::{Mode, Outcome, Phase};
        let temp = tempfile::tempdir().unwrap();
        let tickets = temp.path().join(".tickets");
        let context = MutationContext {
            repo: temp.path().into(),
            tickets_dir: tickets.clone(),
            has_remote: true,
            upstream: Some(git::Upstream {
                remote: "never-contact-fixture".into(),
                reference: "never-contact-fixture/main".into(),
            }),
            config: ProjectConfig::load(&tickets),
            corpus: vec![],
        };
        let paths = [".tickets/01-fixture.md"];
        let Outcome::Preview(effects) = Phase::ExistingPublication(PublicationPlan {
            context: &context,
            paths: &paths,
            message: "private fixture",
            expected: None,
            event: None,
        })
        .execute(Mode::Preview)
        .unwrap() else {
            panic!("preview applied publication effects");
        };
        assert_eq!(effects[0].operation, "publish-existing-ticket");
        assert_eq!(effects[0].paths, vec![temp.path().join(paths[0])]);
        assert!(effects[0].unresolved);
        assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 0);
    }
}

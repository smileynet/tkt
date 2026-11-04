use anyhow::Result;

use crate::commands::common::{is_dry_run, is_json_output, is_quiet};
use crate::mutation::{CommitExpectation, MutationContext};
use crate::{git, DomainError, ErrorKind};

pub fn run() -> Result<i32> {
    let repo = git::repo_root_cwd()?;
    let plan = crate::undo::prepare(&repo)?;
    if is_dry_run() {
        crate::effects::Phase::UndoApplication(UndoApplicationPlan {
            repo: &repo,
            prepared: &plan,
        })
        .execute(crate::effects::Mode::Preview)?;
        if is_json_output() {
            println!(
                "{}",
                serde_json::json!({"ok":true,"operation":"undo","reverted":plan.commit,"subject":plan.subject,"dry_run":true})
            );
        } else if !is_quiet() {
            println!("Would revert {}: {}", plan.commit, plan.subject);
        }
        return Ok(0);
    }
    crate::effects::Phase::UndoApplication(UndoApplicationPlan {
        repo: &repo,
        prepared: &plan,
    })
    .execute(crate::effects::Mode::current())?;
    if is_json_output() {
        println!(
            "{}",
            serde_json::json!({"ok":true,"operation":"undo","reverted":plan.commit,"subject":plan.subject,"dry_run":false})
        );
    } else if !is_quiet() {
        println!(
            "{} reverted {}: {}",
            crate::color::sym_ok(),
            plan.commit,
            plan.subject
        );
    }
    Ok(0)
}

pub(crate) struct UndoApplicationPlan<'a> {
    repo: &'a std::path::Path,
    prepared: &'a crate::undo::UndoPlan,
}

impl UndoApplicationPlan<'_> {
    pub(crate) fn summaries(&self) -> Vec<crate::effects::EffectSummary> {
        let mut effects = vec![crate::effects::EffectSummary {
            operation: "apply-and-publish-exact-undo",
            paths: self
                .prepared
                .paths
                .iter()
                .map(|path| self.repo.join(path))
                .collect(),
            unresolved: true,
        }];
        effects.extend(git::commit_support_summaries(
            self.repo,
            "chore(tickets): undo",
        ));
        effects
    }
    pub(crate) fn apply(self) -> Result<()> {
        let Self {
            repo,
            prepared: plan,
        } = self;
        let ctx = MutationContext::open()?;
        crate::undo::check_clean(repo)?;
        if git::git(repo, &["rev-parse", "HEAD"])? != plan.commit {
            return Err(DomainError::new(
                ErrorKind::Conflict,
                "HEAD changed since undo preparation".into(),
            )
            .into());
        }
        if let Err(error) = git::git(repo, &["revert", "--no-commit", &plan.commit]) {
            return Err(DomainError::with_hint(
            ErrorKind::Io,
            format!("Git could not apply the prepared inverse: {error:#}"),
            "inspect Git status; any conflict or sequencer state is preserved. Resolve or abort that Git operation explicitly before retrying tkt undo".into(),
        ).into());
        }
        if git::git(repo, &["write-tree"])? != plan.parent_tree
            || !git::git(repo, &["--no-optional-locks", "diff", "--name-only"])?.is_empty()
        {
            return Err(DomainError::with_hint(
                ErrorKind::Conflict,
                "prepared revert differs from the exact prior tree; it was not committed".into(),
                "inspect the index and working files; all prepared changes are retained".into(),
            )
            .into());
        }
        let paths: Vec<_> = plan.paths.iter().map(String::as_str).collect();
        let message = format!("chore(tickets): undo {}", plan.commit);
        if let Err(error) = ctx.publish(
            &paths,
            &message,
            Some(CommitExpectation {
                parent: &plan.commit,
                tree: &plan.parent_tree,
            }),
            None,
        ) {
            if git::git(repo, &["rev-parse", "HEAD"])? == plan.commit {
                return Err(DomainError::with_hint(ErrorKind::Io,format!("revert is prepared but was not committed: {error:#}"),"inspect the staged inverse; fix the commit failure and commit or restore those reviewed paths explicitly".into()).into());
            }
            return Err(error);
        }
        Ok(())
    }
}

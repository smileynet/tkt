use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};

/// Run a git command in the given repo directory and return stdout.
pub fn git(repo: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(["-C", &repo.to_string_lossy()])
        .args(args)
        .output()
        .context("failed to execute git")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git {} failed: {}", args.join(" "), stderr.trim());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

/// Find the repository root from a path inside it.
pub fn repo_root(from: &Path) -> Result<std::path::PathBuf> {
    let root = git(from, &["rev-parse", "--show-toplevel"])?;
    Ok(std::path::PathBuf::from(root))
}

/// Fetch from origin (quiet).
pub fn fetch(repo: &Path) -> Result<()> {
    git(repo, &["fetch", "-q"])?;
    Ok(())
}

/// Stage specific files.
pub fn add(repo: &Path, files: &[&str]) -> Result<()> {
    let mut args = vec!["add", "--"];
    args.extend(files);
    git(repo, &args)?;
    Ok(())
}

/// Commit with a message.
pub fn commit(repo: &Path, message: &str) -> Result<()> {
    git(repo, &["commit", "-qm", message])?;
    Ok(())
}

/// Result of a push attempt.
pub enum PushResult {
    /// Push succeeded.
    Success,
    /// Push was rejected due to non-fast-forward (race condition).
    Rejected,
    /// Push failed for a non-race reason (auth, network, hook, etc).
    Failed(String),
}

/// Push to origin (current branch). Returns a typed result distinguishing
/// race rejections from other failures.
pub fn push(repo: &Path) -> Result<PushResult> {
    let output = Command::new("git")
        .args(["-C", &repo.to_string_lossy(), "push", "-q"])
        .output()
        .context("failed to execute git push")?;

    if output.status.success() {
        return Ok(PushResult::Success);
    }

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // Non-fast-forward rejection indicates a race condition (upstream has new commits).
    // These are the only failures safe to retry with pull --rebase.
    if stderr.contains("non-fast-forward")
        || stderr.contains("fetch first")
        || stderr.contains("rejected")
        || stderr.contains("failed to push some refs")
    {
        Ok(PushResult::Rejected)
    } else {
        Ok(PushResult::Failed(stderr.trim().to_string()))
    }
}

/// Push with retry: on race rejection, pull-rebase and try once more.
/// Non-race failures (auth, network, hooks) are propagated immediately
/// without attempting rebase.
pub fn push_with_retry(repo: &Path) -> Result<()> {
    match push(repo)? {
        PushResult::Success => return Ok(()),
        PushResult::Failed(stderr) => {
            bail!("push failed: {}", stderr);
        }
        PushResult::Rejected => {}
    }

    // First rejection: pull-rebase and retry
    pull_rebase(repo)?;

    match push(repo)? {
        PushResult::Success => Ok(()),
        PushResult::Failed(stderr) => {
            bail!("push failed after rebase: {}", stderr);
        }
        PushResult::Rejected => {
            bail!("push rejected twice — resolve upstream state manually");
        }
    }
}

/// Pull with rebase (quiet).
pub fn pull_rebase(repo: &Path) -> Result<()> {
    git(repo, &["pull", "--rebase", "-q"])?;
    Ok(())
}

/// Undo the last commit, keeping the file changes.
pub fn undo_commit_keep_file(repo: &Path) -> Result<()> {
    git(repo, &["reset", "--soft", "HEAD~1"])?;
    Ok(())
}

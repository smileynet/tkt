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

/// Check if a remote named "origin" is configured. Returns Result to propagate git failures.
pub fn has_remote(repo: &Path) -> Result<bool> {
    let remotes = git(repo, &["remote"])?;
    Ok(!remotes.is_empty())
}

/// Get the current branch name (empty string if detached HEAD).
pub fn current_branch(repo: &Path) -> Result<String> {
    git(repo, &["branch", "--show-current"])
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

    // Only classify as a retryable race rejection when stderr specifically
    // mentions non-fast-forward. Other rejections (hooks, policies) use
    // different phrasing and should NOT trigger a rebase retry.
    if stderr.contains("non-fast-forward") || stderr.contains("fetch first") {
        Ok(PushResult::Rejected)
    } else {
        Ok(PushResult::Failed(stderr.trim().to_string()))
    }
}

/// Push with retry: on race rejection, pull-rebase and try once more.
/// Non-race failures (auth, network, hooks) are propagated immediately
/// without attempting rebase.
///
/// Known limitation: after rebase, the mutation is not revalidated against
/// the rebased state. If upstream changed the same ticket between our commit
/// and push without producing a merge conflict, the stale mutation proceeds.
/// In practice, git's line-level conflict detection catches same-field races,
/// and preflight_mutation's check_remote_status catches the common case.
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

/// Undo the last commit without discarding unrelated worktree changes.
/// Uses mixed reset (undoes commit, preserves worktree) then:
/// - Removes any files in .tickets/ that the commit ADDED
/// - Restores any files in .tickets/ that the commit MODIFIED to their pre-commit state
pub fn undo_commit(repo: &Path) -> Result<()> {
    // Get the list of files added in the commit we're about to undo
    let added = git(
        repo,
        &["diff", "--name-only", "--diff-filter=A", "HEAD~1", "HEAD"],
    )
    .unwrap_or_default();
    // Get the list of files modified in the commit we're about to undo
    let modified = git(
        repo,
        &["diff", "--name-only", "--diff-filter=M", "HEAD~1", "HEAD"],
    )
    .unwrap_or_default();
    // Mixed reset: undo commit, keep worktree
    git(repo, &["reset", "HEAD~1"])?;
    // Remove any files that were newly created by the undone commit
    for file in added.lines() {
        if file.starts_with(".tickets/") {
            let path = repo.join(file);
            let _ = std::fs::remove_file(&path);
        }
    }
    // Restore modified files to their pre-commit state
    for file in modified.lines() {
        if file.starts_with(".tickets/") {
            let _ = git(repo, &["checkout", "HEAD", "--", file]);
        }
    }
    Ok(())
}

/// List ticket filenames from the remote (origin/main) without modifying the working tree.
/// Returns basenames like ["01-auth.md", "02-feature.md"].
/// Strips the .tickets/ prefix from ls-tree output.
/// Returns an empty vec if origin/main doesn't exist or has no .tickets/ directory.
pub fn remote_ticket_names(repo: &Path) -> Vec<String> {
    let output = Command::new("git")
        .args([
            "-C",
            &repo.to_string_lossy(),
            "ls-tree",
            "--name-only",
            "origin/main",
            ".tickets/",
        ])
        .output();

    match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
            .lines()
            .filter_map(|l| {
                // ls-tree returns ".tickets/01-foo.md" — strip the prefix
                l.strip_prefix(".tickets/")
                    .or(Some(l)) // fallback if no prefix (shouldn't happen)
                    .filter(|name| name.ends_with(".md"))
                    .map(|s| s.to_string())
            })
            .collect(),
        _ => Vec::new(),
    }
}

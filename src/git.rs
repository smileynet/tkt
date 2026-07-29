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

/// Push to origin (current branch). Returns Ok(true) if pushed, Ok(false) if rejected.
pub fn push(repo: &Path) -> Result<bool> {
    let output = std::process::Command::new("git")
        .args(["-C", &repo.to_string_lossy(), "push", "-q"])
        .output()
        .context("failed to execute git push")?;
    Ok(output.status.success())
}

/// Push with retry: on rejection, pull-rebase and try once more.
pub fn push_with_retry(repo: &Path) -> Result<()> {
    if push(repo)? {
        return Ok(());
    }
    // First rejection: pull-rebase and retry
    pull_rebase(repo)?;
    if push(repo)? {
        return Ok(());
    }
    anyhow::bail!("push rejected twice — resolve upstream state manually");
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

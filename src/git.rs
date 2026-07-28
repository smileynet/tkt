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

/// Push to origin (current branch).
pub fn push(repo: &Path) -> Result<()> {
    git(repo, &["push", "-q"])?;
    Ok(())
}

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

/// Get repository root from the current working directory.
pub fn repo_root_cwd() -> Result<std::path::PathBuf> {
    repo_root(&std::env::current_dir()?)
}

/// Check whether any remote is configured. Returns Result to propagate git failures.
pub fn has_remote(repo: &Path) -> Result<bool> {
    let remotes = git(repo, &["remote"])?;
    Ok(!remotes.is_empty())
}

/// The remote and branch used for remote-aware ticket operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Upstream {
    pub remote: String,
    /// Short tracking name, for example `origin/main`.
    pub reference: String,
}

impl Upstream {
    fn branch(&self) -> &str {
        self.reference
            .strip_prefix(&format!("{}/", self.remote))
            .unwrap_or(&self.reference)
    }
}

/// Resolve the current branch's tracking ref, falling back to the selected remote's HEAD.
///
/// `None` means the repository has no remotes and is intentionally local-only. A repository
/// with remotes but no resolvable upstream is an error: silently treating it as local-only
/// would bypass race detection.
pub fn resolve_upstream(repo: &Path) -> Result<Option<Upstream>> {
    let remotes: Vec<String> = git(repo, &["remote"])?
        .lines()
        .filter(|name| !name.is_empty())
        .map(ToOwned::to_owned)
        .collect();
    if remotes.is_empty() {
        return Ok(None);
    }

    if let Ok(reference) = git(
        repo,
        &[
            "rev-parse",
            "--abbrev-ref",
            "--symbolic-full-name",
            "@{upstream}",
        ],
    ) {
        if let Some((remote, _branch)) = reference.split_once('/') {
            if remotes.iter().any(|name| name == remote) {
                return Ok(Some(Upstream {
                    remote: remote.to_string(),
                    reference,
                }));
            }
        }
    }

    let selected_remote = selected_remote(repo, &remotes)?;
    let reference = match git(
        repo,
        &[
            "symbolic-ref",
            "--quiet",
            "--short",
            &format!("refs/remotes/{selected_remote}/HEAD"),
        ],
    ) {
        Ok(reference) => reference,
        Err(_) => remote_head_reference(repo, &selected_remote)?,
    };
    if !reference.starts_with(&format!("{selected_remote}/")) {
        bail!("remote {selected_remote:?} returned invalid HEAD reference {reference:?}");
    }
    Ok(Some(Upstream {
        remote: selected_remote,
        reference,
    }))
}

fn remote_head_reference(repo: &Path, remote: &str) -> Result<String> {
    let output = git(repo, &["ls-remote", "--symref", remote, "HEAD"]).with_context(|| {
        format!(
            "cannot resolve an upstream for this branch: remote {remote:?} has no cached HEAD and its HEAD could not be queried; set an upstream or fetch that remote"
        )
    })?;
    let target = output
        .lines()
        .find_map(|line| line.strip_prefix("ref: "))
        .and_then(|line| line.split_whitespace().next())
        .and_then(|name| name.strip_prefix("refs/heads/"))
        .ok_or_else(|| anyhow::anyhow!("remote {remote:?} did not advertise a symbolic HEAD"))?;
    Ok(format!("{remote}/{target}"))
}

fn selected_remote(repo: &Path, remotes: &[String]) -> Result<String> {
    if let Ok(remote) = git(repo, &["config", "--get", "remote.pushDefault"]) {
        if remotes.iter().any(|name| name == &remote) {
            return Ok(remote);
        }
        bail!("remote.pushDefault names unknown remote {remote:?}");
    }
    if remotes.iter().any(|name| name == "origin") {
        return Ok("origin".to_string());
    }
    Ok(remotes[0].clone())
}

/// Get the current branch name (empty string if detached HEAD).
pub fn current_branch(repo: &Path) -> Result<String> {
    git(repo, &["branch", "--show-current"])
}

/// Fetch the selected remote (quiet).
pub fn fetch(repo: &Path, upstream: &Upstream) -> Result<()> {
    git(repo, &["fetch", "-q", &upstream.remote])?;
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
pub fn push(repo: &Path, upstream: &Upstream) -> Result<PushResult> {
    // A resolved remote HEAD may not be configured as this branch's tracking
    // upstream. Push an explicit refspec so the fallback is usable on every
    // platform instead of delegating to `git push`'s branch configuration.
    let refspec = format!("HEAD:{}", upstream.branch());
    let output = Command::new("git")
        .args([
            "-C",
            &repo.to_string_lossy(),
            "push",
            "-q",
            &upstream.remote,
            &refspec,
        ])
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
pub fn push_with_retry(repo: &Path, upstream: &Upstream) -> Result<()> {
    match push(repo, upstream)? {
        PushResult::Success => return Ok(()),
        PushResult::Failed(stderr) => {
            bail!("push failed: {}", stderr);
        }
        PushResult::Rejected => {}
    }

    // First rejection: pull-rebase and retry
    pull_rebase(repo, upstream)?;

    match push(repo, upstream)? {
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
pub fn pull_rebase(repo: &Path, upstream: &Upstream) -> Result<()> {
    git(
        repo,
        &[
            "pull",
            "--rebase",
            "-q",
            &upstream.remote,
            upstream.branch(),
        ],
    )?;
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

/// List ticket filenames from the resolved upstream without modifying the working tree.
/// Returns basenames like ["01-auth.md", "02-feature.md"].
/// Strips the .tickets/ prefix from ls-tree output.
/// An absent .tickets/ directory produces an empty list; a git failure is propagated.
pub fn remote_ticket_names(repo: &Path, upstream: &Upstream) -> Result<Vec<String>> {
    Ok(git(
        repo,
        &["ls-tree", "--name-only", &upstream.reference, ".tickets/"],
    )?
    .lines()
    .filter_map(|line| {
        line.strip_prefix(".tickets/")
            .or(Some(line))
            .filter(|name| name.ends_with(".md"))
            .map(ToOwned::to_owned)
    })
    .collect())
}

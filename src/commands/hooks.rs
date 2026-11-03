//! Explicit, repository-local advisory Git hook installation.
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use clap::ValueEnum;

use super::common::{domain_bail, is_dry_run, is_json_output, is_quiet, tickets_dir};

#[derive(Clone, Copy, ValueEnum)]
pub enum Action {
    Install,
    Check,
    Remove,
    Run,
}

const SHIM: &str = include_str!("../../assets/pre-commit.sh");

pub fn run(action: Action) -> Result<i32> {
    let dir = tickets_dir()?;
    if matches!(action, Action::Run) {
        let findings = crate::findings::collect_corpus_findings(&dir)?;
        crate::findings::print_findings(&findings, !is_json_output(), "advisory");
        return Ok(0);
    }
    let root = crate::git::repo_root(&dir)?;
    let configured = Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["config", "--get", "core.hooksPath"])
        .output()?;
    if configured.status.success() {
        if matches!(action, Action::Check) {
            report("custom-hooks-path", false, None);
            return Ok(0);
        }
        domain_bail!(
            Conflict,
            "core.hooksPath is configured; preserve the existing hook system (no files changed)"
        );
    }
    if configured.status.code() != Some(1) {
        anyhow::bail!(
            "cannot inspect core.hooksPath: {}",
            String::from_utf8_lossy(&configured.stderr)
        );
    }
    let common = PathBuf::from(crate::git::git(
        &root,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    )?);
    let hooks = common.join("hooks");
    reject_link(&hooks)?;
    let path = hooks.join("pre-commit");
    reject_link(&path)?;
    let current = match std::fs::read(&path) {
        Ok(bytes) => Some(bytes),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(error).context("cannot read existing pre-commit hook"),
    };
    let owned = current.as_deref() == Some(SHIM.as_bytes());
    let active = owned;
    #[cfg(unix)]
    let active = {
        use std::os::unix::fs::PermissionsExt;
        active && std::fs::metadata(&path)?.permissions().mode() & 0o111 != 0
    };
    if matches!(action, Action::Check) {
        report(
            if active {
                "installed"
            } else if owned {
                "managed-not-executable"
            } else if current.is_some() {
                "foreign"
            } else {
                "absent"
            },
            active,
            Some(&path),
        );
        return Ok(0);
    }
    if current.is_some() && !owned {
        domain_bail!(
            Conflict,
            "{} is not the exact managed hook; preserve it and integrate manually",
            path.display()
        );
    }
    if matches!(action, Action::Install) && owned && !active {
        domain_bail!(Conflict, "managed hook is not executable; inspect its permissions before enabling it (no files changed)");
    }
    if is_dry_run() {
        report(
            if matches!(action, Action::Install) {
                "would-install"
            } else {
                "would-remove"
            },
            active,
            Some(&path),
        );
        return Ok(0);
    }
    match action {
        Action::Install if !owned => install(&hooks, &path)?,
        Action::Remove if owned => remove_owned(&path)?,
        _ => {}
    }
    report(
        if matches!(action, Action::Install) {
            "installed"
        } else {
            "absent"
        },
        matches!(action, Action::Install),
        Some(&path),
    );
    Ok(0)
}

fn reject_link(path: &Path) -> Result<()> {
    if let Ok(metadata) = std::fs::symlink_metadata(path) {
        let linked = metadata.file_type().is_symlink();
        #[cfg(windows)]
        let linked = {
            use std::os::windows::fs::MetadataExt;
            linked || metadata.file_attributes() & 0x400 != 0
        };
        if linked {
            domain_bail!(Conflict, "refusing linked hook path {}", path.display());
        }
    }
    Ok(())
}

fn install(hooks: &Path, path: &Path) -> Result<()> {
    std::fs::create_dir_all(hooks)?;
    match crate::core::atomic_create(path, SHIM, true) {
        Ok(()) => Ok(()),
        Err(error)
            if error
                .downcast_ref::<crate::DomainError>()
                .is_some_and(|e| e.kind == crate::ErrorKind::Conflict) =>
        {
            reject_link(path)?;
            let identical = std::fs::read(path)? == SHIM.as_bytes();
            #[cfg(unix)]
            let identical = {
                use std::os::unix::fs::PermissionsExt;
                identical && std::fs::metadata(path)?.permissions().mode() & 0o111 != 0
            };
            if identical {
                Ok(())
            } else {
                Err(error)
            }
        }
        Err(error) => Err(error),
    }
}

fn remove_owned(path: &Path) -> Result<()> {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let staged = path.with_file_name(format!(
        ".tkt-removed-pre-commit-{}-{nonce}",
        std::process::id()
    ));
    std::fs::rename(path, &staged)
        .context("cannot stage hook removal; retry after inspecting the hook")?;
    if reject_link(&staged).is_err() || !staged.is_file() {
        domain_bail!(Conflict, "hook changed during removal; preserved at {} (restore manually without overwriting a new hook)", staged.display());
    }
    let bytes = std::fs::read(&staged).with_context(|| {
        format!(
            "cannot inspect moved hook; preserved at {}",
            staged.display()
        )
    })?;
    if bytes != SHIM.as_bytes() {
        restore_foreign(path, &staged)?;
        domain_bail!(
            Conflict,
            "hook changed during removal; restored foreign bytes to {}",
            path.display()
        );
    }
    std::fs::remove_file(&staged).with_context(|| {
        format!(
            "cannot finish owned hook removal; preserved at {}",
            staged.display()
        )
    })?;
    Ok(())
}

fn restore_foreign(path: &Path, staged: &Path) -> Result<()> {
    if std::fs::hard_link(staged, path).is_err() {
        domain_bail!(Conflict, "hook changed during removal; preserved at {}; {} was not overwritten (manual recovery required)", staged.display(), path.display());
    }
    std::fs::remove_file(staged).with_context(|| {
        format!(
            "foreign hook restored; extra staging link remains at {}",
            staged.display()
        )
    })?;
    Ok(())
}

fn report(state: &str, installed: bool, path: Option<&Path>) {
    if is_json_output() {
        println!(
            "{}",
            serde_json::json!({"ok":true,"state":state,"installed":installed,"path":path.map(|p|p.display().to_string()),"advisory":true,"shared_worktrees":true})
        );
    } else if !is_quiet() {
        println!(
            "Advisory pre-commit hook: {state}{} (shared by this repository's worktrees)",
            path.map(|p| format!(" at {}", p.display()))
                .unwrap_or_default()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn changed_hook_after_preflight_is_restored_without_loss() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("pre-commit");
        std::fs::write(&path, b"foreign replacement").unwrap();
        assert!(remove_owned(&path).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"foreign replacement");
        assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 1);
    }

    #[test]
    fn recovery_preserves_both_foreign_versions_when_target_reappears() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("pre-commit");
        let staged = temp.path().join("staged");
        std::fs::write(&path, b"new foreign arrival").unwrap();
        std::fs::write(&staged, b"moved foreign bytes").unwrap();
        assert!(restore_foreign(&path, &staged).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"new foreign arrival");
        assert_eq!(std::fs::read(&staged).unwrap(), b"moved foreign bytes");
    }
}

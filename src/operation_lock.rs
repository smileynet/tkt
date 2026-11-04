use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::{git, DomainError, ErrorKind};

pub struct OperationLock {
    path: PathBuf,
    token: Vec<u8>,
}

impl OperationLock {
    pub fn acquire(repo: &Path, operation: &str) -> Result<Option<Self>> {
        match crate::effects::Phase::OperationLock(LockPlan::Acquire { repo, operation })
            .execute(crate::effects::Mode::current())?
        {
            crate::effects::Outcome::Lock(guard) => Ok(guard),
            crate::effects::Outcome::Preview(_) => Ok(None),
            _ => Err(DomainError::new(
                ErrorKind::Conflict,
                "operation lock phase returned no ownership result".into(),
            )
            .into()),
        }
    }

    fn acquire_apply(repo: &Path, operation: &str) -> Result<Self> {
        let path = PathBuf::from(git::git(
            repo,
            &["rev-parse", "--git-path", &format!("tkt-{operation}.lock")],
        )?);
        let path = if path.is_absolute() {
            path
        } else {
            repo.join(path)
        };
        let mut file = match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => return Err(DomainError::with_hint(
                ErrorKind::Conflict, format!("another or interrupted {operation} operation owns {}", path.display()),
                "wait for the active writer; for an interrupted operation inspect Git status/history and the marker before removing a confirmed stale marker manually".into(),
            ).into()),
            Err(error) => return Err(DomainError::with_hint(ErrorKind::Io, format!("cannot create lock {}: {error}", path.display()), "inspect the reported path and permissions".into()).into()),
        };
        let token = format!(
            "pid={}\nstarted_ns={}\n",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        )
        .into_bytes();
        let guard = Self { path, token };
        file.write_all(&guard.token).map_err(|error| {
            DomainError::with_hint(
                ErrorKind::Io,
                format!("cannot write lock {}: {error}", guard.path.display()),
                "inspect the marker before retrying".into(),
            )
        })?;
        Ok(guard)
    }
}

impl Drop for OperationLock {
    fn drop(&mut self) {
        let _ = crate::effects::Phase::OperationLock(LockPlan::Release(self))
            .execute(crate::effects::Mode::Apply);
    }
}

pub(crate) enum LockPlan<'a> {
    Acquire { repo: &'a Path, operation: &'a str },
    Release(&'a OperationLock),
}

impl LockPlan<'_> {
    pub(crate) fn summaries(&self) -> Vec<crate::effects::EffectSummary> {
        match self {
            Self::Acquire { repo, .. } => vec![
                crate::effects::EffectSummary {
                    operation: "acquire-operation-lock",
                    paths: vec![repo.into()],
                    unresolved: true,
                },
                crate::effects::EffectSummary {
                    operation: "release-operation-lock-if-owned",
                    paths: vec![repo.into()],
                    unresolved: true,
                },
            ],
            Self::Release(lock) => vec![crate::effects::EffectSummary {
                operation: "release-operation-lock",
                paths: vec![lock.path.clone()],
                unresolved: false,
            }],
        }
    }

    pub(crate) fn apply(self) -> Result<Option<OperationLock>> {
        match self {
            Self::Acquire { repo, operation } => {
                OperationLock::acquire_apply(repo, operation).map(Some)
            }
            Self::Release(lock) => {
                lock.release_apply();
                Ok(None)
            }
        }
    }
}

impl OperationLock {
    fn release_apply(&self) {
        let mut actual = Vec::new();
        let read = File::open(&self.path).and_then(|file| {
            file.take(self.token.len() as u64 + 1)
                .read_to_end(&mut actual)
        });
        if read.is_ok() && actual == self.token {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_phase_runs_on_error_and_preserves_a_foreign_marker() {
        let temp = tempfile::tempdir().unwrap();
        let output = std::process::Command::new("git")
            .args(["init", "-q"])
            .arg(temp.path())
            .env("DO_NOT_TRACK", "1")
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        let marker = temp.path().join(".git/tkt-fixture.lock");
        let result: Result<()> = (|| {
            let _guard = OperationLock::acquire(temp.path(), "fixture")?.unwrap();
            assert!(marker.exists());
            anyhow::bail!("controlled private failure");
        })();
        assert!(result.is_err());
        assert!(!marker.exists());
        let guard = OperationLock::acquire(temp.path(), "fixture")
            .unwrap()
            .unwrap();
        std::fs::write(&marker, "foreign ownership").unwrap();
        drop(guard);
        assert_eq!(std::fs::read(marker).unwrap(), b"foreign ownership");
    }
}

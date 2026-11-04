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
    pub fn acquire(repo: &Path, operation: &str) -> Result<Self> {
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

use std::path::PathBuf;

use anyhow::Result;

use crate::{core, DomainError, ErrorKind};

#[derive(Clone, Copy)]
pub enum Mode {
    Preview,
    Apply,
}

impl Mode {
    pub fn current() -> Self {
        if crate::DRY_RUN.load(std::sync::atomic::Ordering::Relaxed) {
            Self::Preview
        } else {
            Self::Apply
        }
    }
}

pub enum FileEffect {
    Directory(PathBuf),
    Write {
        path: PathBuf,
        before: Option<String>,
        text: String,
    },
    Remove(PathBuf),
}

pub struct FilePlan(pub Vec<FileEffect>);

impl FilePlan {
    // Operations are ordered, not transactional: earlier publications survive a
    // later failure. Git transaction owners decide commit and push separately.
    pub fn execute(self, mode: Mode) -> Result<()> {
        if matches!(mode, Mode::Preview) {
            return Ok(());
        }
        for effect in self.0 {
            match effect {
                FileEffect::Directory(path) => {
                    if let Some(metadata) = metadata(&path)? {
                        if !metadata.is_dir() || linked(&metadata) {
                            return Err(conflict(&path, "expected an unlinked directory"));
                        }
                    }
                    std::fs::create_dir_all(&path)
                        .map_err(|e| file_error("create directory", &path, e))?;
                }
                FileEffect::Write { path, before, text } => {
                    if read_text(&path)? != before {
                        return Err(conflict(&path, "changed since preparation"));
                    }
                    if before.is_some() {
                        core::atomic_write(&path, &text)?;
                    } else {
                        core::atomic_create(&path, &text, false)?;
                    }
                }
                FileEffect::Remove(path) => match std::fs::remove_file(&path) {
                    Ok(()) => (),
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
                    Err(e) => return Err(file_error("remove file", &path, e)),
                },
            }
        }
        Ok(())
    }
}

pub fn read_text(path: &std::path::Path) -> Result<Option<String>> {
    let Some(metadata) = metadata(path)? else {
        return Ok(None);
    };
    if !metadata.is_file() || linked(&metadata) {
        return Err(conflict(path, "expected an unlinked regular file"));
    }
    std::fs::read_to_string(path)
        .map(Some)
        .map_err(|e| file_error("read file", path, e))
}

fn metadata(path: &std::path::Path) -> Result<Option<std::fs::Metadata>> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) => Ok(Some(metadata)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(file_error("inspect path", path, e)),
    }
}

fn linked(metadata: &std::fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        metadata.file_type().is_symlink()
    }
}

fn conflict(path: &std::path::Path, reason: &str) -> anyhow::Error {
    DomainError::with_hint(
        ErrorKind::Conflict,
        format!("cannot update {}: {reason}", path.display()),
        "inspect the path and prepare the operation again; preserve foreign content".into(),
    )
    .into()
}

fn file_error(action: &str, path: &std::path::Path, error: std::io::Error) -> anyhow::Error {
    DomainError::with_hint(
        ErrorKind::Io,
        format!("cannot {action} {}: {error}", path.display()),
        format!(
            "check permissions, locks and the filesystem at {}",
            path.display()
        ),
    )
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn changed_snapshot_and_non_directory_are_conflicts() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("file");
        std::fs::write(&path, "original").unwrap();
        let before = read_text(&path).unwrap();
        std::fs::write(&path, "external edit").unwrap();
        let result = FilePlan(vec![FileEffect::Write {
            path: path.clone(),
            before,
            text: "stale merge".into(),
        }])
        .execute(Mode::Apply);
        assert_eq!(
            result
                .unwrap_err()
                .downcast_ref::<DomainError>()
                .unwrap()
                .kind,
            ErrorKind::Conflict
        );
        let result = FilePlan(vec![FileEffect::Directory(path.clone())]).execute(Mode::Apply);
        assert_eq!(
            result
                .unwrap_err()
                .downcast_ref::<DomainError>()
                .unwrap()
                .kind,
            ErrorKind::Conflict
        );
        assert_eq!(std::fs::read_to_string(path).unwrap(), "external edit");
        assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 1);
    }

    #[test]
    fn preview_does_not_create_replace_or_remove() {
        let temp = tempfile::tempdir().unwrap();
        let old = temp.path().join("old");
        std::fs::write(&old, "original").unwrap();
        FilePlan(vec![
            FileEffect::Directory(temp.path().join("new")),
            FileEffect::Write {
                path: old.clone(),
                before: Some("original".into()),
                text: "replacement".into(),
            },
            FileEffect::Write {
                path: temp.path().join("absent"),
                before: None,
                text: "new".into(),
            },
            FileEffect::Remove(old.clone()),
        ])
        .execute(Mode::Preview)
        .unwrap();
        assert_eq!(std::fs::read_to_string(old).unwrap(), "original");
        assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 1);
    }

    #[test]
    fn conflict_keeps_foreign_file_and_earlier_effects() {
        let temp = tempfile::tempdir().unwrap();
        let foreign = temp.path().join("foreign");
        let previous = temp.path().join("previous");
        std::fs::write(&foreign, "foreign bytes").unwrap();
        let result = FilePlan(vec![
            FileEffect::Write {
                path: previous.clone(),
                before: None,
                text: "published".into(),
            },
            FileEffect::Write {
                path: foreign.clone(),
                before: None,
                text: "overwrite".into(),
            },
            FileEffect::Remove(previous.clone()),
        ])
        .execute(Mode::Apply);
        assert_eq!(
            result
                .unwrap_err()
                .downcast_ref::<DomainError>()
                .unwrap()
                .kind,
            ErrorKind::Conflict
        );
        assert_eq!(std::fs::read_to_string(foreign).unwrap(), "foreign bytes");
        assert_eq!(std::fs::read_to_string(previous).unwrap(), "published");
        assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 2);
    }
}

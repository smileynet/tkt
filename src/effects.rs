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
    ReplaceSnapshot {
        path: PathBuf,
        before: String,
        text: String,
    },
    CreateNoClobber {
        path: PathBuf,
        text: String,
        executable: bool,
    },
    RelocateSnapshot {
        source: PathBuf,
        destination: PathBuf,
        before: String,
        text: String,
    },
    RemoveOwned {
        path: PathBuf,
        text: String,
    },
    UnlinkEntry(PathBuf),
    RemoveDirectory(PathBuf),
}

impl FileEffect {
    pub fn write(path: PathBuf, before: Option<String>, text: String) -> Self {
        match before {
            Some(before) => Self::ReplaceSnapshot { path, before, text },
            None => Self::CreateNoClobber {
                path,
                text,
                executable: false,
            },
        }
    }
}

pub struct FilePlan(pub Vec<FileEffect>);
pub struct PreparedFiles(Vec<FileEffect>);

impl FilePlan {
    pub fn prepare(self) -> Result<PreparedFiles> {
        for effect in &self.0 {
            match effect {
                FileEffect::Directory(path) | FileEffect::RemoveDirectory(path) => {
                    if let Some(info) = metadata(path)? {
                        if !info.is_dir() || linked(&info) {
                            return Err(conflict(path, "expected an unlinked directory"));
                        }
                    }
                }
                FileEffect::ReplaceSnapshot { path, before, .. } => {
                    if read_text(path)?.as_deref() != Some(before.as_str()) {
                        return Err(conflict(path, "changed since preparation"));
                    }
                }
                FileEffect::CreateNoClobber { path, .. } => {
                    if metadata(path)?.is_some() {
                        return Err(conflict(
                            path,
                            "already exists; preserved without overwrite",
                        ));
                    }
                }
                FileEffect::RemoveOwned { path, text } => {
                    if read_text(path)?.as_deref() != Some(text.as_str()) {
                        return Err(conflict(path, "owned removal snapshot changed"));
                    }
                }
                FileEffect::UnlinkEntry(_) => (),
                FileEffect::RelocateSnapshot {
                    source,
                    destination,
                    before,
                    ..
                } => {
                    if read_text(source)?.as_deref() != Some(before.as_str()) {
                        return Err(conflict(source, "relocation source changed"));
                    }
                    if metadata(destination)?.is_some() {
                        return Err(conflict(
                            destination,
                            "relocation destination already exists",
                        ));
                    }
                }
            }
        }
        Ok(PreparedFiles(self.0))
    }

    // Operations are ordered, not transactional: earlier publications survive a
    // later failure. Git transaction owners decide commit and push separately.
    pub fn execute(self, mode: Mode) -> Result<()> {
        self.prepare()?.execute(mode)
    }
}

impl PreparedFiles {
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
                FileEffect::ReplaceSnapshot { path, before, text } => {
                    if read_text(&path)?.as_deref() != Some(before.as_str()) {
                        return Err(conflict(&path, "changed since preparation"));
                    }
                    core::atomic_write(&path, &text)?;
                }
                FileEffect::CreateNoClobber {
                    path,
                    text,
                    executable,
                } => core::atomic_create(&path, &text, executable)?,
                FileEffect::RemoveOwned { path, text } => remove_owned(&path, &text)?,
                FileEffect::UnlinkEntry(path) => match std::fs::remove_file(&path) {
                    Ok(()) => (),
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
                    Err(error) => return Err(file_error("unlink entry", &path, error)),
                },
                FileEffect::RelocateSnapshot {
                    source,
                    destination,
                    before,
                    text,
                } => {
                    if read_text(&source)?.as_deref() != Some(before.as_str()) {
                        return Err(conflict(&source, "relocation source changed"));
                    }
                    core::atomic_create(&destination, &text, false)?;
                    remove_owned(&source, &before)?;
                }
                FileEffect::RemoveDirectory(path) => {
                    if let Some(info) = metadata(&path)? {
                        if !info.is_dir() || linked(&info) {
                            return Err(conflict(&path, "expected an unlinked directory"));
                        }
                        std::fs::remove_dir_all(&path)
                            .map_err(|e| file_error("remove directory", &path, e))?;
                    }
                }
            }
        }
        Ok(())
    }
}

fn remove_owned(path: &std::path::Path, expected: &str) -> Result<()> {
    if read_text(path)?.as_deref() != Some(expected) {
        return Err(conflict(path, "owned removal snapshot changed"));
    }
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let staged = path.with_file_name(format!(".tkt-removed-{}-{nonce}", std::process::id()));
    std::fs::rename(path, &staged).map_err(|e| file_error("stage owned removal", path, e))?;
    let actual = read_text(&staged).map_err(|error| {
        DomainError::with_hint(
            error
                .downcast_ref::<DomainError>()
                .map_or(ErrorKind::Io, |domain| domain.kind),
            format!(
                "cannot inspect moved file; preserved at {}: {error}",
                staged.display()
            ),
            "inspect the preserved staging entry and restore it without overwriting new content"
                .into(),
        )
    })?;
    if actual.as_deref() != Some(expected) {
        restore_foreign(path, &staged)?;
        return Err(conflict(
            path,
            "changed during removal; foreign content restored",
        ));
    }
    std::fs::remove_file(&staged)
        .map_err(|e| file_error("finish owned removal; inspect staging file", &staged, e))?;
    Ok(())
}

pub(crate) fn restore_foreign(path: &std::path::Path, staged: &std::path::Path) -> Result<()> {
    if std::fs::hard_link(staged, path).is_err() {
        return Err(DomainError::with_hint(
            ErrorKind::Conflict,
            format!(
                "changed during removal; preserved at {}; {} was not overwritten",
                staged.display(),
                path.display()
            ),
            "restore the preserved file manually without overwriting new content".into(),
        )
        .into());
    }
    std::fs::remove_file(staged)
        .map_err(|e| file_error("remove restored staging link", staged, e))?;
    Ok(())
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

pub(crate) fn linked(metadata: &std::fs::Metadata) -> bool {
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
    fn relocation_previews_and_late_collisions_preserve_both_sides() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        let destination = temp.path().join("destination");
        std::fs::write(&source, "original").unwrap();
        let plan = || {
            FilePlan(vec![FileEffect::RelocateSnapshot {
                source: source.clone(),
                destination: destination.clone(),
                before: "original".into(),
                text: "converted".into(),
            }])
        };
        plan().execute(Mode::Preview).unwrap();
        assert_eq!(std::fs::read(&source).unwrap(), b"original");
        assert!(!destination.exists());
        let prepared = plan().prepare().unwrap();
        std::fs::write(&destination, "late owner").unwrap();
        assert!(prepared.execute(Mode::Apply).is_err());
        assert_eq!(std::fs::read(&source).unwrap(), b"original");
        assert_eq!(std::fs::read(&destination).unwrap(), b"late owner");
        std::fs::remove_file(&destination).unwrap();
        let prepared = plan().prepare().unwrap();
        std::fs::write(&source, "changed source").unwrap();
        assert!(prepared.execute(Mode::Apply).is_err());
        assert_eq!(std::fs::read(&source).unwrap(), b"changed source");
        assert!(!destination.exists());
        std::fs::write(&source, "original").unwrap();
        plan().execute(Mode::Apply).unwrap();
        assert!(!source.exists());
        assert_eq!(std::fs::read(destination).unwrap(), b"converted");
        assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 1);
    }

    #[test]
    fn preview_validates_snapshots_and_apply_rechecks_after_preparation() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("owner");
        std::fs::write(&path, "initial").unwrap();
        let plan = || {
            FilePlan(vec![FileEffect::ReplaceSnapshot {
                path: path.clone(),
                before: "initial".into(),
                text: "replacement".into(),
            }])
        };
        let prepared = plan().prepare().unwrap();
        std::fs::write(&path, "external edit").unwrap();
        for result in [plan().execute(Mode::Preview), prepared.execute(Mode::Apply)] {
            assert_eq!(
                result
                    .unwrap_err()
                    .downcast_ref::<DomainError>()
                    .unwrap()
                    .kind,
                ErrorKind::Conflict
            );
        }
        assert_eq!(std::fs::read(&path).unwrap(), b"external edit");
        let fresh = temp.path().join("fresh");
        let create = FilePlan(vec![FileEffect::CreateNoClobber {
            path: fresh.clone(),
            text: "ours".into(),
            executable: false,
        }])
        .prepare()
        .unwrap();
        std::fs::write(&fresh, "late arrival").unwrap();
        assert_eq!(
            create
                .execute(Mode::Apply)
                .unwrap_err()
                .downcast_ref::<DomainError>()
                .unwrap()
                .kind,
            ErrorKind::Conflict
        );
        assert_eq!(std::fs::read(fresh).unwrap(), b"late arrival");
    }

    #[test]
    fn parsed_ticket_write_preserves_a_changed_source() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("01-owner.md");
        std::fs::write(
            &path,
            "---\nid: \"01\"\ntitle: Owner\nstatus: open\nblocked_by: []\n---\n\nOwner prose\n",
        )
        .unwrap();
        let mut file = core::TicketFile::parse(&path).unwrap();
        file.set_field("title", "Changed title");
        std::fs::write(&path, "concurrent owner bytes").unwrap();
        assert_eq!(
            file.write()
                .unwrap_err()
                .downcast_ref::<DomainError>()
                .unwrap()
                .kind,
            ErrorKind::Conflict
        );
        assert_eq!(std::fs::read(path).unwrap(), b"concurrent owner bytes");
    }

    #[test]
    fn changed_snapshot_and_non_directory_are_conflicts() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("file");
        std::fs::write(&path, "original").unwrap();
        let before = read_text(&path).unwrap();
        std::fs::write(&path, "external edit").unwrap();
        let result = FilePlan(vec![FileEffect::write(
            path.clone(),
            before,
            "stale merge".into(),
        )])
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
            FileEffect::write(old.clone(), Some("original".into()), "replacement".into()),
            FileEffect::write(temp.path().join("absent"), None, "new".into()),
            FileEffect::RemoveOwned {
                path: old.clone(),
                text: "original".into(),
            },
            FileEffect::RemoveDirectory(temp.path().to_path_buf()),
        ])
        .execute(Mode::Preview)
        .unwrap();
        assert_eq!(std::fs::read_to_string(old).unwrap(), "original");
        assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 1);
    }

    #[test]
    fn directory_removal_preserves_linked_root_and_foreign_target() {
        let temp = tempfile::tempdir().unwrap();
        let foreign = temp.path().join("foreign");
        std::fs::create_dir(&foreign).unwrap();
        std::fs::write(foreign.join("owner"), "preserved").unwrap();
        let link = temp.path().join("selected");
        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(&foreign, &link).unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&foreign, &link).unwrap();
        let error = FilePlan(vec![FileEffect::RemoveDirectory(link.clone())])
            .execute(Mode::Apply)
            .unwrap_err();
        assert_eq!(
            error.downcast_ref::<DomainError>().unwrap().kind,
            ErrorKind::Conflict
        );
        assert!(std::fs::symlink_metadata(link)
            .unwrap()
            .file_type()
            .is_symlink());
        assert_eq!(std::fs::read(foreign.join("owner")).unwrap(), b"preserved");
    }

    #[test]
    fn preparation_conflict_preserves_foreign_and_does_not_start_earlier_effects() {
        let temp = tempfile::tempdir().unwrap();
        let foreign = temp.path().join("foreign");
        let previous = temp.path().join("previous");
        std::fs::write(&foreign, "foreign bytes").unwrap();
        let result = FilePlan(vec![
            FileEffect::write(previous.clone(), None, "published".into()),
            FileEffect::write(foreign.clone(), None, "overwrite".into()),
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
        assert!(!previous.exists());
        assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 1);
    }
}

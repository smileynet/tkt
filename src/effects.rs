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

#[derive(Debug)]
pub struct EffectSummary {
    pub operation: &'static str,
    pub paths: Vec<PathBuf>,
    pub unresolved: bool,
}

impl serde::Serialize for EffectSummary {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut summary = serializer.serialize_struct("EffectSummary", 3)?;
        summary.serialize_field("operation", self.operation)?;
        let paths: Vec<_> = self
            .paths
            .iter()
            .map(|path| path.to_string_lossy())
            .collect();
        summary.serialize_field("paths", &paths)?;
        summary.serialize_field("unresolved", &self.unresolved)?;
        summary.end()
    }
}

pub(crate) enum Phase<'a> {
    Files(PreparedFiles),
    Diagnostic(crate::telemetry::DiagnosticPlan<'a>),
    TelemetryMutation(crate::commands::telemetry::TelemetryMutationPlan),
    ExistingPublication(crate::mutation::PublicationPlan<'a>),
    Lifecycle(crate::lifecycle::Notification<'a>),
    OperationLock(crate::operation_lock::LockPlan<'a>),
    RemoteRefresh(crate::git::RemoteRefreshPlan<'a>),
    AllocationPublication(crate::transaction::AllocationPlan<'a>),
    AllocationCommit(crate::transaction::AllocationCommitPlan<'a>),
    Import(crate::commands::import::ImportPlan<'a>),
    CollisionRebase(crate::commands::rebase::RebasePlan<'a>),
    UndoApplication(crate::commands::undo::UndoApplicationPlan<'a>),
    Cadence {
        repo: &'a std::path::Path,
        tags: &'a [String],
        blockers: &'a [String],
    },
}

pub(crate) enum Outcome {
    Applied,
    Preview(Vec<EffectSummary>),
    Cadence(Option<crate::nudge::Hint>),
    Lock(Option<crate::operation_lock::OperationLock>),
    Allocation(crate::transaction::AllocationResult),
    AllocationCommit(Option<crate::lifecycle::PreparedEvent>),
    ImportCommit(String),
    Rebased(crate::renumber::AppliedRenumber),
    TelemetryMutation(&'static str),
}

impl Phase<'_> {
    pub fn summaries(&self) -> Vec<EffectSummary> {
        let summary = |operation, paths, unresolved| EffectSummary {
            operation,
            paths,
            unresolved,
        };
        match self {
            Self::Files(files) => files
                .0
                .iter()
                .map(|effect| match effect {
                    FileEffect::Directory(path) => {
                        summary("create-directory", vec![path.clone()], false)
                    }
                    FileEffect::ReplaceSnapshot { path, .. } => {
                        summary("replace-snapshot", vec![path.clone()], false)
                    }
                    FileEffect::CreateNoClobber { path, .. } => {
                        summary("create-no-clobber", vec![path.clone()], false)
                    }
                    FileEffect::RelocateSnapshot {
                        source,
                        destination,
                        ..
                    } => summary(
                        "relocate-snapshot",
                        vec![source.clone(), destination.clone()],
                        false,
                    ),
                    FileEffect::RemoveOwned { path, .. } => {
                        summary("remove-owned", vec![path.clone()], false)
                    }
                    FileEffect::UnlinkEntry(path) => {
                        summary("unlink-entry", vec![path.clone()], false)
                    }
                    FileEffect::RemoveDirectory(path) => {
                        summary("remove-directory", vec![path.clone()], false)
                    }
                })
                .collect(),
            Self::Diagnostic(plan) => vec![plan.summary()],
            Self::TelemetryMutation(plan) => vec![plan.summary()],
            Self::ExistingPublication(plan) => plan.summaries(),
            Self::Lifecycle(plan) => vec![plan.summary()],
            Self::OperationLock(plan) => plan.summaries(),
            Self::RemoteRefresh(plan) => vec![plan.summary()],
            Self::AllocationPublication(plan) => vec![plan.summary()],
            Self::AllocationCommit(plan) => plan.summaries(),
            Self::Import(plan) => plan.summaries(),
            Self::CollisionRebase(plan) => plan.summaries(),
            Self::UndoApplication(plan) => plan.summaries(),
            Self::Cadence { repo, .. } => {
                vec![summary("record-cadence", vec![repo.to_path_buf()], false)]
            }
        }
    }

    pub fn execute(self, mode: Mode) -> Result<Outcome> {
        if matches!(mode, Mode::Preview) {
            let effects = self.summaries();
            if crate::JSON_OUTPUT.load(std::sync::atomic::Ordering::Relaxed) {
                eprintln!("{}", serde_json::json!({"preview_effects": effects}));
            }
            return Ok(Outcome::Preview(effects));
        }
        match self {
            Self::Files(files) => files.apply()?,
            Self::Diagnostic(plan) => plan.apply(),
            Self::TelemetryMutation(plan) => return plan.apply().map(Outcome::TelemetryMutation),
            Self::ExistingPublication(plan) => plan.apply()?,
            Self::Lifecycle(plan) => plan.apply(),
            Self::OperationLock(plan) => return plan.apply().map(Outcome::Lock),
            Self::RemoteRefresh(plan) => plan.apply()?,
            Self::AllocationPublication(plan) => return plan.apply().map(Outcome::Allocation),
            Self::AllocationCommit(plan) => return plan.apply().map(Outcome::AllocationCommit),
            Self::Import(plan) => return plan.apply(),
            Self::CollisionRebase(plan) => return plan.apply().map(Outcome::Rebased),
            Self::UndoApplication(plan) => plan.apply()?,
            Self::Cadence {
                repo,
                tags,
                blockers,
            } => {
                return Ok(Outcome::Cadence(crate::nudge::record_and_check_apply(
                    repo, tags, blockers,
                )))
            }
        }
        Ok(Outcome::Applied)
    }
}

pub fn replace_state(path: &std::path::Path, text: String) -> Result<()> {
    let mut effects = Vec::new();
    if let Some(parent) = path.parent() {
        effects.push(FileEffect::Directory(parent.into()));
    }
    effects.push(FileEffect::write(path.into(), read_text(path)?, text));
    FilePlan(effects).execute(Mode::current())
}

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
        Phase::Files(self).execute(mode).map(drop)
    }

    fn apply(self) -> Result<()> {
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
    fn preview_reports_lock_and_cadence_without_starting_their_protocols() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::create_dir(temp.path().join(".git")).unwrap();
        for phase in [
            Phase::OperationLock(crate::operation_lock::LockPlan::Acquire {
                repo: temp.path(),
                operation: "fixture",
            }),
            Phase::Cadence {
                repo: temp.path(),
                tags: &[],
                blockers: &[],
            },
        ] {
            let Outcome::Preview(summaries) = phase.execute(Mode::Preview).unwrap() else {
                panic!("preview applied an owner phase");
            };
            assert!(!summaries.is_empty());
            let encoded = serde_json::to_value(&summaries[0]).unwrap();
            assert!(
                encoded["operation"].as_str().unwrap().contains("lock")
                    || encoded["operation"] == "record-cadence"
            );
            assert_eq!(
                std::fs::read_dir(temp.path().join(".git")).unwrap().count(),
                0
            );
        }
    }

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

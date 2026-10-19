pub mod ticket;
pub mod validate;

pub use ticket::{
    corpus_index, find_ticket, frontier, frontier_with_capabilities, id_width, json_string_escape,
    load_corpus, max_id, new_ticket_text, resolve_blocked_by_ref, yaml_scalar_escape, AcSelection,
    CorpusIndex, Env, NewTicketParams, Priority, Status, Ticket, TicketFile, ENV_VALUES,
    STATUS_VALUES,
};

use std::path::Path;

use crate::{DomainError, ErrorKind};

/// Atomically write `contents` to `path`.
///
/// Writes to a sibling temp file in the same directory (same filesystem, so the
/// rename is atomic), then renames it over `path`. A reader never observes a
/// torn/half-written ticket file, and a failed or interrupted write leaves the
/// original file intact — the correct guarantee for a files-are-the-database tool.
///
/// Atomicity only: durability against power loss (fsync of file + parent dir) is
/// deliberately not attempted — ticket files are git-committed, so git is the
/// durability backstop, and avoiding fsync keeps writes fast.
///
/// On failure returns a `DomainError { kind: Io }` (exit code 2) carrying an
/// actionable hint. This is an environment condition (read-only / locked /
/// permission-denied), not a tool bug, so it is surfaced as a domain error rather
/// than an unclassified "crash".
pub fn atomic_write(path: &Path, contents: &str) -> anyhow::Result<()> {
    let dir = path.parent().unwrap_or_else(|| Path::new("."));

    // A per-process, time-suffixed sibling keeps concurrent invocations from
    // colliding while retaining the same-filesystem rename guarantee.
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "ticket".to_string());
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let tmp_path = dir.join(format!(
        ".{file_name}.tmp.{}.{}",
        std::process::id(),
        unique
    ));

    if let Err(e) = std::fs::write(&tmp_path, contents) {
        let _ = std::fs::remove_file(&tmp_path);
        return Err(io_error(path, &e));
    }

    if let Err(e) = std::fs::rename(&tmp_path, path) {
        // Best-effort cleanup of the temp file; report the original cause.
        let _ = std::fs::remove_file(&tmp_path);
        return Err(io_error(path, &e));
    }

    Ok(())
}

/// Build the actionable `DomainError { Io }` for a failed ticket write.
fn io_error(path: &Path, source: &std::io::Error) -> anyhow::Error {
    DomainError::with_hint(
        ErrorKind::Io,
        format!("cannot write {}: {}", path.display(), source),
        format!(
            "the file may be read-only, locked by another process, or on a \
             read-only worktree — e.g. `chmod +w {}`",
            path.display()
        ),
    )
    .into()
}

/// Extract the byte range of the acceptance criteria section content from a ticket body.
/// Returns the range starting after the `## Acceptance criteria` heading line,
/// ending at the next `## ` heading or EOF. Returns None if no AC section exists.
pub fn ac_section_range(body: &str) -> Option<std::ops::Range<usize>> {
    let heading = "## Acceptance criteria";
    // Match only when heading appears at the start of a line (or start of body)
    let start_idx = if body.starts_with(heading) {
        Some(0)
    } else {
        body.find(&format!("\n{}", heading)).map(|i| i + 1)
    }?;
    let after_heading = start_idx + heading.len();
    let content_start = body[after_heading..]
        .find('\n')
        .map(|i| after_heading + i + 1)
        .unwrap_or(body.len());
    let content_end = body[content_start..]
        .find("\n## ")
        .map(|i| content_start + i)
        .unwrap_or(body.len());
    Some(content_start..content_end)
}

#[cfg(test)]
mod tests {
    use super::atomic_write;
    use std::path::PathBuf;

    fn test_dir(label: &str) -> PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("tkt-{label}-{}.{}", std::process::id(), nonce));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn rename_replaces_an_existing_file() {
        let dir = test_dir("rename-overwrite");
        let source = dir.join("source");
        let destination = dir.join("destination");
        std::fs::write(&source, "new").unwrap();
        std::fs::write(&destination, "old").unwrap();

        std::fs::rename(&source, &destination).unwrap();

        assert_eq!(std::fs::read_to_string(&destination).unwrap(), "new");
        assert!(!source.exists());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn atomic_write_replaces_existing_file_without_leftover_temp() {
        let dir = test_dir("atomic-write");
        let destination = dir.join("ticket.md");
        std::fs::write(&destination, "old").unwrap();

        atomic_write(&destination, "new").unwrap();

        assert_eq!(std::fs::read_to_string(&destination).unwrap(), "new");
        let temp_prefix = ".ticket.md.tmp.";
        let has_temp = std::fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .any(|entry| entry.file_name().to_string_lossy().starts_with(temp_prefix));
        assert!(
            !has_temp,
            "successful atomic writes clean up their temp file"
        );
        std::fs::remove_dir_all(dir).unwrap();
    }
}

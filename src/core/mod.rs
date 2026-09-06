pub mod ticket;
pub mod validate;

pub use ticket::{
    corpus_index, find_ticket, frontier, frontier_with_default_env, id_width, json_string_escape,
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

    // Unique-enough temp name in the same directory: <target>.tmp.<pid>.
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "ticket".to_string());
    let tmp_path = dir.join(format!(".{}.tmp.{}", file_name, std::process::id()));

    if let Err(e) = std::fs::write(&tmp_path, contents) {
        return Err(io_error(path, &e));
    }

    // On Windows, fs::rename cannot overwrite an existing destination — delete
    // it first (mirrors renumber.rs). This reintroduces a small non-atomic
    // window on Windows only; POSIX rename overwrites atomically.
    #[cfg(windows)]
    if path.exists() {
        if let Err(e) = std::fs::remove_file(path) {
            let _ = std::fs::remove_file(&tmp_path);
            return Err(io_error(path, &e));
        }
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

//! Derived plan output with an explicitly owned Markdown region.
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::common::{domain_bail, is_dry_run, is_json_output, is_quiet, tickets_dir};
use crate::core::{self, Ticket};

const BEGIN: &str = "<!-- tkt:projection:begin -->";
const END: &str = "<!-- tkt:projection:end -->";

pub fn run(destination: Option<&str>) -> Result<i32> {
    let dir = tickets_dir()?;
    let corpus = core::load_corpus(&dir)?;
    let mut expected = 0;
    for entry in std::fs::read_dir(&dir)? {
        if entry?.path().extension().is_some_and(|ext| ext == "md") {
            expected += 1;
        }
    }
    let ids: HashSet<_> = corpus.iter().map(|ticket| &ticket.id).collect();
    if corpus.len() != expected || ids.len() != corpus.len() {
        domain_bail!(Validation, "cannot generate from incomplete or duplicate ticket identities; repair the corpus first");
    }
    if corpus.iter().any(|ticket| {
        !ticket
            .id
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphanumeric())
            || !ticket
                .id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
    }) {
        domain_bail!(
            Validation,
            "plan-table IDs must use alphanumerics, hyphens or underscores"
        );
    }
    let projection = render(&corpus);
    let Some(destination) = destination else {
        if is_json_output() {
            println!(
                "{}",
                serde_json::json!({"ok":true,"projection":projection,"tickets":corpus.len()})
            );
        } else {
            print!("{projection}");
        }
        return Ok(0);
    };
    let requested = PathBuf::from(destination);
    let parent = requested
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let parent = parent
        .canonicalize()
        .context("projection parent directory must already exist")?;
    let name = requested.file_name().ok_or_else(|| {
        crate::DomainError::new(
            crate::ErrorKind::Validation,
            "projection needs a file path".into(),
        )
    })?;
    let path = parent.join(name);
    if path.starts_with(dir.canonicalize()?) {
        domain_bail!(
            Validation,
            "projection output must be outside the ticket database"
        );
    }
    let old = read_destination(&path)?;
    let output = match &old {
        Some(text) if !text.is_empty() => replace_owned(text, &projection)?,
        _ => projection,
    };
    let changed = old.as_deref() != Some(output.as_str());
    if changed && !is_dry_run() {
        publish(&path, old.as_deref(), &output)?;
    }
    if is_json_output() {
        println!(
            "{}",
            serde_json::json!({"ok":true,"path":path.display().to_string(),"changed":changed,"dry_run":is_dry_run(),"tickets":corpus.len(),"projection":output})
        );
    } else if is_dry_run() {
        print!("{output}");
    } else if !is_quiet() {
        println!(
            "{} {}",
            if changed { "Generated" } else { "Unchanged" },
            path.display()
        );
    }
    Ok(0)
}

fn read_destination(path: &Path) -> Result<Option<String>> {
    if let Ok(metadata) = std::fs::symlink_metadata(path) {
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            domain_bail!(
                Conflict,
                "projection output must be a regular file, not a link or directory"
            );
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if metadata.file_attributes() & 0x400 != 0 {
                domain_bail!(Conflict, "projection output must not be a reparse point");
            }
        }
    }
    match std::fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).context("cannot read projection destination"),
    }
}

fn publish(path: &Path, old: Option<&str>, output: &str) -> Result<()> {
    if let Some(expected) = old {
        if read_destination(path)?.as_deref() != Some(expected) {
            domain_bail!(Conflict, "projection destination changed since preview; no file replaced, retry from current content");
        }
        return core::atomic_write(path, output);
    }
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let staging = path.with_file_name(format!(
        ".tkt-plan-projection-{}-{nonce}",
        std::process::id()
    ));
    core::atomic_write(&staging, output)?;
    let published = std::fs::hard_link(&staging, path);
    let cleanup = std::fs::remove_file(&staging);
    if let Err(error) = published {
        if error.kind() == std::io::ErrorKind::AlreadyExists {
            domain_bail!(
                Conflict,
                "projection destination appeared during generation; preserved it without overwrite"
            );
        }
        return Err(error).context(
            "cannot publish new projection without overwrite; filesystem must support hard links",
        );
    }
    cleanup.context("projection published, but staging cleanup failed")?;
    Ok(())
}

fn render(corpus: &[Ticket]) -> String {
    let mut tickets: Vec<_> = corpus.iter().collect();
    tickets.sort_by_key(|ticket| (ticket.priority_sort_key(), ticket.numeric_key()));
    let mut text = format!("{BEGIN}\n## Ticket projection\n\nGenerated from all ticket files; priority then ID. Human sequencing belongs outside this block.\n\n| ID | Work | Status |\n|---:|---|---|\n");
    for ticket in tickets {
        text.push_str(&format!(
            "| {} | {} | {} |\n",
            cell(&ticket.id),
            cell(&ticket.title),
            cell(&super::sync_plan::plan_status_value(ticket))
        ));
    }
    text.push_str(&format!("{END}\n"));
    text
}

fn cell(value: &str) -> String {
    value
        .chars()
        .map(|c| match c {
            '&' => "&amp;".into(),
            '<' => "&lt;".into(),
            '>' => "&gt;".into(),
            '|' => "&#124;".into(),
            '`' => "&#96;".into(),
            '[' => "&#91;".into(),
            ']' => "&#93;".into(),
            '*' => "&#42;".into(),
            '\\' => "&#92;".into(),
            '\r' | '\n' => " ".into(),
            _ => c.to_string(),
        })
        .collect()
}

fn replace_owned(text: &str, projection: &str) -> Result<String> {
    let mut begins = Vec::new();
    let mut ends = Vec::new();
    let offsets: Vec<_> = text
        .split_inclusive('\n')
        .scan(0usize, |offset, line| {
            let current = *offset;
            *offset += line.len();
            Some(current)
        })
        .collect();
    for (index, line) in crate::renumber::prose_lines(text) {
        if line == BEGIN {
            begins.push(offsets[index]);
        }
        if line == END {
            ends.push(offsets[index] + END.len());
        }
    }
    if begins.len() != 1 || ends.len() != 1 || begins[0] >= ends[0] {
        domain_bail!(Conflict, "existing projection destination needs exactly one ordered, unfenced marker pair; human content was preserved");
    }
    Ok(format!(
        "{}{}{}",
        &text[..begins[0]],
        projection.trim_end_matches('\n'),
        &text[ends[0]..]
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publication_does_not_replace_a_foreign_arrival() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("projection.md");
        std::fs::write(&path, "human arrival").unwrap();
        assert!(publish(&path, None, "generated output").is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "human arrival");
        assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 1);
    }

    #[test]
    fn replacement_rejects_a_stale_output_snapshot() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("projection.md");
        std::fs::write(&path, "new human edit").unwrap();
        assert!(publish(&path, Some("old snapshot"), "generated output").is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "new human edit");
    }
}

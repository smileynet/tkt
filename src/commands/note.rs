use std::fs::File;
use std::io::Read;

use anyhow::Result;

use crate::commands::common::{is_dry_run, is_json_output, is_quiet};
use crate::core::{AgentId, Ticket};
use crate::effects::{read_text, FileEffect, FilePlan, Mode};
use crate::mutation::MutationContext;
use crate::{git, DomainError, ErrorKind};

const MAX_NOTE_BYTES: usize = 64 * 1024;

fn input_error(message: impl Into<String>) -> anyhow::Error {
    DomainError::new(ErrorKind::Validation, message.into()).into()
}

fn io_error(action: &str, error: std::io::Error) -> anyhow::Error {
    DomainError::with_hint(
        ErrorKind::Io,
        format!("{action}: {error}"),
        "check the input/path permissions and retry after inspecting the reported state".into(),
    )
    .into()
}

fn read_input(reader: impl Read) -> Result<String> {
    let mut bytes = Vec::new();
    reader
        .take(MAX_NOTE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| io_error("cannot read note input", e))?;
    if bytes.len() > MAX_NOTE_BYTES {
        return Err(input_error("note input exceeds 64 KiB"));
    }
    String::from_utf8(bytes).map_err(|_| input_error("note input must be UTF-8"))
}

fn nonce() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}

pub fn run(id: &str, source: &str, actor: Option<&AgentId>) -> Result<i32> {
    crate::core::validate::validate_free_text(id, "id", 256).map_err(input_error)?;
    let text = match source {
        "-" => read_input(std::io::stdin().lock())?,
        file if file.starts_with('@') => read_input(
            File::open(&file[1..])
                .map_err(|e| io_error(&format!("cannot open note input {}", &file[1..]), e))?,
        )?,
        literal => {
            if literal.len() > MAX_NOTE_BYTES {
                return Err(input_error("note input exceeds 64 KiB"));
            }
            literal.to_string()
        }
    };
    if text.trim().is_empty() {
        return Err(input_error("note input must contain text"));
    }
    if text.contains("## Resolution") {
        return Err(input_error("the literal '## Resolution' is reserved in note input, including fenced or inline examples; link to that material instead"));
    }
    let repo = git::repo_root_cwd()?;
    let _lock = crate::operation_lock::OperationLock::acquire(&repo, "note")?;
    let ctx = MutationContext::open()?;
    let target = ctx.find_ticket(id)?;
    let path = target.file.path.clone();
    let relative = ctx.rel_path(&path);
    if git::git(&repo, &["ls-files", "--cached", "--", &relative])?.is_empty() {
        return Err(DomainError::new(
            ErrorKind::Conflict,
            format!("{relative} must already be tracked before appending a note"),
        )
        .into());
    }
    if !git::git(
        &repo,
        &[
            "--no-optional-locks",
            "status",
            "--porcelain=v1",
            "--untracked-files=all",
            "--",
            &relative,
        ],
    )?
    .is_empty()
    {
        return Err(DomainError::with_hint(ErrorKind::Conflict,format!("{relative} has uncommitted or untracked content"),"commit or stash that ticket's edits before appending; unrelated staged work may remain".into()).into());
    }
    let before = read_text(&path)?.ok_or_else(|| {
        DomainError::new(
            ErrorKind::NotFound,
            format!("ticket {} disappeared", target.id),
        )
    })?;
    let current = Ticket::parse_str(&before, &path)?;
    if current.id != target.id {
        return Err(DomainError::new(
            ErrorKind::Conflict,
            "ticket identity changed during note preparation".into(),
        )
        .into());
    }
    let actor = actor.map(AgentId::as_str).unwrap_or("anonymous");
    let mut after = before.clone();
    if !after.ends_with('\n') {
        after.push('\n');
    }
    // Distinguish otherwise identical appends when publication reconciles Git patches.
    after.push_str(&format!(
        "\n## Note — {}\n\nBy: {actor}\n\n<!-- tkt-note:v1 {}-{} -->\n",
        super::close::chrono_date(),
        std::process::id(),
        nonce()
    ));
    after.push_str(&text);
    if !after.ends_with('\n') {
        after.push('\n');
    }
    let appended = after.len() - before.len();
    FilePlan(vec![FileEffect::write(path, Some(before), after)]).execute(Mode::current())?;
    ctx.publish(
        &[&relative],
        &format!("chore(tickets): note {}", current.id),
        None,
        None,
    )?;
    if is_json_output() {
        println!(
            "{}",
            serde_json::json!({"ok":true,"operation":"note","id":current.id,"actor":actor,"appended_bytes":appended,"dry_run":is_dry_run()})
        );
    } else if !is_quiet() {
        println!(
            "{} {} note to {}",
            crate::color::sym_ok(),
            if is_dry_run() {
                "would append"
            } else {
                "appended"
            },
            current.id
        );
    }
    Ok(0)
}

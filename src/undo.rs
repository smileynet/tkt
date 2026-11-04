use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use serde_json::{json, Value};

use crate::core::{Status, Ticket};
use crate::{git, DomainError, ErrorKind};

const TRAILER: &str = "Tkt-Undo: ";

#[derive(Clone, Debug, PartialEq, Eq)]
struct Blob {
    mode: String,
    oid: String,
}

#[derive(Debug)]
struct Change {
    path: String,
    before: Option<Blob>,
    after: Blob,
}

#[derive(Debug)]
struct Receipt {
    subject: String,
    parent: String,
    before_tree: String,
    after_tree: String,
    changes: Vec<Change>,
}

pub struct UndoPlan {
    pub commit: String,
    pub subject: String,
    pub parent_tree: String,
    pub paths: Vec<String>,
}

fn refusal(reason: impl std::fmt::Display) -> anyhow::Error {
    DomainError::with_hint(
        ErrorKind::Conflict,
        format!("cannot undo HEAD: {reason}"),
        "inspect Git history and working state; only a clean, receipted new/claim/close at HEAD can be reverted".into(),
    ).into()
}

fn oid(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && value.bytes().all(|b| b.is_ascii_hexdigit())
}

pub(crate) fn operation(subject: &str) -> Option<(&str, &str)> {
    let rest = subject.strip_prefix("chore(tickets): ")?;
    let (kind, args) = rest.split_once(' ')?;
    let id = if kind == "new" {
        let (id, slug) = args.split_once(' ')?;
        crate::core::validate::validate_slug(slug).ok()?;
        id
    } else if matches!(kind, "claim" | "close") {
        args
    } else {
        return None;
    };
    if id.is_empty()
        || !id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return None;
    }
    Some((kind, id))
}

fn blob_json(blob: &Blob) -> Value {
    json!({"mode":blob.mode,"oid":blob.oid})
}

impl Receipt {
    fn encode(&self) -> String {
        json!({"v":1,"subject":self.subject,"parent":self.parent,"before_tree":self.before_tree,"after_tree":self.after_tree,"changes":self.changes.iter().map(|c|json!({"path":c.path,"before":c.before.as_ref().map(blob_json),"after":blob_json(&c.after)})).collect::<Vec<_>>()}).to_string()
    }

    fn decode(raw: &str) -> Result<Self> {
        fn string(value: &Value, key: &str) -> Result<String> {
            value
                .get(key)
                .and_then(Value::as_str)
                .map(str::to_owned)
                .ok_or_else(|| refusal(format!("invalid receipt field {key}")))
        }
        fn blob(value: &Value) -> Result<Blob> {
            let result = Blob {
                mode: string(value, "mode")?,
                oid: string(value, "oid")?,
            };
            if result.mode != "100644" || !oid(&result.oid) {
                return Err(refusal("unsupported file mode or object ID"));
            }
            Ok(result)
        }
        let value: Value = serde_json::from_str(raw).map_err(|_| refusal("malformed receipt"))?;
        if value.get("v").and_then(Value::as_u64) != Some(1) {
            return Err(refusal("unknown receipt version"));
        }
        let rows = value
            .get("changes")
            .and_then(Value::as_array)
            .ok_or_else(|| refusal("missing receipt changes"))?;
        let mut changes = Vec::new();
        for row in rows {
            changes.push(Change {
                path: string(row, "path")?,
                before: if row.get("before") == Some(&Value::Null) {
                    None
                } else {
                    Some(blob(&row["before"])?)
                },
                after: blob(&row["after"])?,
            });
        }
        let result = Self {
            subject: string(&value, "subject")?,
            parent: string(&value, "parent")?,
            before_tree: string(&value, "before_tree")?,
            after_tree: string(&value, "after_tree")?,
            changes,
        };
        if ![&result.parent, &result.before_tree, &result.after_tree]
            .into_iter()
            .all(|s| oid(s))
            || result.encode() != raw
        {
            return Err(refusal(
                "noncanonical, duplicated or unknown receipt fields",
            ));
        }
        Ok(result)
    }
}

fn blob_at(repo: &Path, tree: &str, path: &str) -> Result<Option<Blob>> {
    let bytes = git::git_bytes(repo, &["ls-tree", "-r", "-z", tree, "--", path])?;
    if bytes.is_empty() {
        return Ok(None);
    }
    let text = String::from_utf8(bytes).context("Git tree entry is not UTF-8")?;
    let (header, name) = text
        .trim_end_matches('\0')
        .split_once('\t')
        .ok_or_else(|| refusal("malformed tree entry"))?;
    let parts: Vec<_> = header.split(' ').collect();
    if name != path || parts.len() != 3 || parts[1] != "blob" || !oid(parts[2]) {
        return Err(refusal("unsupported tree entry"));
    }
    Ok(Some(Blob {
        mode: parts[0].into(),
        oid: parts[2].into(),
    }))
}

fn text(repo: &Path, blob: &Blob) -> Result<String> {
    String::from_utf8(git::git_bytes(repo, &["cat-file", "blob", &blob.oid])?)
        .context("ticket/plan blob is not UTF-8")
}

fn changed_paths(repo: &Path, before: &str, after: &str) -> Result<Vec<String>> {
    let bytes = git::git_bytes(
        repo,
        &[
            "diff-tree",
            "--no-commit-id",
            "--no-renames",
            "--name-only",
            "-r",
            "-z",
            before,
            after,
        ],
    )?;
    bytes
        .split(|b| *b == 0)
        .filter(|s| !s.is_empty())
        .map(|s| String::from_utf8(s.to_vec()).context("changed path is not UTF-8"))
        .collect()
}

fn close_body(before: &str, after: &str) -> bool {
    let acceptance = crate::core::ac_section_range(before);
    let has_resolution = crate::core::resolution_section_range(before).is_some();
    let prefix = if has_resolution {
        before
    } else {
        before.trim_end()
    };
    if after.len() < prefix.len() {
        return false;
    }
    let a = prefix.as_bytes();
    let b = after.as_bytes();
    for i in 0..a.len() {
        if a[i] != b[i]
            && !(a[i] == b' '
                && b[i] == b'x'
                && i >= 3
                && i + 1 < a.len()
                && acceptance.as_ref().is_some_and(|range| range.contains(&i))
                && &a[i - 3..=i + 1] == b"- [ ]")
        {
            return false;
        }
    }
    if has_resolution {
        after.len() == before.len()
    } else {
        after
            .get(prefix.len()..)
            .and_then(|s| s.strip_prefix("\n\n## Resolution ("))
            .and_then(|s| s.split_once(")\n\n"))
            .is_some_and(|(date, _)| {
                let bytes = date.as_bytes();
                bytes.len() == 10
                    && bytes[4] == b'-'
                    && bytes[7] == b'-'
                    && bytes
                        .iter()
                        .enumerate()
                        .all(|(i, b)| i == 4 || i == 7 || b.is_ascii_digit())
                    && date[5..7]
                        .parse::<u8>()
                        .is_ok_and(|m| (1..=12).contains(&m))
                    && date[8..10]
                        .parse::<u8>()
                        .is_ok_and(|d| (1..=31).contains(&d))
            })
    }
}

fn validate_receipt(repo: &Path, receipt: &Receipt) -> Result<()> {
    let (kind, id) =
        operation(&receipt.subject).ok_or_else(|| refusal("unrecognized operation"))?;
    let actual = changed_paths(repo, &receipt.before_tree, &receipt.after_tree)?;
    let recorded: Vec<_> = receipt.changes.iter().map(|c| c.path.clone()).collect();
    if actual != recorded || !(1..=2).contains(&actual.len()) {
        return Err(refusal("receipt path set differs from the tree change"));
    }
    let tickets: Vec<_> = receipt
        .changes
        .iter()
        .filter(|c| {
            c.path.starts_with(".tickets/")
                && c.path.ends_with(".md")
                && !c.path[9..].contains(['/', '\\'])
        })
        .collect();
    if tickets.len() != 1
        || receipt
            .changes
            .iter()
            .any(|c| c.path != "docs/plan.md" && c.path != tickets[0].path)
    {
        return Err(refusal("unsupported changed paths"));
    }
    for change in &receipt.changes {
        if change.after.mode != "100644"
            || change.before.as_ref().is_some_and(|b| b.mode != "100644")
        {
            return Err(refusal("unsupported ticket or plan mode"));
        }
        if blob_at(repo, &receipt.before_tree, &change.path)? != change.before
            || blob_at(repo, &receipt.after_tree, &change.path)? != Some(change.after.clone())
        {
            return Err(refusal("receipt blob or mode mismatch"));
        }
    }
    let change = tickets[0];
    let after = Ticket::parse_str(&text(repo, &change.after)?, Path::new(&change.path))
        .map_err(|e| refusal(format!("invalid resulting ticket: {e}")))?;
    if after.id != id {
        return Err(refusal("ticket ID differs from the operation"));
    }
    if kind == "new" {
        let slug = receipt.subject.rsplit_once(' ').unwrap().1;
        if change.before.is_some()
            || receipt.changes.len() != 1
            || change.path != format!(".tickets/{id}-{slug}.md")
            || matches!(after.status, Status::Done | Status::Cancelled)
        {
            return Err(refusal("new transition differs from its receipt"));
        }
    } else {
        let before_blob = change
            .before
            .as_ref()
            .ok_or_else(|| refusal("missing original ticket"))?;
        let before = Ticket::parse_str(&text(repo, before_blob)?, Path::new(&change.path))
            .map_err(|e| refusal(format!("invalid original ticket: {e}")))?;
        let allowed = if kind == "claim" {
            &["status", "assignee"][..]
        } else {
            &["status"][..]
        };
        if before.id != id
            || before
                .file
                .fm
                .iter()
                .filter(|(k, _)| !allowed.contains(&k.as_str()))
                .ne(after
                    .file
                    .fm
                    .iter()
                    .filter(|(k, _)| !allowed.contains(&k.as_str())))
        {
            return Err(refusal("unrelated ticket metadata changed"));
        }
        let valid = if kind == "claim" {
            before.status == Status::Open
                && after.status == Status::InProgress
                && before.body == after.body
        } else {
            !matches!(before.status, Status::Done | Status::Cancelled)
                && after.status == Status::Done
                && close_body(&before.body, &after.body)
        };
        if !valid {
            return Err(refusal(
                "ticket transition or body differs from the operation",
            ));
        }
        if let Some(plan) = receipt.changes.iter().find(|c| c.path == "docs/plan.md") {
            let before = plan
                .before
                .as_ref()
                .ok_or_else(|| refusal("newly tracked plan has no safe recorded prior state"))?;
            let value = if kind == "close" {
                "✅ done"
            } else {
                "in_progress"
            };
            if crate::commands::sync_plan::rewrite_status_cell(&text(repo, before)?, id, value)
                != text(repo, &plan.after)?
            {
                return Err(refusal("plan changed outside the matching status cells"));
            }
        }
    }
    Ok(())
}

struct PrivateIndex(PathBuf);
impl PrivateIndex {
    fn create() -> Result<Self> {
        let root = std::env::temp_dir();
        for _ in 0..8 {
            let nonce = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos();
            let dir = root.join(format!("tkt-undo-index-{}-{nonce}", std::process::id()));
            match std::fs::create_dir(&dir) {
                Ok(()) => return Ok(Self(dir)),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => (),
                Err(e) => return Err(e.into()),
            }
        }
        Err(refusal("cannot allocate a private Git index"))
    }
    fn git(&self, repo: &Path, args: &[&str]) -> Result<String> {
        let out = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(args)
            .env("GIT_INDEX_FILE", self.0.join("index"))
            .stdin(Stdio::null())
            .output()?;
        if !out.status.success() {
            return Err(anyhow::anyhow!(
                "private index preparation failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            ));
        }
        Ok(String::from_utf8(out.stdout)?.trim().into())
    }
}
impl Drop for PrivateIndex {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(self.0.join("index"));
        let _ = std::fs::remove_file(self.0.join("index.lock"));
        let _ = std::fs::remove_dir(&self.0);
    }
}

fn head(repo: &Path) -> Result<Option<String>> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["rev-parse", "--verify", "--quiet", "HEAD"])
        .output()?;
    if out.status.success() {
        return Ok(Some(String::from_utf8(out.stdout)?.trim().into()));
    }
    if out.status.code() == Some(1) {
        return Ok(None);
    }
    Err(anyhow::anyhow!(
        "cannot resolve HEAD: {}",
        String::from_utf8_lossy(&out.stderr).trim()
    ))
}

pub fn decorate_commit(repo: &Path, subject: &str, paths: &[&str]) -> Result<String> {
    if operation(subject).is_none() {
        return Ok(subject.into());
    }
    let Some(parent) = head(repo)? else {
        return Ok(subject.into());
    };
    let before_tree = git::git(repo, &["rev-parse", "HEAD^{tree}"])?;
    let index = PrivateIndex::create()?;
    index.git(repo, &["read-tree", &parent])?;
    let mut args = vec!["add", "--"];
    args.extend_from_slice(paths);
    index.git(repo, &args)?;
    let after_tree = index.git(repo, &["write-tree"])?;
    let mut changes = Vec::new();
    for path in changed_paths(repo, &before_tree, &after_tree)? {
        let Some(after) = blob_at(repo, &after_tree, &path)? else {
            return Ok(subject.into());
        };
        changes.push(Change {
            before: blob_at(repo, &before_tree, &path)?,
            path,
            after,
        });
    }
    let receipt = Receipt {
        subject: subject.into(),
        parent,
        before_tree,
        after_tree,
        changes,
    };
    match validate_receipt(repo, &receipt) {
        Ok(()) => (),
        Err(e)
            if e.downcast_ref::<DomainError>()
                .is_some_and(|d| d.kind == ErrorKind::Conflict) =>
        {
            return Ok(subject.into())
        }
        Err(e) => return Err(e),
    }
    Ok(format!("{subject}\n\n{TRAILER}{}", receipt.encode()))
}

pub fn check_clean(repo: &Path) -> Result<()> {
    if !git::git(
        repo,
        &[
            "--no-optional-locks",
            "status",
            "--porcelain=v1",
            "--untracked-files=no",
        ],
    )?
    .is_empty()
    {
        return Err(refusal("index or tracked worktree is dirty"));
    }
    for state in [
        "MERGE_HEAD",
        "CHERRY_PICK_HEAD",
        "REVERT_HEAD",
        "rebase-merge",
        "rebase-apply",
        "sequencer",
        "index.lock",
    ] {
        let p = PathBuf::from(git::git(repo, &["rev-parse", "--git-path", state])?);
        let p = if p.is_absolute() { p } else { repo.join(p) };
        match std::fs::symlink_metadata(p) {
            Ok(_) => return Err(refusal(format!("preexisting Git state {state}"))),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}

pub fn prepare(repo: &Path) -> Result<UndoPlan> {
    check_clean(repo)?;
    let commit = head(repo)?.ok_or_else(|| refusal("no commit at HEAD"))?;
    let parents = git::git(repo, &["show", "-s", "--format=%P", &commit])?;
    if parents.split_whitespace().count() != 1 {
        return Err(refusal("root and merge commits are not supported"));
    }
    let message = git::git(repo, &["show", "-s", "--format=%B", &commit])?;
    let lines: Vec<_> = message.lines().collect();
    if lines.len() != 3 || !lines[1].is_empty() || !lines[2].starts_with(TRAILER) {
        return Err(refusal(
            "missing or duplicated generated receipt; legacy and undo commits are not recognized",
        ));
    }
    let receipt = Receipt::decode(&lines[2][TRAILER.len()..])?;
    if lines[0] != receipt.subject
        || parents != receipt.parent
        || git::git(repo, &["rev-parse", "HEAD^{tree}"])? != receipt.after_tree
        || git::git(
            repo,
            &["rev-parse", &format!("{}^{{tree}}", receipt.parent)],
        )? != receipt.before_tree
    {
        return Err(refusal("receipt parent or tree does not match HEAD"));
    }
    validate_receipt(repo, &receipt)?;
    Ok(UndoPlan {
        commit,
        subject: receipt.subject,
        parent_tree: receipt.before_tree,
        paths: receipt.changes.into_iter().map(|c| c.path).collect(),
    })
}

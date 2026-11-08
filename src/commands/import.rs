//! Create-only GitHub import; provider reads finish before local application.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::Result;
use serde_json::{json, Value};

use super::common::{is_dry_run, is_json_output, is_quiet, tickets_dir};
use crate::core::{self, Ticket, TicketFile};
use crate::effects::{read_text, FileEffect, FilePlan, Mode};
use crate::github::{self, Extraction, Issue, Repository};
use crate::transaction::{GitTransaction, PushAttempt};
use crate::{git, DomainError, ErrorKind};

const FORMAT: &str = "tkt-github-import/v1";
const SNAPSHOTS: &str = ".tickets/imports/github-v1";

fn conflict(message: impl Into<String>) -> anyhow::Error {
    DomainError::with_hint(ErrorKind::Conflict, message.into(), "inspect Git status, the import journal and source ownership before retrying; do not overwrite conflicting files".into()).into()
}

fn required<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value[key]
        .as_str()
        .ok_or_else(|| conflict(format!("invalid import metadata: {key}")))
}

fn metadata_path(repo: &Path, name: &str) -> Result<PathBuf> {
    let path = PathBuf::from(git::git(repo, &["rev-parse", "--git-path", name])?);
    Ok(if path.is_absolute() {
        path
    } else {
        repo.join(path)
    })
}

fn hash(repo: &Path, text: &str) -> Result<String> {
    let mut child = Command::new("git")
        .args(["-C", &repo.to_string_lossy(), "hash-object", "--stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    child
        .stdin
        .take()
        .expect("piped stdin")
        .write_all(text.as_bytes())?;
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(conflict("cannot compute the import content identity"));
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}

pub(crate) struct Pair {
    source_id: u64,
    local_id: String,
    ticket_path: String,
    ticket: String,
    snapshot_path: String,
    snapshot: String,
}

impl Pair {
    fn files(&self) -> [(&str, &str); 2] {
        [
            (&self.ticket_path, &self.ticket),
            (&self.snapshot_path, &self.snapshot),
        ]
    }
    fn journal(&self) -> Value {
        json!({"source_id":self.source_id,"local_id":self.local_id,"ticket_path":self.ticket_path,"ticket":self.ticket,"snapshot_path":self.snapshot_path,"snapshot":self.snapshot})
    }
}

fn pair(
    repo: &Path,
    repository: &Repository,
    issue: &Issue,
    id: &str,
    criteria: &[String],
) -> Result<Pair> {
    core::validate::validate_id(id).map_err(conflict)?;
    let ticket_path = format!(".tickets/{id}-github-{}.md", issue.id);
    let snapshot_path = format!("{SNAPSHOTS}/{}.json", issue.id);
    let body = issue.body.as_deref().unwrap_or("");
    let longest = body.split(|c| c != '`').map(str::len).max().unwrap_or(0);
    let fence = "`".repeat(longest.max(2) + 1);
    let body = format!("# Imported GitHub issue\n\nImported from {} (repository {}, issue {}).\n\n## Source body\n\n{fence}markdown\n{body}\n{fence}\n\n## Acceptance criteria\n\n- [ ] {}\n", issue.url, repository.name, issue.number, core::DEFAULT_AC_PROMPT);
    let text = core::new_ticket_text(&core::NewTicketParams {
        id,
        title: &issue.title,
        body: Some(&body),
        blocked_by: &[],
        env: None,
        spec: Some(&issue.url),
        priority: None,
        status: Some("open"),
        validation_criteria: criteria,
        tags: &["import:github".into()],
        requires: &[],
        agent_eligibility: &core::AgentEligibility::Any,
    });
    let mut ticket = TicketFile::parse_str(&text, &repo.join(&ticket_path))?;
    ticket.set_field(
        "import_source",
        &format!("\"github.com:issue:{}\"", issue.id),
    );
    let ticket = ticket.serialize();
    let snapshot = serde_json::to_string_pretty(
        &json!({"format":FORMAT,"host":"github.com","source_id":issue.id,"repository":{"id":repository.id,"full_name":repository.name},"local_id":id,"ticket_path":ticket_path,"api_version":github::API_VERSION,"ticket_blob":hash(repo,&ticket)?,"validation_criteria":criteria,"issue":issue.raw}),
    )? + "\n";
    Ok(Pair {
        source_id: issue.id,
        local_id: id.into(),
        ticket_path,
        ticket,
        snapshot_path,
        snapshot,
    })
}

fn decode_snapshot(repo: &Path, path: &str, text: &str) -> Result<Pair> {
    let value: Value = serde_json::from_str(text)
        .map_err(|_| conflict(format!("unmanaged or malformed snapshot {path}")))?;
    if value["format"] != FORMAT
        || value["host"] != "github.com"
        || value["api_version"] != github::API_VERSION
    {
        return Err(conflict(format!(
            "unsupported snapshot ownership at {path}"
        )));
    }
    let name = required(&value["repository"], "full_name")?;
    let metadata = json!({"id":value["repository"]["id"],"full_name":name,"html_url":format!("https://github.com/{name}")});
    let extraction = github::decode(&metadata, name, &[json!([value["issue"]])], None)?;
    let issue = extraction
        .issues
        .first()
        .ok_or_else(|| conflict("an owned snapshot must record the originally open issue"))?;
    let criteria: Vec<String> = value["validation_criteria"]
        .as_array()
        .ok_or_else(|| conflict("snapshot requires local validation criteria array"))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| conflict("snapshot validation criterion must be a string"))
        })
        .collect::<Result<_>>()?;
    validate_criteria(&criteria)?;
    let expected = pair(
        repo,
        &extraction.repository,
        issue,
        required(&value, "local_id")?,
        &criteria,
    )?;
    let expected_value: Value = serde_json::from_str(&expected.snapshot)?;
    if expected_value != value || expected.snapshot_path != path {
        return Err(conflict(format!(
            "snapshot identity/content mismatch at {path}"
        )));
    }
    Ok(expected)
}

fn source(ticket: &Ticket) -> Result<Option<u64>> {
    let Some(raw) = ticket.file.get("import_source") else {
        return Ok(None);
    };
    let decoded: String = if raw.starts_with('"') {
        serde_json::from_str(raw).map_err(|_| conflict("invalid import_source scalar"))?
    } else {
        raw.into()
    };
    let id = decoded
        .strip_prefix("github.com:issue:")
        .and_then(|id| id.parse::<u64>().ok())
        .filter(|id| *id > 0)
        .ok_or_else(|| conflict("unrecognized import_source identity"))?;
    Ok(Some(id))
}

fn inventory(
    repo: &Path,
    paths: &[String],
    mut read: impl FnMut(&str) -> Result<String>,
) -> Result<BTreeMap<u64, Pair>> {
    let mut tickets = BTreeMap::new();
    let mut local_ids = BTreeSet::new();
    for path in paths.iter().filter(|path| {
        path.starts_with(".tickets/") && path.ends_with(".md") && !path[9..].contains('/')
    }) {
        let ticket = Ticket::parse_str(&read(path)?, &repo.join(path))?;
        if !local_ids.insert(ticket.id.clone())
            || !path.starts_with(&format!(".tickets/{}-", ticket.id))
        {
            return Err(conflict(
                "duplicate local ticket ID or filename/frontmatter identity mismatch",
            ));
        }
        if let Some(id) = source(&ticket)? {
            if tickets.insert(id, (ticket.id, path)).is_some() {
                return Err(conflict(format!("duplicate imported identity {id}")));
            }
        }
    }
    let mut snapshots = BTreeMap::new();
    for path in paths
        .iter()
        .filter(|path| path.starts_with(&format!("{SNAPSHOTS}/")))
    {
        let pair = decode_snapshot(repo, path, &read(path)?)?;
        let Some((id, ticket_path)) = tickets.remove(&pair.source_id) else {
            return Err(conflict(format!(
                "snapshot {path} has no matching imported ticket"
            )));
        };
        if id != pair.local_id || *ticket_path != pair.ticket_path {
            return Err(conflict(format!(
                "snapshot {path} references a different local ticket"
            )));
        }
        snapshots.insert(pair.source_id, pair);
    }
    if !tickets.is_empty() {
        return Err(conflict("imported ticket has no owned source snapshot"));
    }
    Ok(snapshots)
}

fn local_paths(repo: &Path) -> Result<Vec<String>> {
    let mut paths = Vec::new();
    for entry in std::fs::read_dir(repo.join(".tickets"))? {
        let path = entry?.path();
        if path.extension().is_some_and(|extension| extension == "md") {
            paths.push(format!(
                ".tickets/{}",
                path.file_name().expect("directory entry").to_string_lossy()
            ));
        }
    }
    let directory = repo.join(SNAPSHOTS);
    if directory.exists() {
        for entry in std::fs::read_dir(directory)? {
            paths.push(format!(
                "{SNAPSHOTS}/{}",
                entry?.file_name().to_string_lossy()
            ));
        }
    }
    paths.sort();
    Ok(paths)
}

fn identity_inventory(txn: &GitTransaction) -> Result<BTreeMap<u64, Pair>> {
    let mut local = inventory(&txn.repo, &local_paths(&txn.repo)?, |path| {
        read_text(&txn.repo.join(path))?.ok_or_else(|| conflict(format!("{path} disappeared")))
    })?;
    if let Some(upstream) = &txn.upstream {
        let paths: Vec<String> = git::git(
            &txn.repo,
            &[
                "ls-tree",
                "-r",
                "--name-only",
                &upstream.reference,
                "--",
                ".tickets/",
            ],
        )?
        .lines()
        .map(str::to_owned)
        .collect();
        let remote = inventory(&txn.repo, &paths, |path| {
            git::git_bytes(
                &txn.repo,
                &["show", &format!("{}:{path}", upstream.reference)],
            )
            .and_then(|bytes| Ok(String::from_utf8(bytes)?))
        })?;
        for (id, pair) in remote {
            if let Some(existing) = local.get(&id) {
                if existing.snapshot != pair.snapshot {
                    return Err(conflict(format!(
                        "local/upstream ownership differs for source {id}"
                    )));
                }
            } else {
                local.insert(id, pair);
            }
        }
    }
    Ok(local)
}

fn directories(repo: &Path) -> Result<()> {
    for path in [".tickets", ".tickets/imports", SNAPSHOTS] {
        match std::fs::symlink_metadata(repo.join(path)) {
            Ok(meta) if !meta.is_dir() || crate::effects::linked(&meta) => {
                return Err(conflict(format!("{path} must be an unlinked directory")))
            }
            Ok(_) => (),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

fn clean(repo: &Path) -> Result<()> {
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
        return Err(conflict("import requires a clean tracked worktree and index; unrelated untracked files may remain"));
    }
    no_git_operation(repo)
}

fn no_git_operation(repo: &Path) -> Result<()> {
    for state in [
        "MERGE_HEAD",
        "CHERRY_PICK_HEAD",
        "REVERT_HEAD",
        "rebase-merge",
        "rebase-apply",
        "sequencer",
        "index.lock",
    ] {
        if metadata_path(repo, state)?.exists() {
            return Err(conflict(format!("unfinished Git operation: {state}")));
        }
    }
    Ok(())
}

fn journal_value(
    branch: &str,
    selector: &Value,
    source: &Value,
    base: &str,
    pairs: &[Pair],
) -> Value {
    json!({"format":FORMAT,"branch":branch,"selector":selector,"source":source,"phase":"prepared","base":base,"commit":null,"pairs":pairs.iter().map(Pair::journal).collect::<Vec<_>>()})
}

fn update_journal(path: &Path, phase: &str, commit: Option<&str>) -> Result<()> {
    let text = read_text(path)?.ok_or_else(|| conflict("import journal disappeared"))?;
    let mut value: Value = serde_json::from_str(&text)?;
    value["phase"] = json!(phase);
    value["commit"] = json!(commit);
    FilePlan(vec![FileEffect::ReplaceSnapshot {
        path: path.into(),
        before: text,
        text: serde_json::to_string_pretty(&value)? + "\n",
    }])
    .execute(Mode::current())
}

fn remove_committed_journal(path: &Path, selector: &Value, commit: &str) -> Result<()> {
    let text = read_text(path)?.ok_or_else(|| conflict("import journal disappeared"))?;
    let value: Value = serde_json::from_str(&text)?;
    if value["format"] != FORMAT
        || value["selector"] != *selector
        || value["phase"] != "committed"
        || value["commit"] != commit
    {
        return Err(conflict("cannot remove a changed committed import journal"));
    }
    FilePlan(vec![FileEffect::RemoveOwned {
        path: path.into(),
        text,
    }])
    .execute(Mode::current())
}

fn start_attempt(path: &Path, value: &Value) -> Result<()> {
    let text = serde_json::to_string_pretty(value)? + "\n";
    match read_text(path)? {
        None => FilePlan(vec![FileEffect::CreateNoClobber {
            path: path.into(),
            text,
            executable: false,
        }])
        .execute(Mode::current()),
        Some(previous) => {
            let old: Value = serde_json::from_str(&previous)?;
            if !same_retry(&old, value) {
                return Err(conflict(
                    "new import attempt cannot replace a different pending journal",
                ));
            }
            FilePlan(vec![FileEffect::write(
                path.to_owned(),
                Some(previous),
                text,
            )])
            .execute(Mode::current())
        }
    }
}

fn same_retry(old: &Value, new: &Value) -> bool {
    old["phase"] == "retry_ready"
        && old["format"] == FORMAT
        && old["selector"] == new["selector"]
        && old["source"] == new["source"]
        && old["branch"] == new["branch"]
}

fn journal_pairs(repo: &Path, value: &Value) -> Result<Vec<Pair>> {
    if value["format"] != FORMAT {
        return Err(conflict("unrecognized import journal; it was preserved"));
    }
    let rows = value["pairs"]
        .as_array()
        .filter(|pairs| !pairs.is_empty() && pairs.len() < github::MAX_OBJECTS)
        .ok_or_else(|| conflict("invalid journal pairs"))?;
    let mut pairs = Vec::new();
    let mut ids = BTreeSet::new();
    let mut local_ids = BTreeSet::new();
    for row in rows {
        let pair = decode_snapshot(
            repo,
            required(row, "snapshot_path")?,
            required(row, "snapshot")?,
        )?;
        if pair.journal() != *row
            || !ids.insert(pair.source_id)
            || !local_ids.insert(pair.local_id.clone())
        {
            return Err(conflict("journal pair content or identities do not match"));
        }
        pairs.push(pair);
    }
    Ok(pairs)
}

fn oid(value: &str) -> Result<&str> {
    if !matches!(value.len(), 40 | 64) || !value.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err(conflict("invalid journal Git object identity"));
    }
    Ok(value)
}

fn apply(repo: &Path, pairs: &[Pair]) -> Result<()> {
    directories(repo)?;
    let mut effects = vec![
        FileEffect::Directory(repo.join(".tickets/imports")),
        FileEffect::Directory(repo.join(SNAPSHOTS)),
    ];
    for pair in pairs {
        for (path, text) in pair.files() {
            match read_text(&repo.join(path))? {
                Some(current) if current == text => (),
                Some(_) => {
                    return Err(conflict(format!("interrupted import collides with {path}")))
                }
                None => effects.push(FileEffect::write(repo.join(path), None, text.into())),
            }
        }
    }
    FilePlan(effects).execute(Mode::current())
}

fn paths(pairs: &[Pair]) -> Vec<&str> {
    pairs
        .iter()
        .flat_map(|pair| pair.files().into_iter().map(|(path, _)| path))
        .collect()
}

fn owned_changes_only(repo: &Path, pairs: &[Pair]) -> Result<()> {
    let expected: BTreeSet<_> = paths(pairs).into_iter().collect();
    for args in [
        vec!["--no-optional-locks", "diff", "--name-only", "-z", "--"],
        vec![
            "--no-optional-locks",
            "diff",
            "--cached",
            "--name-only",
            "-z",
            "--",
        ],
    ] {
        let changed = git::git(repo, &args)?;
        if changed
            .split('\0')
            .filter(|path| !path.is_empty())
            .any(|path| !expected.contains(path))
        {
            return Err(conflict(
                "unrelated tracked/index edits prevent import recovery",
            ));
        }
    }
    for pair in pairs {
        for (path, text) in pair.files() {
            if !git::git(repo, &["ls-files", "--cached", "--", path])?.is_empty()
                && git::git_bytes(repo, &["show", &format!(":{path}")])? != text.as_bytes()
            {
                return Err(conflict(format!("journal index content changed at {path}")));
            }
        }
    }
    Ok(())
}

fn verify_commit(repo: &Path, base: &str, commit: &str, pairs: &[Pair]) -> Result<()> {
    clean(repo)?;
    if git::git(repo, &["rev-parse", "HEAD"])? != commit
        || git::git(repo, &["rev-list", "--parents", "-n", "1", commit])?
            != format!("{commit} {base}")
    {
        return Err(conflict(
            "import HEAD/parent changed; retained local work requires inspection",
        ));
    }
    let expected: BTreeSet<_> = paths(pairs).into_iter().map(str::to_owned).collect();
    let added: BTreeSet<_> = git::git(
        repo,
        &["diff", "--name-only", "--diff-filter=A", base, commit, "--"],
    )?
    .lines()
    .map(str::to_owned)
    .collect();
    let changed: BTreeSet<_> = git::git(repo, &["diff", "--name-only", base, commit, "--"])?
        .lines()
        .map(str::to_owned)
        .collect();
    if expected != added || expected != changed {
        return Err(conflict(
            "import commit changed unexpected paths; it remains local",
        ));
    }
    for pair in pairs {
        for (path, text) in pair.files() {
            if read_text(&repo.join(path))?.as_deref() != Some(text)
                || git::git_bytes(repo, &["show", &format!("{commit}:{path}")])? != text.as_bytes()
            {
                return Err(conflict(format!(
                    "import content changed at {path}; no automatic rollback or push"
                )));
            }
        }
    }
    Ok(())
}

fn commit_pairs(
    txn: &GitTransaction,
    journal: &Path,
    branch: &str,
    base: &str,
    pairs: &[Pair],
) -> Result<String> {
    let crate::effects::Outcome::ImportCommit(commit) =
        crate::effects::Phase::Import(ImportPlan::Commit {
            txn,
            journal,
            branch,
            base,
            pairs,
        })
        .execute(Mode::current())?
    else {
        return Err(conflict("import commit was not applied"));
    };
    Ok(commit)
}

fn commit_pairs_apply(
    txn: &GitTransaction,
    journal: &Path,
    branch: &str,
    base: &str,
    pairs: &[Pair],
) -> Result<String> {
    no_git_operation(&txn.repo)?;
    owned_changes_only(&txn.repo, pairs)?;
    if git::git(&txn.repo, &["symbolic-ref", "--quiet", "HEAD"])? != branch
        || git::git(&txn.repo, &["rev-parse", "HEAD"])? != base
    {
        return Err(conflict(
            "import branch/HEAD changed before applying its journal",
        ));
    }
    let mut base_args = vec!["ls-tree", "-r", "--name-only", base, "--"];
    let pair_paths = paths(pairs);
    base_args.extend_from_slice(&pair_paths);
    if !git::git(&txn.repo, &base_args)?.is_empty() {
        return Err(conflict(
            "journal destinations were already tracked at its base",
        ));
    }
    apply(&txn.repo, pairs)?;
    update_journal(journal, "applied", None)?;
    let paths = paths(pairs);
    git::add(&txn.repo, &paths)?;
    git::commit(&txn.repo, "chore(tickets): import GitHub issues", &paths)?;
    let commit = git::git(&txn.repo, &["rev-parse", "HEAD"])?;
    verify_commit(&txn.repo, base, &commit, pairs)?;
    update_journal(journal, "committed", Some(&commit))?;
    Ok(commit)
}

fn rollback(
    txn: &GitTransaction,
    journal: &Path,
    branch: &str,
    base: &str,
    commit: &str,
    pairs: &[Pair],
) -> Result<()> {
    crate::effects::Phase::Import(ImportPlan::Rollback {
        txn,
        journal,
        branch,
        base,
        commit,
        pairs,
    })
    .execute(Mode::current())
    .map(drop)
}

fn rollback_apply(
    txn: &GitTransaction,
    journal: &Path,
    branch: &str,
    base: &str,
    commit: &str,
    pairs: &[Pair],
) -> Result<()> {
    verify_commit(&txn.repo, base, commit, pairs)?;
    update_journal(journal, "rolling_back", Some(commit))?;
    if git::git(&txn.repo, &["symbolic-ref", "--quiet", "HEAD"])? != branch {
        return Err(conflict("import branch changed before rollback"));
    }
    git::git(
        &txn.repo,
        &[
            "update-ref",
            "-m",
            "tkt import allocation retry",
            branch,
            base,
            commit,
        ],
    )?;
    finish_rollback(txn, journal, branch, base, pairs)
}

fn finish_rollback(
    txn: &GitTransaction,
    journal: &Path,
    branch: &str,
    base: &str,
    pairs: &[Pair],
) -> Result<()> {
    crate::effects::Phase::Import(ImportPlan::FinishRollback {
        txn,
        journal,
        branch,
        base,
        pairs,
    })
    .execute(Mode::current())
    .map(drop)
}

fn finish_rollback_apply(
    txn: &GitTransaction,
    journal: &Path,
    branch: &str,
    base: &str,
    pairs: &[Pair],
) -> Result<()> {
    if git::git(&txn.repo, &["symbolic-ref", "--quiet", "HEAD"])? != branch
        || git::git(&txn.repo, &["rev-parse", "HEAD"])? != base
    {
        return Err(conflict("rollback HEAD changed; import journal preserved"));
    }
    no_git_operation(&txn.repo)?;
    owned_changes_only(&txn.repo, pairs)?;
    for pair in pairs {
        for (path, text) in pair.files() {
            if let Some(current) = read_text(&txn.repo.join(path))? {
                if current != text {
                    return Err(conflict(format!("rollback collision at {path}")));
                }
            }
            if !git::git(&txn.repo, &["ls-tree", "--name-only", base, "--", path])?.is_empty() {
                return Err(conflict(format!("rollback base already owned {path}")));
            }
        }
    }
    let mut reset = vec!["reset", base, "--"];
    let pair_paths = paths(pairs);
    reset.extend_from_slice(&pair_paths);
    git::git(&txn.repo, &reset)?;
    if git::git(&txn.repo, &["rev-parse", "HEAD"])? != base {
        return Err(conflict(
            "HEAD advanced during index cleanup; pair files retained",
        ));
    }
    for pair in pairs {
        for (path, text) in pair.files() {
            match read_text(&txn.repo.join(path))? {
                Some(current) if current == text => FilePlan(vec![FileEffect::RemoveOwned {
                    path: txn.repo.join(path),
                    text: current,
                }])
                .execute(Mode::current())?,
                Some(_) => return Err(conflict(format!("rollback worktree collision at {path}"))),
                None => (),
            }
        }
    }
    clean(&txn.repo)?;
    update_journal(journal, "rolled_back", None)?;
    finish_rebase(txn, journal)
}

fn finish_rebase(txn: &GitTransaction, journal: &Path) -> Result<()> {
    crate::effects::Phase::Import(ImportPlan::FinishRebase { txn, journal })
        .execute(Mode::current())
        .map(drop)
}

fn finish_rebase_apply(txn: &GitTransaction, journal: &Path) -> Result<()> {
    clean(&txn.repo)?;
    if let Some(upstream) = &txn.upstream {
        git::pull_rebase(&txn.repo, upstream)?;
    }
    clean(&txn.repo)?;
    update_journal(journal, "retry_ready", None)
}

enum Recovered {
    Completed(Extraction, Vec<String>),
    Retry(Extraction),
}

pub(crate) enum ImportPlan<'a> {
    Commit {
        txn: &'a GitTransaction,
        journal: &'a Path,
        branch: &'a str,
        base: &'a str,
        pairs: &'a [Pair],
    },
    Rollback {
        txn: &'a GitTransaction,
        journal: &'a Path,
        branch: &'a str,
        base: &'a str,
        commit: &'a str,
        pairs: &'a [Pair],
    },
    FinishRollback {
        txn: &'a GitTransaction,
        journal: &'a Path,
        branch: &'a str,
        base: &'a str,
        pairs: &'a [Pair],
    },
    FinishRebase {
        txn: &'a GitTransaction,
        journal: &'a Path,
    },
}

impl ImportPlan<'_> {
    pub(crate) fn summaries(&self) -> Vec<crate::effects::EffectSummary> {
        let (txn, journal, operation) = match self {
            Self::Commit { txn, journal, .. } => (*txn, *journal, "commit-import-journal"),
            Self::Rollback { txn, journal, .. } => (*txn, *journal, "rollback-import-attempt"),
            Self::FinishRollback { txn, journal, .. } => (*txn, *journal, "finish-import-rollback"),
            Self::FinishRebase { txn, journal } => (*txn, *journal, "rebase-import-for-retry"),
        };
        let mut effects = vec![crate::effects::EffectSummary {
            operation,
            paths: vec![journal.into(), txn.repo.clone()],
            unresolved: true,
        }];
        let pairs = match self {
            Self::Commit { pairs, .. }
            | Self::Rollback { pairs, .. }
            | Self::FinishRollback { pairs, .. } => Some(*pairs),
            Self::FinishRebase { .. } => None,
        };
        if let Some(pairs) = pairs {
            effects[0]
                .paths
                .extend(paths(pairs).iter().map(|path| txn.repo.join(path)));
        }
        if matches!(self, Self::Commit { .. }) {
            effects.push(crate::effects::EffectSummary {
                operation: "update-import-journal",
                paths: vec![journal.into()],
                unresolved: true,
            });
            effects.extend(git::commit_support_summaries(
                &txn.repo,
                "chore(tickets): import GitHub issues",
            ));
        }
        effects
    }

    pub(crate) fn apply(self) -> Result<crate::effects::Outcome> {
        match self {
            Self::Commit {
                txn,
                journal,
                branch,
                base,
                pairs,
            } => {
                return commit_pairs_apply(txn, journal, branch, base, pairs)
                    .map(crate::effects::Outcome::ImportCommit)
            }
            Self::Rollback {
                txn,
                journal,
                branch,
                base,
                commit,
                pairs,
            } => rollback_apply(txn, journal, branch, base, commit, pairs)?,
            Self::FinishRollback {
                txn,
                journal,
                branch,
                base,
                pairs,
            } => finish_rollback_apply(txn, journal, branch, base, pairs)?,
            Self::FinishRebase { txn, journal } => finish_rebase_apply(txn, journal)?,
        }
        Ok(crate::effects::Outcome::Applied)
    }
}

fn recover(txn: &GitTransaction, journal: &Path, selector: &Value) -> Result<Option<Recovered>> {
    let Some(text) = read_text(journal)? else {
        return Ok(None);
    };
    if is_dry_run() {
        return Err(conflict("a pending import journal requires reviewed recovery without --dry-run; preview made no changes"));
    }
    let value: Value = serde_json::from_str(&text)
        .map_err(|_| conflict("malformed import journal was preserved"))?;
    if value["selector"] != *selector {
        return Err(conflict("pending import selected a different repository or label; resume that exact request explicitly"));
    }
    no_git_operation(&txn.repo)?;
    let branch = required(&value, "branch")?;
    if git::git(&txn.repo, &["symbolic-ref", "--quiet", "HEAD"])? != branch {
        return Err(conflict("import journal belongs to another branch"));
    }
    let source = &value["source"];
    if serde_json::to_vec(source)?.len() >= github::MAX_BYTES + 1024 {
        return Err(conflict("journal source exceeds the extraction bound"));
    }
    let extraction = github::decode(
        &source["repository"],
        required(selector, "repository")?,
        source["pages"]
            .as_array()
            .ok_or_else(|| conflict("journal lacks source pages"))?,
        selector["label"].as_str(),
    )?;
    let pairs = journal_pairs(&txn.repo, &value)?;
    for pair in &pairs {
        let snapshot: Value = serde_json::from_str(&pair.snapshot)?;
        if !extraction
            .issues
            .iter()
            .any(|issue| issue.id == pair.source_id && issue.raw == snapshot["issue"])
            || snapshot["validation_criteria"] != selector["validation_criteria"]
            || snapshot["repository"]["id"] != extraction.repository.id
        {
            return Err(conflict(
                "journal pair does not belong to its recorded extraction",
            ));
        }
    }
    let base = oid(required(&value, "base")?)?;
    let head = git::git(&txn.repo, &["rev-parse", "HEAD"])?;
    let commit = match required(&value, "phase")? {
        "rolling_back" => {
            if head == base {
                finish_rollback(txn, journal, branch, base, &pairs)?;
            } else {
                rollback(
                    txn,
                    journal,
                    branch,
                    base,
                    oid(required(&value, "commit")?)?,
                    &pairs,
                )?;
            }
            return Ok(Some(Recovered::Retry(extraction)));
        }
        "rolled_back" => {
            finish_rebase(txn, journal)?;
            return Ok(Some(Recovered::Retry(extraction)));
        }
        "retry_ready" => {
            clean(&txn.repo)?;
            return Ok(Some(Recovered::Retry(extraction)));
        }
        "prepared" | "applied" if head == base => commit_pairs(txn, journal, branch, base, &pairs)?,
        "prepared" | "applied" | "committed" => {
            let commit = value["commit"].as_str().unwrap_or(&head);
            verify_commit(&txn.repo, base, oid(commit)?, &pairs)?;
            update_journal(journal, "committed", Some(commit))?;
            commit.to_owned()
        }
        _ => {
            return Err(conflict(
                "unsupported import journal phase; preserved for inspection",
            ))
        }
    };
    match txn.push_once(Some(&commit))? {
        PushAttempt::Done(_) => {
            remove_committed_journal(journal, selector, &commit)?;
            Ok(Some(Recovered::Completed(
                extraction,
                pairs.iter().map(|pair| pair.local_id.clone()).collect(),
            )))
        }
        PushAttempt::Rejected => {
            rollback(txn, journal, branch, base, &commit, &pairs)?;
            Ok(Some(Recovered::Retry(extraction)))
        }
    }
}

fn plan(
    txn: &GitTransaction,
    extraction: &Extraction,
    criteria: &[String],
) -> Result<(Vec<Pair>, usize)> {
    let existing = identity_inventory(txn)?;
    let names = txn.scan_names()?;
    let base = core::max_id(&names)
        .checked_add(1)
        .ok_or_else(|| conflict("ticket ID range exhausted"))?;
    let width = core::id_width(&names);
    let mut pairs = Vec::new();
    let mut skipped = 0;
    for issue in &extraction.issues {
        if let Some(previous) = existing.get(&issue.id) {
            let old: Value = serde_json::from_str(&previous.snapshot)?;
            if old["repository"]["id"] != extraction.repository.id
                || old["issue"]["number"] != issue.number
            {
                return Err(conflict(format!(
                    "source {} moved or changed provenance",
                    issue.id
                )));
            }
            skipped += 1;
            continue;
        }
        let number = base
            .checked_add(pairs.len() as u64)
            .ok_or_else(|| conflict("ticket ID range exhausted"))?;
        let pair = pair(
            &txn.repo,
            &extraction.repository,
            issue,
            &format!("{number:0>width$}"),
            criteria,
        )?;
        for (path, _) in pair.files() {
            if std::fs::symlink_metadata(txn.repo.join(path)).is_ok() {
                return Err(conflict(format!("unmanaged import destination {path}")));
            }
        }
        pairs.push(pair);
    }
    Ok((pairs, skipped))
}

fn validate_criteria(criteria: &[String]) -> Result<()> {
    if criteria.len() > 16 {
        return Err(conflict(
            "import accepts at most 16 shared validation criteria",
        ));
    }
    for criterion in criteria {
        core::validate::validate_free_text(criterion, "validation", 2000).map_err(conflict)?;
        if criterion.trim().is_empty() {
            return Err(conflict("validation criteria must contain text"));
        }
    }
    Ok(())
}

pub fn run(repository: Option<&str>, label: Option<&str>, criteria: &[String]) -> Result<i32> {
    validate_criteria(criteria)?;
    let dir = tickets_dir()?;
    let repo = git::repo_root(&dir)?;
    let name = match repository {
        Some(name) => name.to_owned(),
        None => {
            let origin = git::git(&repo, &["remote", "get-url", "origin"])?;
            origin
                .strip_prefix("https://github.com/")
                .or_else(|| origin.strip_prefix("git@github.com:"))
                .ok_or_else(|| {
                    conflict("origin is not a supported github.com URL; pass --repo OWNER/NAME")
                })?
                .trim_end_matches(".git")
                .to_owned()
        }
    };
    github::repository_name(&name)?;
    if let Some(label) = label {
        core::validate::validate_free_text(label, "label", 200).map_err(conflict)?;
        if label.is_empty() || label.contains(',') {
            return Err(conflict("label must be one nonempty name without commas"));
        }
    }
    let config = super::common::project_config(&dir)?;
    directories(&repo)?;
    let txn = GitTransaction::new_unfetched(&dir)?;
    let journal = metadata_path(&repo, "tkt-github-import.json")?;
    let selector = json!({"repository":name,"label":label,"validation_criteria":criteria});
    let (extraction, recovered) = match recover(&txn, &journal, &selector)? {
        Some(Recovered::Completed(source, ids)) => {
            report(&source, &ids, None, true);
            return Ok(0);
        }
        Some(Recovered::Retry(source)) => (source, true),
        None => (github::extract(&repo, &name, label)?, false),
    };
    txn.fetch()?;
    clean(&repo)?;
    let branch = git::git(&repo, &["symbolic-ref", "--quiet", "HEAD"])?;
    let mut imported = Vec::new();
    let mut skipped = 0;
    for attempt in 0..2 {
        let (pairs, existing) = plan(&txn, &extraction, criteria)?;
        skipped = existing;
        if pairs.is_empty() {
            if is_dry_run() {
                FilePlan(vec![]).execute(Mode::Preview)?;
            }
            if let Some(text) = read_text(&journal)? {
                let old: Value = serde_json::from_str(&text)?;
                if !same_retry(
                    &old,
                    &journal_value(
                        &branch,
                        &selector,
                        &extraction.raw,
                        &git::git(&repo, &["rev-parse", "HEAD"])?,
                        &pairs,
                    ),
                ) {
                    return Err(conflict("cannot clear an unexpected import journal"));
                }
                FilePlan(vec![FileEffect::RemoveOwned {
                    path: journal.clone(),
                    text,
                }])
                .execute(Mode::current())?;
            }
            break;
        }
        if config.close_require_validation_criteria && criteria.is_empty() {
            return Err(conflict("project requires validation criteria at creation; pass repeatable --validation text applied to every new issue, never inferred from source checkboxes"));
        }
        if let Some(namespace) = config.label_namespaces.get("import") {
            if !namespace.values.is_empty()
                && !namespace.values.iter().any(|value| value == "github")
            {
                return Err(conflict(
                    "project label policy excludes the fixed import:github tag",
                ));
            }
        }
        if is_dry_run() {
            let base = git::git(&repo, &["rev-parse", "HEAD"])?;
            start_attempt(
                &journal,
                &journal_value(&branch, &selector, &extraction.raw, &base, &pairs),
            )?;
            apply(&repo, &pairs)?;
            crate::effects::Phase::Import(ImportPlan::Commit {
                txn: &txn,
                journal: &journal,
                branch: &branch,
                base: &base,
                pairs: &pairs,
            })
            .execute(Mode::Preview)?;
            crate::effects::Phase::AllocationPublication(
                crate::transaction::AllocationPlan::Once {
                    transaction: &txn,
                    revision: None,
                },
            )
            .execute(Mode::Preview)?;
            imported = pairs.iter().map(|pair| pair.local_id.clone()).collect();
            break;
        }
        let base = git::git(&repo, &["rev-parse", "HEAD"])?;
        start_attempt(
            &journal,
            &journal_value(&branch, &selector, &extraction.raw, &base, &pairs),
        )?;
        let commit = commit_pairs(&txn, &journal, &branch, &base, &pairs)?;
        match txn.push_once(Some(commit.as_ref()))? {
            PushAttempt::Done(_) => {
                imported = pairs.iter().map(|pair| pair.local_id.clone()).collect();
                remove_committed_journal(&journal, &selector, commit.as_ref())?;
                break;
            }
            PushAttempt::Rejected if attempt == 0 => {
                rollback(&txn, &journal, &branch, &base, &commit, &pairs)?
            }
            PushAttempt::Rejected => {
                return Err(git::local_commit_push_error(
                    &repo,
                    "import push was rejected twice; verified local commit and journal retained",
                ))
            }
        }
    }
    report(&extraction, &imported, Some(skipped), recovered);
    Ok(0)
}

fn report(source: &Extraction, imported: &[String], skipped: Option<usize>, recovered: bool) {
    if is_json_output() {
        println!(
            "{}",
            json!({"ok":true,"operation":"import","provider":"github","repository":source.repository.name,"imported":imported,"skipped_existing":skipped,"skipped_closed":source.closed,"skipped_pull_requests":source.pull_requests,"skipped_duplicates":source.duplicates,"skipped_label":source.filtered,"received_objects":source.received,"dry_run":is_dry_run(),"recovered":recovered,"updates_supported":false})
        );
    } else if !is_quiet() {
        println!("{} {} {} open GitHub issues from {}; existing count {}, {} closed, {} pull requests and {} repeated objects skipped",crate::color::sym_ok(),if recovered {"recovered"}else if is_dry_run(){"would import"}else{"imported"},imported.len(),source.repository.name,skipped.map(|n|n.to_string()).unwrap_or_else(||"not recomputed during recovery".into()),source.closed,source.pull_requests,source.duplicates);
    }
}

#[cfg(test)]
mod phase_tests {
    use super::*;

    #[test]
    fn import_commit_and_recovery_previews_include_exact_owned_pair_paths() {
        use crate::effects::{Outcome, Phase};
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let init = Command::new("git")
            .args(["init", "-q", "-b", "main"])
            .arg(&root)
            .env("DO_NOT_TRACK", "1")
            .output()
            .unwrap();
        assert!(init.status.success(), "{init:?}");
        let dir = root.join(".tickets");
        std::fs::create_dir(&dir).unwrap();
        std::fs::write(
            dir.join("config.toml"),
            "[push]\nenabled=false\n[commit]\nhooks=\"skip\"\n",
        )
        .unwrap();
        let txn = GitTransaction::new_unfetched(&dir).unwrap();
        let journal = root.join(".git/tkt-github-import.json");
        let pairs = vec![Pair {
            source_id: 101,
            local_id: "05".into(),
            ticket_path: ".tickets/05-import-fixture.md".into(),
            ticket: "private ticket bytes".into(),
            snapshot_path: ".tickets/imports/github/77-101.json".into(),
            snapshot: "private snapshot bytes".into(),
        }];
        let marker = root.join(".git/tkt-allocation.lock");
        let before_marker = std::fs::read(&marker).unwrap();
        for (plan, operation, owns_pairs) in [
            (
                ImportPlan::Commit {
                    txn: &txn,
                    journal: &journal,
                    branch: "refs/heads/main",
                    base: "unresolved",
                    pairs: &pairs,
                },
                "commit-import-journal",
                true,
            ),
            (
                ImportPlan::Rollback {
                    txn: &txn,
                    journal: &journal,
                    branch: "refs/heads/main",
                    base: "unresolved",
                    commit: "unresolved",
                    pairs: &pairs,
                },
                "rollback-import-attempt",
                true,
            ),
            (
                ImportPlan::FinishRollback {
                    txn: &txn,
                    journal: &journal,
                    branch: "refs/heads/main",
                    base: "unresolved",
                    pairs: &pairs,
                },
                "finish-import-rollback",
                true,
            ),
            (
                ImportPlan::FinishRebase {
                    txn: &txn,
                    journal: &journal,
                },
                "rebase-import-for-retry",
                false,
            ),
        ] {
            let Outcome::Preview(effects) = Phase::Import(plan).execute(Mode::Preview).unwrap()
            else {
                panic!("preview applied recovery effects");
            };
            assert_eq!(effects[0].operation, operation);
            assert!(effects[0].unresolved);
            let mut expected: std::collections::BTreeSet<_> =
                [journal.clone(), root.to_path_buf()].into_iter().collect();
            if owns_pairs {
                expected.extend(paths(&pairs).iter().map(|path| root.join(path)));
            }
            assert_eq!(
                effects[0]
                    .paths
                    .iter()
                    .cloned()
                    .collect::<std::collections::BTreeSet<_>>(),
                expected
            );
            assert_eq!(std::fs::read(&marker).unwrap(), before_marker);
            assert!(!journal.exists());
            assert!(!root.join(&pairs[0].ticket_path).exists());
            assert!(!root.join(&pairs[0].snapshot_path).exists());
        }
    }
}

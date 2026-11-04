//! Current completed tickets with dates attributable to stored Git transitions.

use std::collections::{BTreeMap, VecDeque};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use serde_json::json;

use super::common::{is_json_output, is_quiet, tickets_dir};
use crate::core::{self, Status, Ticket};
use crate::{git, DomainError, ErrorKind};

const HISTORY_LOOKUPS: usize = 4096;
const PATH_COMMITS: usize = 256;
const OBJECT_BYTES: usize = 32 * 1024 * 1024;
const LIST_BYTES: usize = 8 * 1024 * 1024;
const READ_BYTES: usize = 64 * 1024 * 1024;
const TREE_ENTRIES: usize = 4096;
const CACHE_BYTES: usize = 16 * 1024 * 1024;
const CACHE_ENTRIES: usize = 8192;
const WEEK_SECONDS: i64 = 604_800;
const MIN_TIME: i64 = -62_135_596_800;
const MAX_TIME: i64 = 253_402_300_799;

#[derive(Clone, Copy)]
enum Unknown {
    MissingHead,
    Shallow,
    Unavailable,
    Budget,
    Limit,
    MissingFile,
    Uncommitted,
    Identity,
    Duplicate,
    InvalidHistory,
    Merge,
    InitialDone,
    PathBoundary,
    NoTransition,
    Timestamp,
    ObjectLimit,
    ListingLimit,
    CacheLimit,
    ReadLimit,
}

impl Unknown {
    fn label(self) -> &'static str {
        match self {
            Self::MissingHead => "missing_or_unreadable_head",
            Self::Shallow => "shallow_history",
            Self::Unavailable => "history_unavailable",
            Self::Budget => "report_history_budget",
            Self::Limit => "ticket_history_limit",
            Self::MissingFile => "untracked_or_missing_committed_file",
            Self::Uncommitted => "uncommitted_done_transition",
            Self::Identity => "identity_discontinuity",
            Self::Duplicate => "duplicate_current_identity",
            Self::InvalidHistory => "unparseable_history",
            Self::Merge => "merge_history",
            Self::InitialDone => "initial_done_without_transition",
            Self::PathBoundary => "path_boundary_or_rename",
            Self::NoTransition => "no_verified_transition",
            Self::Timestamp => "invalid_or_future_commit_time",
            Self::ObjectLimit => "history_object_limit",
            Self::ListingLimit => "history_listing_limit",
            Self::CacheLimit => "history_cache_limit",
            Self::ReadLimit => "history_read_limit",
        }
    }
}

#[derive(Clone)]
struct StoredState {
    id: String,
    status: Status,
}

#[derive(Clone)]
enum Parents {
    Root,
    Single(String),
    Merge,
}

#[derive(Clone)]
struct Commit {
    parents: Parents,
    unix: i64,
}

enum Completion {
    Known { unix: i64, commit: String },
    Unknown(Unknown),
}

enum History {
    Unavailable(Unknown),
    Ready(Box<Reader>),
}

struct Reader {
    repo: PathBuf,
    head: String,
    remaining: usize,
    paths: BTreeMap<(String, String), Option<StoredState>>,
    blobs: BTreeMap<String, StoredState>,
    commits: BTreeMap<String, Commit>,
    trees: BTreeMap<String, TicketTree>,
    tree_failures: BTreeMap<String, Unknown>,
    tree_bytes: usize,
    tree_entries: usize,
    tree_order: VecDeque<String>,
    objects: Option<Objects>,
    metadata_bytes: usize,
    metadata_entries: usize,
    remaining_bytes: usize,
}

struct TicketTree {
    entries: BTreeMap<String, std::result::Result<String, Unknown>>,
    bytes: usize,
}

struct Objects {
    child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
}

impl Objects {
    fn open(repo: &Path) -> std::result::Result<Self, Unknown> {
        let mut child = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args([
                "--no-optional-locks",
                "--no-replace-objects",
                "--no-lazy-fetch",
                "cat-file",
                "--batch",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| Unknown::Unavailable)?;
        let input = child.stdin.take().expect("piped stdin");
        let output = BufReader::new(child.stdout.take().expect("piped stdout"));
        Ok(Self {
            child,
            input,
            output,
        })
    }

    fn read(
        &mut self,
        id: &str,
        kind: &str,
        remaining_bytes: &mut usize,
    ) -> std::result::Result<String, Unknown> {
        writeln!(self.input, "{id}")
            .and_then(|_| self.input.flush())
            .map_err(|_| Unknown::Unavailable)?;
        let mut header = Vec::new();
        (&mut self.output)
            .take(256)
            .read_until(b'\n', &mut header)
            .map_err(|_| Unknown::Unavailable)?;
        if header.last() != Some(&b'\n') {
            return Err(Unknown::InvalidHistory);
        }
        let header = std::str::from_utf8(&header).map_err(|_| Unknown::InvalidHistory)?;
        let fields: Vec<_> = header.split_whitespace().collect();
        if fields.len() == 2 && fields[1] == "missing" {
            return Err(Unknown::Unavailable);
        }
        if fields.len() != 3 || fields[0] != id || fields[1] != kind {
            return Err(Unknown::InvalidHistory);
        }
        let size = fields[2]
            .parse::<usize>()
            .map_err(|_| Unknown::InvalidHistory)?;
        if size > OBJECT_BYTES {
            return Err(Unknown::ObjectLimit);
        }
        if size > *remaining_bytes {
            return Err(Unknown::ReadLimit);
        }
        *remaining_bytes -= size;
        let mut bytes = vec![0; size];
        let mut end = [0];
        self.output
            .read_exact(&mut bytes)
            .and_then(|_| self.output.read_exact(&mut end))
            .map_err(|_| Unknown::Unavailable)?;
        if end != [b'\n'] {
            return Err(Unknown::InvalidHistory);
        }
        String::from_utf8(bytes).map_err(|_| Unknown::InvalidHistory)
    }
}

impl Drop for Objects {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn oid(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

impl History {
    fn open(repo: &Path) -> Self {
        let Ok(head) = git::git(
            repo,
            &[
                "--no-replace-objects",
                "--no-lazy-fetch",
                "rev-parse",
                "--verify",
                "HEAD",
            ],
        ) else {
            return Self::Unavailable(Unknown::MissingHead);
        };
        if !oid(&head) {
            return Self::Unavailable(Unknown::InvalidHistory);
        }
        match git::git(repo, &["rev-parse", "--is-shallow-repository"]).as_deref() {
            Ok("true") => Self::Unavailable(Unknown::Shallow),
            Ok("false") => Self::Ready(Box::new(Reader {
                repo: repo.into(),
                head,
                remaining: HISTORY_LOOKUPS,
                paths: BTreeMap::new(),
                blobs: BTreeMap::new(),
                commits: BTreeMap::new(),
                trees: BTreeMap::new(),
                tree_failures: BTreeMap::new(),
                tree_bytes: 0,
                tree_entries: 0,
                tree_order: VecDeque::new(),
                objects: None,
                metadata_bytes: 0,
                metadata_entries: 0,
                remaining_bytes: READ_BYTES,
            })),
            _ => Self::Unavailable(Unknown::Unavailable),
        }
    }

    fn completion(&mut self, ticket: &Ticket, now: i64) -> Completion {
        let result = match self {
            Self::Unavailable(reason) => Err(*reason),
            Self::Ready(reader) => reader.completion(ticket, now),
        };
        match result {
            Ok((unix, commit)) => Completion::Known { unix, commit },
            Err(reason) => Completion::Unknown(reason),
        }
    }
}

impl Reader {
    fn reserve_metadata(&mut self, bytes: usize) -> std::result::Result<(), Unknown> {
        if self.metadata_bytes + bytes > CACHE_BYTES || self.metadata_entries >= CACHE_ENTRIES {
            return Err(Unknown::CacheLimit);
        }
        self.metadata_bytes += bytes;
        self.metadata_entries += 1;
        Ok(())
    }

    fn object(&mut self, id: &str, kind: &str) -> std::result::Result<String, Unknown> {
        if self.remaining == 0 {
            return Err(Unknown::Budget);
        }
        self.remaining -= 1;
        if self.objects.is_none() {
            self.objects = Some(Objects::open(&self.repo)?);
        }
        let result = self
            .objects
            .as_mut()
            .expect("initialized object reader")
            .read(id, kind, &mut self.remaining_bytes);
        if result.is_err() {
            self.objects = None;
        }
        result
    }

    fn tree_entry(
        &mut self,
        commit: &str,
        path: &str,
    ) -> std::result::Result<Option<String>, Unknown> {
        if let Some(reason) = self.tree_failures.get(commit) {
            return Err(*reason);
        }
        if !self.trees.contains_key(commit) {
            let prepared = (|| -> std::result::Result<TicketTree, Unknown> {
                let raw = self.query(&["ls-tree", "-z", commit, "--", ".tickets/"])?;
                if !raw.is_empty() && !raw.ends_with('\0') {
                    return Err(Unknown::InvalidHistory);
                }
                let mut entries = BTreeMap::new();
                let mut bytes = 0;
                for entry in raw.split('\0').filter(|entry| !entry.is_empty()) {
                    let (metadata, name) = entry.split_once('\t').ok_or(Unknown::InvalidHistory)?;
                    if !name.starts_with(".tickets/")
                        || !name.ends_with(".md")
                        || name[9..].contains('/')
                    {
                        continue;
                    }
                    if entries.len() >= TREE_ENTRIES {
                        return Err(Unknown::ListingLimit);
                    }
                    let fields: Vec<_> = metadata.split_whitespace().collect();
                    let value = if fields.len() == 3
                        && matches!(fields[0], "100644" | "100755")
                        && fields[1] == "blob"
                        && oid(fields[2])
                    {
                        Ok(fields[2].to_owned())
                    } else {
                        Err(Unknown::InvalidHistory)
                    };
                    bytes += name.len() + value.as_ref().map_or(0, String::len);
                    if bytes > CACHE_BYTES {
                        return Err(Unknown::ListingLimit);
                    }
                    if entries.insert(name.to_owned(), value).is_some() {
                        return Err(Unknown::InvalidHistory);
                    }
                }
                Ok(TicketTree { entries, bytes })
            })();
            let tree = match prepared {
                Ok(tree) => tree,
                Err(reason) => {
                    self.tree_failures.insert(commit.into(), reason);
                    return Err(reason);
                }
            };
            while self.tree_order.len() >= 8
                || self.tree_bytes + tree.bytes > CACHE_BYTES
                || self.tree_entries + tree.entries.len() > CACHE_ENTRIES
            {
                let Some(oldest) = self.tree_order.pop_front() else {
                    return Err(Unknown::ListingLimit);
                };
                let old = self.trees.remove(&oldest).expect("cached historical tree");
                self.tree_bytes -= old.bytes;
                self.tree_entries -= old.entries.len();
            }
            self.tree_bytes += tree.bytes;
            self.tree_entries += tree.entries.len();
            if commit != self.head {
                self.tree_order.push_back(commit.into());
            }
            self.trees.insert(commit.into(), tree);
        }
        self.trees[commit].entries.get(path).cloned().transpose()
    }

    fn query(&mut self, args: &[&str]) -> std::result::Result<String, Unknown> {
        if self.remaining == 0 {
            return Err(Unknown::Budget);
        }
        if self.remaining_bytes == 0 {
            return Err(Unknown::ReadLimit);
        }
        self.remaining -= 1;
        let limit = LIST_BYTES.min(self.remaining_bytes);
        let mut command = vec![
            "--no-pager",
            "--no-optional-locks",
            "--no-replace-objects",
            "--no-lazy-fetch",
            "--literal-pathspecs",
        ];
        command.extend_from_slice(args);
        let mut child = Command::new("git")
            .arg("-C")
            .arg(&self.repo)
            .args(&command)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| Unknown::Unavailable)?;
        let mut bytes = Vec::new();
        let read = child
            .stdout
            .take()
            .expect("piped stdout")
            .take(limit as u64 + 1)
            .read_to_end(&mut bytes);
        self.remaining_bytes -= bytes.len().min(limit);
        if read.is_err() || bytes.len() > limit {
            let _ = child.kill();
            let _ = child.wait();
            return Err(if bytes.len() > limit {
                if limit < LIST_BYTES {
                    Unknown::ReadLimit
                } else {
                    Unknown::ListingLimit
                }
            } else {
                Unknown::Unavailable
            });
        }
        if !child.wait().map_err(|_| Unknown::Unavailable)?.success() {
            return Err(Unknown::Unavailable);
        }
        String::from_utf8(bytes).map_err(|_| Unknown::InvalidHistory)
    }

    fn state(
        &mut self,
        commit: &str,
        path: &str,
    ) -> std::result::Result<Option<StoredState>, Unknown> {
        let key = (commit.to_owned(), path.to_owned());
        if let Some(state) = self.paths.get(&key) {
            return Ok(state.clone());
        }
        let Some(blob) = self.tree_entry(commit, path)? else {
            return Ok(None);
        };
        let state = if let Some(state) = self.blobs.get(&blob) {
            state.clone()
        } else {
            let text = self.object(&blob, "blob")?;
            let parsed = Ticket::parse_str(&text, &self.repo.join(path))
                .map_err(|_| Unknown::InvalidHistory)?;
            let state = StoredState {
                id: parsed.id,
                status: parsed.status,
            };
            self.reserve_metadata(blob.len() + state.id.len())?;
            self.blobs.insert(blob, state.clone());
            state
        };
        self.reserve_metadata(key.0.len() + key.1.len() + state.id.len())?;
        self.paths.insert(key, Some(state.clone()));
        Ok(Some(state))
    }

    fn commit(&mut self, commit: &str) -> std::result::Result<Commit, Unknown> {
        if let Some(record) = self.commits.get(commit) {
            return Ok(record.clone());
        }
        let raw = self.object(commit, "commit")?;
        let header = raw.split_once("\n\n").ok_or(Unknown::InvalidHistory)?.0;
        let mut parents = Parents::Root;
        let mut unix = None;
        for line in header.lines() {
            if let Some(parent) = line.strip_prefix("parent ") {
                if !oid(parent) {
                    return Err(Unknown::InvalidHistory);
                }
                parents = match parents {
                    Parents::Root => Parents::Single(parent.into()),
                    _ => Parents::Merge,
                };
            }
            if let Some(committer) = line.strip_prefix("committer ") {
                if unix.is_some() {
                    return Err(Unknown::InvalidHistory);
                }
                unix = committer
                    .rsplit(' ')
                    .nth(1)
                    .and_then(|value| value.parse::<i64>().ok());
                if unix.is_none() {
                    return Err(Unknown::InvalidHistory);
                }
            }
        }
        let record = Commit {
            parents,
            unix: unix.ok_or(Unknown::InvalidHistory)?,
        };
        self.reserve_metadata(
            commit.len()
                + match &record.parents {
                    Parents::Single(parent) => parent.len(),
                    _ => 0,
                },
        )?;
        self.commits.insert(commit.into(), record.clone());
        Ok(record)
    }

    fn completion(
        &mut self,
        ticket: &Ticket,
        now: i64,
    ) -> std::result::Result<(i64, String), Unknown> {
        let path = ticket
            .path
            .strip_prefix(&self.repo)
            .map_err(|_| Unknown::InvalidHistory)?
            .to_string_lossy()
            .replace('\\', "/");
        let head = self.head.clone();
        let current = self.state(&head, &path)?.ok_or(Unknown::MissingFile)?;
        if current.id != ticket.id {
            return Err(Unknown::Identity);
        }
        if current.status != Status::Done {
            return Err(Unknown::Uncommitted);
        }
        let tip = self.commit(&head)?;
        match &tip.parents {
            Parents::Root => return Err(Unknown::InitialDone),
            Parents::Single(parent) => {
                let before = self.state(parent, &path)?.ok_or(Unknown::PathBoundary)?;
                if before.id != ticket.id {
                    return Err(Unknown::Identity);
                }
                if before.status != Status::Done {
                    if !valid_time(tip.unix, now) {
                        return Err(Unknown::Timestamp);
                    }
                    return Ok((tip.unix, head));
                }
            }
            _ => (),
        }
        let limit = format!("--max-count={}", PATH_COMMITS + 1);
        let log = self.query(&[
            "log",
            "--first-parent",
            "--no-show-signature",
            &limit,
            "--format=%H",
            &head,
            "--",
            &path,
        ])?;
        let records: Vec<_> = log.lines().collect();
        for candidate in records.iter().take(PATH_COMMITS) {
            if !oid(candidate) {
                return Err(Unknown::InvalidHistory);
            }
            let record = self.commit(candidate)?;
            let parent = match &record.parents {
                Parents::Root => return Err(Unknown::InitialDone),
                Parents::Single(parent) => parent,
                _ => return Err(Unknown::Merge),
            };
            let after = self.state(candidate, &path)?.ok_or(Unknown::PathBoundary)?;
            let before = self.state(parent, &path)?.ok_or(Unknown::PathBoundary)?;
            if after.id != ticket.id || before.id != ticket.id {
                return Err(Unknown::Identity);
            }
            if after.status != Status::Done {
                return Err(Unknown::InvalidHistory);
            }
            if before.status != Status::Done {
                if !valid_time(record.unix, now) {
                    return Err(Unknown::Timestamp);
                }
                return Ok((record.unix, (*candidate).into()));
            }
        }
        Err(if records.len() > PATH_COMMITS {
            Unknown::Limit
        } else {
            Unknown::NoTransition
        })
    }
}

fn valid_time(unix: i64, now: i64) -> bool {
    (MIN_TIME..=MAX_TIME).contains(&unix) && unix <= now
}

fn timestamp(unix: i64) -> String {
    let seconds = unix.rem_euclid(86400);
    format!(
        "{}T{:02}:{:02}:{:02}Z",
        super::close::utc_date(unix),
        seconds / 3600,
        seconds % 3600 / 60,
        seconds % 60
    )
}

fn in_window(unix: i64, now: i64) -> bool {
    (now - WEEK_SECONDS..=now).contains(&unix)
}

pub fn run(week: bool, all_context: bool) -> Result<i32> {
    let dir = tickets_dir()?;
    let repo = git::repo_root(&dir)?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|value| i64::try_from(value.as_secs()).ok())
        .filter(|value| *value <= MAX_TIME)
        .ok_or_else(|| {
            DomainError::new(
                ErrorKind::Io,
                "system clock is outside the supported UTC calendar".into(),
            )
        })?;
    let corpus = core::load_corpus(&dir)?;
    let context = crate::context::load(&dir);
    let mut identities = BTreeMap::new();
    for ticket in &corpus {
        *identities.entry(ticket.id.as_str()).or_insert(0usize) += 1;
    }
    let mut history = History::open(&repo);
    let mut rows = Vec::new();
    let mut hidden = 0;
    for ticket in corpus.iter().filter(|ticket| ticket.status == Status::Done) {
        if !all_context && !context.matches(&ticket.tags) {
            hidden += 1;
            continue;
        }
        let completion = if identities[ticket.id.as_str()] > 1 {
            Completion::Unknown(Unknown::Duplicate)
        } else {
            history.completion(ticket, now)
        };
        if week && matches!(&completion,Completion::Known{unix,..} if !in_window(*unix,now)) {
            continue;
        }
        rows.push((ticket, completion));
    }
    rows.sort_by(|(a, ad), (b, bd)| {
        let date = match (ad, bd) {
            (Completion::Known { unix: a, .. }, Completion::Known { unix: b, .. }) => b.cmp(a),
            (Completion::Known { .. }, _) => std::cmp::Ordering::Less,
            (_, Completion::Known { .. }) => std::cmp::Ordering::Greater,
            _ => std::cmp::Ordering::Equal,
        };
        date.then_with(|| {
            a.id.parse::<u64>()
                .unwrap_or(u64::MAX)
                .cmp(&b.id.parse::<u64>().unwrap_or(u64::MAX))
        })
        .then_with(|| a.id.cmp(&b.id))
    });
    if hidden > 0 {
        if is_json_output() {
            eprintln!(
                "{}",
                json!({"warning":{"kind":"context_narrowing","hidden_done":hidden,"message":"tag context narrows this report; use --all-context for the full corpus"}})
            );
        } else {
            eprintln!(
                "Tag context hides {hidden} done tickets; use --all-context for the full corpus"
            );
        }
    }
    crate::RESULT_COUNT.store(rows.len() as i32, std::sync::atomic::Ordering::Relaxed);
    let unknown_dates = rows
        .iter()
        .filter(|(_, date)| matches!(date, Completion::Unknown(_)))
        .count();
    let dated = rows.len() - unknown_dates;
    let dated_in_window = rows
        .iter()
        .filter(|(_, date)| matches!(date,Completion::Known{unix,..} if in_window(*unix,now)))
        .count();
    if is_json_output() {
        let values:Vec<_>=rows.iter().map(|(ticket,date)|{
            let mut value=json!({"id":ticket.id,"title":ticket.title,"status":"done","closed_at":null,"closed_unix":null,"closure_commit":null,"unknown_reason":null,"in_window":null});
            match date {Completion::Known{unix,commit}=>{value["closed_at"]=json!(timestamp(*unix));value["closed_unix"]=json!(unix);value["closure_commit"]=json!(commit);value["in_window"]=json!(in_window(*unix,now));},Completion::Unknown(reason)=>value["unknown_reason"]=json!(reason.label())};value
        }).collect();
        println!(
            "{}",
            json!({"ok":true,"operation":"done","filter":if week{"week"}else{"all"},"timezone":"UTC","date_source":"git_committer_transition","window":{"since_unix":now-WEEK_SECONDS,"until_unix":now,"duration_seconds":WEEK_SECONDS,"inclusive":true},"rows":values,"count":rows.len(),"dated_count":dated,"dated_in_window_count":dated_in_window,"unknown_date_count":unknown_dates,"history_limits":{"lookups_per_report":HISTORY_LOOKUPS,"path_commits_per_ticket":PATH_COMMITS,"object_bytes":OBJECT_BYTES,"listing_bytes":LIST_BYTES,"tree_entries":TREE_ENTRIES,"cached_tree_bytes":CACHE_BYTES,"cached_tree_entries":CACHE_ENTRIES,"cached_metadata_bytes":CACHE_BYTES,"cached_metadata_entries":CACHE_ENTRIES,"read_bytes":READ_BYTES}})
        );
    } else if !is_quiet() {
        println!(
            "Completed tickets — {} (UTC Git transitions)",
            if week {
                format!(
                    "{} through {}, inclusive",
                    timestamp(now - WEEK_SECONDS),
                    timestamp(now)
                )
            } else {
                "all dates".into()
            }
        );
        for (ticket, date) in &rows {
            let date = match date {
                Completion::Known { unix, .. } => timestamp(*unix),
                Completion::Unknown(reason) => format!("unknown ({})", reason.label()),
            };
            println!(
                "{date}\t{}\t{}",
                single_line(&ticket.id),
                single_line(&ticket.title)
            );
        }
        println!("{} current done tickets; {dated} dated, {unknown_dates} dates unknown; cancelled work is excluded", rows.len());
    }
    Ok(0)
}

fn single_line(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_control() {
                ' '
            } else {
                character
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn rolling_week_includes_exact_endpoints_and_excludes_adjacent_seconds() {
        let now = 1_800_000_000;
        assert!(super::in_window(now, now));
        assert!(super::in_window(now - super::WEEK_SECONDS, now));
        assert!(!super::in_window(now + 1, now));
        assert!(!super::in_window(now - super::WEEK_SECONDS - 1, now));
    }
}

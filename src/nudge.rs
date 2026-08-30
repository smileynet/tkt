//! Advisory batch nudge: after several consecutive `tkt new` calls that share a
//! tag or blocker, suggest `tkt batch` — which collapses N commits/pushes into
//! one. Modeled on git's `advice.*` system: advisory only, never changes exit
//! code or stdout, never auto-runs, and is trivially opt-out.
//!
//! State lives in `.git/tkt-cadence.jsonl` (per-repo, uncommitted, ephemeral).
//! Records are pruned to a short time window so unrelated sessions don't chain.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Consecutive `new` calls sharing an attribute needed before nudging.
const NUDGE_THRESHOLD: usize = 3;
/// Records older than this (seconds) are pruned — a stale gap is not a burst.
const WINDOW_SECS: u64 = 120;

/// A one-line advisory. `code` is a stable machine token; `message` is human
/// prose; `suggested_command` is copy-pasteable; `disable` documents opt-out.
pub struct Hint {
    pub code: &'static str,
    pub message: String,
    pub suggested_command: String,
    pub disable: &'static str,
}

impl Hint {
    /// JSON object for the `hints[]` array in the success envelope.
    pub fn to_json(&self) -> String {
        format!(
            "{{\"code\":{},\"message\":{},\"suggested_command\":{},\"disable\":{}}}",
            crate::cli::json_escape(self.code),
            crate::cli::json_escape(&self.message),
            crate::cli::json_escape(&self.suggested_command),
            crate::cli::json_escape(self.disable),
        )
    }
}

/// True when the caller opted out of advisories via env (CI/subprocess friendly).
pub fn advice_disabled() -> bool {
    matches!(
        std::env::var("TKT_ADVICE").ok().as_deref(),
        Some("0") | Some("off") | Some("false")
    )
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn state_path(repo_root: &Path) -> PathBuf {
    // Prefer the repo's .git directory (per-repo, uncommitted). In a worktree
    // .git is a file, not a dir — fall back to the system temp dir keyed by a
    // hash of the repo path so unrelated repos don't share cadence state.
    let git = repo_root.join(".git");
    if git.is_dir() {
        return git.join("tkt-cadence.jsonl");
    }
    let mut key: u64 = 1469598103934665603; // FNV-1a offset basis
    for b in repo_root.to_string_lossy().as_bytes() {
        key ^= *b as u64;
        key = key.wrapping_mul(1099511628211);
    }
    std::env::temp_dir().join(format!("tkt-cadence-{:016x}.jsonl", key))
}

/// One recorded `new` invocation: when, and its shared-scope attributes.
struct Record {
    ts: u64,
    tags: Vec<String>,
    blocked_by: Vec<String>,
}

fn parse_line(line: &str) -> Option<Record> {
    let v: serde_json::Value = serde_json::from_str(line.trim()).ok()?;
    let ts = v.get("ts")?.as_u64()?;
    let tags = v
        .get("tags")
        .and_then(|t| t.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    let blocked_by = v
        .get("blocked_by")
        .and_then(|t| t.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    Some(Record {
        ts,
        tags,
        blocked_by,
    })
}

fn record_json(ts: u64, tags: &[String], blocked_by: &[String]) -> String {
    let arr = |items: &[String]| -> String {
        let parts: Vec<String> = items.iter().map(|s| crate::cli::json_escape(s)).collect();
        format!("[{}]", parts.join(","))
    };
    format!(
        "{{\"ts\":{},\"tags\":{},\"blocked_by\":{}}}",
        ts,
        arr(tags),
        arr(blocked_by)
    )
}

fn shares_attr(a: &Record, tags: &[String], blocked_by: &[String]) -> bool {
    tags.iter().any(|t| a.tags.contains(t)) || blocked_by.iter().any(|b| a.blocked_by.contains(b))
}

/// Record this `new` call and, if it completes a burst of `NUDGE_THRESHOLD`
/// recent `new` calls that all share a tag or blocker, return a batch Hint.
///
/// Never errors out of the caller: state I/O failures degrade to "no nudge".
pub fn record_and_check(repo_root: &Path, tags: &[String], blocked_by: &[String]) -> Option<Hint> {
    if advice_disabled() {
        // Still keep state consistent-free: skip entirely when opted out.
        return None;
    }
    let path = state_path(repo_root);
    let now = now_secs();

    // Load recent records within the window.
    let mut recent: Vec<Record> = std::fs::read_to_string(&path)
        .ok()
        .map(|s| {
            s.lines()
                .filter_map(parse_line)
                .filter(|r| now.saturating_sub(r.ts) <= WINDOW_SECS)
                .collect()
        })
        .unwrap_or_default();

    // Append the current call.
    recent.push(Record {
        ts: now,
        tags: tags.to_vec(),
        blocked_by: blocked_by.to_vec(),
    });

    // Rewrite the pruned window (best-effort).
    let _ = write_state(&path, &recent);

    // Only nudge when this call has a shared attribute to batch on.
    if tags.is_empty() && blocked_by.is_empty() {
        return None;
    }

    // Count recent calls (including this one) that share an attribute with it.
    let sharing = recent
        .iter()
        .filter(|r| shares_attr(r, tags, blocked_by))
        .count();

    if sharing >= NUDGE_THRESHOLD {
        let scope = if !tags.is_empty() {
            format!("--tags {}", tags.join(","))
        } else {
            format!("--blocked-by {}", blocked_by.join(","))
        };
        Some(Hint {
            code: "prefer-batch",
            message: format!(
                "{} consecutive `tkt new` calls share {} — `tkt batch` creates them in one commit/push",
                sharing, scope
            ),
            suggested_command: format!("tkt batch \"slug:title\" \"slug:title\" ... {}", scope),
            disable: "set TKT_ADVICE=0 to silence, or use -q",
        })
    } else {
        None
    }
}

fn write_state(path: &Path, records: &[Record]) -> std::io::Result<()> {
    let mut buf = String::new();
    for r in records {
        buf.push_str(&record_json(r.ts, &r.tags, &r.blocked_by));
        buf.push('\n');
    }
    let mut f = std::fs::File::create(path)?;
    f.write_all(buf.as_bytes())
}

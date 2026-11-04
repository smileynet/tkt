use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

fn command(program: &str) -> Command {
    let mut command = Command::new(program);
    command
        .env("DO_NOT_TRACK", "1")
        .env("TKT_NO_USER_CONFIG", "1")
        .env("TKT_UPDATE_CHECK", "0")
        .env("TKT_DEBUG", "0")
        .env(
            "GIT_CONFIG_GLOBAL",
            if cfg!(windows) { "NUL" } else { "/dev/null" },
        )
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_COUNT", "0")
        .env_remove("GIT_CONFIG_PARAMETERS")
        .env_remove("TKT_CONTEXT")
        .env_remove("TKT_AGENT");
    command
}
fn git(root: &Path, args: &[&str]) -> String {
    let output = command("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .unwrap();
    assert!(output.status.success(), "{args:?}: {output:?}");
    String::from_utf8(output.stdout).unwrap().trim().into()
}
fn text(id: &str, status: &str, body: &str, tag: &str) -> String {
    format!("---\nid: \"{id}\"\ntitle: \"Work {id}\"\nstatus: {status}\nblocked_by: []\ntags: [\"{tag}\"]\nassignee: \"other-agent\"\nclosed_at: \"2099-01-01\"\n---\n\n# Work\n\n{body}\n")
}
struct Fixture {
    _temp: tempfile::TempDir,
    root: PathBuf,
    now: i64,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::Builder::new()
            .prefix("tkt-done-report-")
            .tempdir()
            .unwrap();
        let root = temp.path().join("repo");
        std::fs::create_dir(&root).unwrap();
        git(&root, &["init", "-q", "-b", "main"]);
        git(&root, &["config", "user.name", "Report fixture"]);
        git(&root, &["config", "user.email", "fixture@example.invalid"]);
        git(&root, &["config", "core.autocrlf", "false"]);
        std::fs::create_dir(root.join(".tickets")).unwrap();
        std::fs::write(root.join(".tickets/config.toml"), "[push]\nenabled=false\n").unwrap();
        std::fs::write(root.join("README.md"), "Owner source\n").unwrap();
        Self {
            _temp: temp,
            root,
            now: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        }
    }
    fn state(&self, id: &str, status: &str, body: &str) {
        std::fs::write(
            self.root.join(format!(".tickets/{id}-work.md")),
            text(id, status, body, "stream:alpha"),
        )
        .unwrap();
    }
    fn commit(&self, unix: i64, zone: &str) -> String {
        git(&self.root, &["add", "--", ".tickets", "README.md"]);
        let date = format!("@{unix} {zone}");
        let output = command("git")
            .arg("-C")
            .arg(&self.root)
            .args(["commit", "-qm", "fixture transition"])
            .env("GIT_AUTHOR_DATE", &date)
            .env("GIT_COMMITTER_DATE", &date)
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        git(&self.root, &["rev-parse", "HEAD"])
    }
    fn cli(&self, args: &[&str]) -> Command {
        let mut command = command(env!("CARGO_BIN_EXE_tkt"));
        command
            .current_dir(&self.root)
            .args(args)
            .env("XDG_CONFIG_HOME", self.root.join("private-config"))
            .env("XDG_STATE_HOME", self.root.join("private-state"))
            .env("TZ", "Pacific/Honolulu");
        command
    }
    fn report(&self, extra: &[&str]) -> (Value, Output) {
        let mut args = vec!["done", "-o", "json"];
        args.extend_from_slice(extra);
        let output = self.cli(&args).output().unwrap();
        assert!(output.status.success(), "{extra:?}: {output:?}");
        (serde_json::from_slice(&output.stdout).unwrap(), output)
    }
}
fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn walk(root: &Path, path: &Path, map: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in std::fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(root, &path, map)
            } else {
                map.insert(
                    path.strip_prefix(root).unwrap().to_owned(),
                    std::fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut map = BTreeMap::new();
    walk(root, root, &mut map);
    map
}
fn row<'a>(report: &'a Value, id: &str) -> &'a Value {
    report["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == id)
        .unwrap()
}

#[test]
fn dates_use_raw_commit_transitions_in_utc_not_resolution_mtime_or_later_edits() {
    let f = Fixture::new();
    f.state("01", "open", "Initial work");
    f.commit(f.now - 86400, "+0000");
    f.state(
        "01",
        "done",
        "## Resolution (2099-01-01)\n\nMisleading body date",
    );
    let close = f.commit(1_704_067_200, "+1400");
    f.state(
        "01",
        "done",
        "## Resolution (2099-01-01)\n\nMisleading body date\n\nLater metadata edit",
    );
    f.commit(f.now - 60, "-1000");
    let before = snapshot(&f.root);
    let (report, _) = f.report(&[]);
    assert_eq!(row(&report, "01")["closed_unix"], 1_704_067_200i64);
    assert_eq!(row(&report, "01")["closed_at"], "2024-01-01T00:00:00Z");
    assert_eq!(row(&report, "01")["closure_commit"], close);
    assert_eq!(row(&report, "01")["in_window"], false);
    assert_eq!(report["date_source"], "git_committer_transition");
    assert_eq!(snapshot(&f.root), before);
    let out = f.cli(&["done"]).output().unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8(out.stdout)
        .unwrap()
        .contains("2024-01-01T00:00:00Z"));
    assert_eq!(snapshot(&f.root), before);
}

#[test]
fn week_keeps_unknowns_excludes_known_old_dates_and_reports_the_latest_reclosure() {
    let f = Fixture::new();
    for id in ["01", "02", "03", "04", "05"] {
        f.state(id, "open", "Source");
    }
    f.commit(f.now - 30 * 86400, "+0000");
    f.state("01", "done", "First completion");
    f.commit(f.now - 10 * 86400, "+0000");
    f.state("01", "open", "Reopened work");
    f.commit(f.now - 2 * 86400, "+0000");
    f.state("01", "done", "Second completion");
    let latest = f.commit(f.now - 3600, "+1400");
    f.state("02", "done", "Old completion");
    f.commit(f.now - 10 * 86400, "+0000");
    f.state("03", "done", "Future clock");
    f.commit(f.now + 86400, "+0000");
    f.state("04", "cancelled", "Retired");
    f.commit(f.now - 60, "+0000");
    let before = snapshot(&f.root);
    let (report, _) = f.report(&["--week"]);
    assert_eq!(report["count"], 2);
    assert_eq!(report["dated_count"], 1);
    assert_eq!(report["unknown_date_count"], 1);
    assert_eq!(report["dated_in_window_count"], 1);
    assert_eq!(row(&report, "01")["closure_commit"], latest);
    assert_eq!(row(&report, "01")["closed_unix"], f.now - 3600);
    assert_eq!(row(&report, "01")["in_window"], true);
    assert_eq!(row(&report, "03")["closed_at"], Value::Null);
    assert_eq!(row(&report, "03")["in_window"], Value::Null);
    assert_eq!(
        row(&report, "03")["unknown_reason"],
        "invalid_or_future_commit_time"
    );
    assert_eq!(report["window"]["duration_seconds"], 604800);
    assert_eq!(report["window"]["inclusive"], true);
    assert_eq!(
        report["window"]["until_unix"].as_i64().unwrap()
            - report["window"]["since_unix"].as_i64().unwrap(),
        604800
    );
    assert_eq!(snapshot(&f.root), before);
    let (all, _) = f.report(&[]);
    assert_eq!(all["count"], 3);
    assert_eq!(row(&all, "02")["in_window"], false);
}

#[test]
fn missing_uncommitted_root_duplicate_and_changed_identity_dates_are_unknown() {
    for state in [
        "no-head",
        "untracked",
        "uncommitted",
        "initial",
        "duplicate",
        "id-change",
        "rename",
        "invalid-parent",
    ] {
        let f = Fixture::new();
        match state {
            "no-head" => f.state("01", "done", "Uncommitted root"),
            "untracked" => {
                f.commit(f.now - 60, "+0000");
                f.state("01", "done", "Untracked work");
            }
            "initial" => {
                f.state("01", "done", "Already done at root");
                f.commit(f.now - 60, "+0000");
            }
            "duplicate" => {
                f.state("01", "done", "Initial done");
                std::fs::write(
                    f.root.join(".tickets/02-other.md"),
                    text("01", "done", "Duplicated ID", "stream:alpha"),
                )
                .unwrap();
                f.commit(f.now - 60, "+0000");
            }
            "invalid-parent" => {
                std::fs::write(
                    f.root.join(".tickets/01-work.md"),
                    format!(
                        "\n{}",
                        text("01", "open", "Invalid leading bytes", "stream:alpha")
                    ),
                )
                .unwrap();
                f.commit(f.now - 3600, "+0000");
                f.state("01", "done", "Current valid data");
                f.commit(f.now - 60, "+0000");
            }
            _ => {
                f.state("01", "open", "Initial");
                f.commit(f.now - 3600, "+0000");
                f.state("01", "done", "Completed");
                if state != "uncommitted" {
                    f.commit(f.now - 1800, "+0000");
                }
                if state == "id-change" {
                    std::fs::write(
                        f.root.join(".tickets/01-work.md"),
                        text("02", "done", "Identity changed", "stream:alpha"),
                    )
                    .unwrap();
                    f.commit(f.now - 60, "+0000");
                }
                if state == "rename" {
                    git(
                        &f.root,
                        &["mv", ".tickets/01-work.md", ".tickets/01-renamed.md"],
                    );
                    f.commit(f.now - 60, "+0000");
                }
            }
        }
        let before = snapshot(&f.root);
        let (report, _) = f.report(&["--week"]);
        assert!(report["count"].as_u64().unwrap() > 0, "{state}");
        for row in report["rows"].as_array().unwrap() {
            assert_eq!(row["closed_at"], Value::Null, "{state}: {row}");
            assert_eq!(row["in_window"], Value::Null, "{state}");
            assert!(row["unknown_reason"].is_string());
        }
        assert_eq!(snapshot(&f.root), before, "{state}");
        let expected = match state {
            "no-head" => "missing_or_unreadable_head",
            "untracked" => "untracked_or_missing_committed_file",
            "uncommitted" => "uncommitted_done_transition",
            "initial" => "initial_done_without_transition",
            "duplicate" => "duplicate_current_identity",
            "id-change" => "identity_discontinuity",
            "rename" => "path_boundary_or_rename",
            _ => "unparseable_history",
        };
        assert_eq!(report["rows"][0]["unknown_reason"], expected, "{state}");
    }
}

#[test]
fn merge_and_shallow_history_are_explicitly_unknown_without_fetching() {
    let f = Fixture::new();
    f.state("01", "open", "Initial");
    f.commit(f.now - 3 * 86400, "+0000");
    git(&f.root, &["checkout", "-qb", "feature"]);
    f.state("01", "done", "Completed on branch");
    f.commit(f.now - 2 * 86400, "+0000");
    git(&f.root, &["checkout", "-q", "main"]);
    std::fs::write(f.root.join("README.md"), "Main progress\n").unwrap();
    f.commit(f.now - 86400, "+0000");
    let out = command("git")
        .arg("-C")
        .arg(&f.root)
        .args(["merge", "--no-ff", "-qm", "merge completed work", "feature"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    let before = snapshot(&f.root);
    let (report, _) = f.report(&[]);
    assert_eq!(row(&report, "01")["unknown_reason"], "merge_history");
    assert_eq!(snapshot(&f.root), before);
    let f = Fixture::new();
    f.state("01", "open", "Initial");
    f.commit(f.now - 3600, "+0000");
    f.state("01", "done", "Completed");
    f.commit(f.now - 60, "+0000");
    assert!(row(&f.report(&[]).0, "01")["closed_at"].is_string());
    let clone = f._temp.path().join("shallow");
    let out = command("git")
        .args(["clone", "-q", "--depth", "1", "--no-local"])
        .arg(&f.root)
        .arg(&clone)
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    let before = snapshot(&clone);
    let out = f
        .cli(&["done", "--week", "-o", "json"])
        .current_dir(&clone)
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    let report: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(row(&report, "01")["unknown_reason"], "shallow_history");
    assert_eq!(snapshot(&clone), before);
}

#[test]
fn context_is_reported_and_can_be_bypassed_without_agent_or_machine_narrowing() {
    let f = Fixture::new();
    for (id, tag) in [("01", "stream:alpha"), ("02", "stream:beta")] {
        std::fs::write(
            f.root.join(format!(".tickets/{id}-work.md")),
            text(id, "open", "Initial", tag),
        )
        .unwrap();
    }
    f.commit(f.now - 3600, "+0000");
    for (id, tag) in [("01", "stream:alpha"), ("02", "stream:beta")] {
        std::fs::write(
            f.root.join(format!(".tickets/{id}-work.md")),
            text(id, "done", "Completed", tag),
        )
        .unwrap();
    }
    f.commit(f.now - 60, "+0000");
    let out = f.cli(&["context", "stream:alpha"]).output().unwrap();
    assert!(out.status.success(), "{out:?}");
    let before = snapshot(&f.root);
    let (report, out) = f.report(&["--agent", "different-agent"]);
    assert_eq!(report["count"], 1);
    assert!(String::from_utf8_lossy(&out.stderr).contains("context_narrowing"));
    let (report, _) = f.report(&["--all-context", "--quiet"]);
    assert_eq!(report["count"], 2);
    assert_eq!(snapshot(&f.root), before);
}

#[test]
fn per_ticket_history_limit_is_unknown_instead_of_using_a_recent_metadata_date() {
    let f = Fixture::new();
    f.state("01", "open", "Initial");
    let base = f.commit(f.now - 10 * 86400, "+0000");
    let mut input = Vec::new();
    let mut previous = base;
    for index in 0..260 {
        let body = text(
            "01",
            "done",
            &format!("Metadata edit {index}"),
            "stream:alpha",
        );
        let blob = index * 2 + 1;
        let commit = blob + 1;
        let unix = f.now - 3600 + index;
        write!(input,"blob\nmark :{blob}\ndata {}\n{}\ncommit refs/heads/main\nmark :{commit}\ncommitter Report fixture <fixture@example.invalid> {unix} +0000\ndata 7\nfixture\nfrom {previous}\nM 100644 :{blob} .tickets/01-work.md\n\n",body.len(),body).unwrap();
        previous = format!(":{commit}");
    }
    let mut child = command("git")
        .arg("-C")
        .arg(&f.root)
        .args(["fast-import", "--quiet"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(&input).unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "{out:?}");
    git(&f.root, &["reset", "--hard", "HEAD"]);
    let before = snapshot(&f.root);
    let (report, _) = f.report(&["--week"]);
    assert_eq!(row(&report, "01")["unknown_reason"], "ticket_history_limit");
    assert_eq!(row(&report, "01")["in_window"], Value::Null);
    assert_eq!(snapshot(&f.root), before);
}

#[test]
fn report_lookup_budget_retains_remaining_rows_with_unknown_dates() {
    let f = Fixture::new();
    for number in 1..=3000 {
        let id = format!("{number:04}");
        f.state(&id, "open", "Initial record");
    }
    f.commit(f.now - 3600, "+0000");
    for number in 1..=3000 {
        let id = format!("{number:04}");
        f.state(&id, "done", "Completed record");
    }
    f.commit(f.now - 60, "+0000");
    let before = snapshot(&f.root);
    let (report, _) = f.report(&[]);
    assert_eq!(report["count"], 3000);
    let rows = report["rows"].as_array().unwrap();
    assert!(rows.iter().any(|row| row["closed_at"].is_string()));
    assert!(rows
        .iter()
        .any(|row| row["unknown_reason"] == "report_history_budget"));
    assert!(rows
        .iter()
        .filter(|row| row["unknown_reason"] == "report_history_budget")
        .all(|row| row["closed_at"].is_null() && row["in_window"].is_null()));
    assert_eq!(snapshot(&f.root), before);
}

#[test]
fn missing_promised_history_never_triggers_a_lazy_fetch() {
    let f = Fixture::new();
    f.state("01", "open", "Initial");
    f.commit(f.now - 3600, "+0000");
    f.state("01", "done", "Completed");
    f.commit(f.now - 60, "+0000");
    let blob = git(&f.root, &["rev-parse", "HEAD:.tickets/01-work.md"]);
    std::fs::remove_file(
        f.root
            .join(".git/objects")
            .join(&blob[..2])
            .join(&blob[2..]),
    )
    .unwrap();
    git(&f.root, &["config", "core.repositoryformatversion", "1"]);
    git(&f.root, &["config", "extensions.partialclone", "origin"]);
    git(&f.root, &["config", "remote.origin.promisor", "true"]);
    git(
        &f.root,
        &[
            "config",
            "remote.origin.url",
            "ssh://fixture.invalid/repository",
        ],
    );
    let script = f.root.join(".git/no-fetch.sh");
    std::fs::write(
        &script,
        "#!/bin/sh\nprintf attempted > .git/fetch-attempted\nexit 1\n",
    )
    .unwrap();
    let ssh = format!("sh \"{}\"", script.to_string_lossy().replace('\\', "/"));
    let out = command("git")
        .arg("-C")
        .arg(&f.root)
        .args(["cat-file", "-p", &blob])
        .env("GIT_SSH_COMMAND", &ssh)
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(
        f.root.join(".git/fetch-attempted").exists(),
        "lazy-fetch oracle was not exercised: {out:?}"
    );
    std::fs::remove_file(f.root.join(".git/fetch-attempted")).unwrap();
    let before = snapshot(&f.root);
    let out = f
        .cli(&["done", "--week", "-o", "json"])
        .env("GIT_SSH_COMMAND", &ssh)
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    let report: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(row(&report, "01")["unknown_reason"], "history_unavailable");
    assert_eq!(row(&report, "01")["closed_at"], Value::Null);
    assert!(!f.root.join(".git/fetch-attempted").exists());
    assert_eq!(snapshot(&f.root), before);
}

#[test]
fn oversized_history_reports_its_limit_without_changing_files() {
    let f = Fixture::new();
    f.state("01", "open", &"x".repeat(33 * 1024 * 1024));
    f.commit(f.now - 3600, "+0000");
    f.state("01", "done", "Small current record");
    f.commit(f.now - 60, "+0000");
    let before = snapshot(&f.root);
    let (report, _) = f.report(&["--week"]);
    assert_eq!(row(&report, "01")["unknown_reason"], "history_object_limit");
    assert_eq!(row(&report, "01")["closed_at"], Value::Null);
    assert_eq!(snapshot(&f.root), before);
}

#[test]
fn oversized_tree_listing_never_produces_dates_from_a_partial_map() {
    let f = Fixture::new();
    for number in 1..=4097 {
        let id = format!("{number:04}");
        f.state(&id, "done", "Initial closed record");
    }
    f.commit(f.now - 60, "+0000");
    let before = snapshot(&f.root);
    let (report, _) = f.report(&["--week"]);
    assert_eq!(report["count"], 4097);
    assert!(report["rows"]
        .as_array()
        .unwrap()
        .iter()
        .all(|row| row["unknown_reason"] == "history_listing_limit"
            && row["closed_at"].is_null()
            && row["in_window"].is_null()));
    assert_eq!(snapshot(&f.root), before);
}

#[test]
fn total_history_payload_limit_is_explicit_after_verified_earlier_rows() {
    let f = Fixture::new();
    let payload = "x".repeat(20 * 1024 * 1024);
    for id in ["01", "02", "03", "04"] {
        f.state(id, "open", &payload);
    }
    f.commit(f.now - 3600, "+0000");
    for id in ["01", "02", "03", "04"] {
        f.state(id, "done", "Small completed record");
    }
    f.commit(f.now - 60, "+0000");
    let before = snapshot(&f.root);
    let (report, _) = f.report(&["--week"]);
    let rows = report["rows"].as_array().unwrap();
    assert_eq!(
        rows.iter()
            .filter(|row| row["closed_at"].is_string())
            .count(),
        3
    );
    assert_eq!(
        rows.iter()
            .filter(
                |row| row["unknown_reason"] == "history_read_limit" && row["in_window"].is_null()
            )
            .count(),
        1
    );
    assert_eq!(snapshot(&f.root), before);
}

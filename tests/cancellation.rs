use serde_json::Value;
use std::path::Path;
use std::process::{Command, Output};

fn run(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tkt"))
        .current_dir(dir)
        .args(args)
        .env("DO_NOT_TRACK", "1")
        .env("TKT_DEBUG", "0")
        .env_remove("TKT_DEBUG_FORMAT")
        .env("TKT_NO_USER_CONFIG", "1")
        .env_remove("TKT_AGENT")
        .env_remove("CREW_ENV")
        .output()
        .unwrap()
}

fn ok(dir: &Path, args: &[&str]) -> String {
    let output = run(dir, args);
    assert!(
        output.status.success(),
        "{:?}: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

fn git(dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {:?}: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

fn repo() -> tempfile::TempDir {
    repo_with_push("enabled = false", false)
}

fn repo_with_push(push_line: &str, remote_enabled: bool) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    assert!(Command::new("git")
        .current_dir(dir.path())
        .args(["init", "-q"])
        .status()
        .unwrap()
        .success());
    git(dir.path(), &["config", "user.name", "test"]);
    git(dir.path(), &["config", "user.email", "test@test"]);
    std::fs::create_dir(dir.path().join(".tickets")).unwrap();
    std::fs::write(
        dir.path().join(".tickets/config.toml"),
        format!("[push]\n{push_line}\n"),
    )
    .unwrap();
    std::fs::create_dir(dir.path().join("docs")).unwrap();
    std::fs::write(
        dir.path().join("docs/plan.md"),
        "# Plan\n\n| ID | Work | Status |\n|---:|---|---|\n| 01 | Target | open |\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join(".tickets/01-target.md"),
        ticket_text("01", "open", "blocked_by: []\n", "- [ ] First criterion\n"),
    )
    .unwrap();
    git(dir.path(), &["add", "."]);
    git(dir.path(), &["commit", "-m", "seed"]);

    if remote_enabled {
        let bare = dir.path().join("remote.git");
        assert!(Command::new("git")
            .args(["init", "--bare", bare.to_str().unwrap()])
            .status()
            .unwrap()
            .success());
        git(
            dir.path(),
            &["remote", "add", "origin", bare.to_str().unwrap()],
        );
        git(dir.path(), &["push", "-u", "origin", "HEAD"]);
    }
    dir
}

fn ticket_text(id: &str, status: &str, fields: &str, body: &str) -> String {
    format!("---\nid: \"{id}\"\ntitle: \"Ticket {id}\"\nstatus: {status}\n{fields}---\n\n{body}")
}

fn write_ticket(dir: &Path, id: &str, slug: &str, status: &str, fields: &str, body: &str) {
    std::fs::write(
        dir.join(format!(".tickets/{id}-{slug}.md")),
        ticket_text(id, status, fields, body),
    )
    .unwrap();
    git(dir, &["add", &format!(".tickets/{id}-{slug}.md")]);
    git(dir, &["commit", "-m", &format!("add {id}")]);
}

fn text(dir: &Path, id: &str) -> String {
    let path = std::fs::read_dir(dir.join(".tickets"))
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| {
            path.extension().is_some_and(|ext| ext == "md")
                && path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with(&format!("{id}-"))
        })
        .unwrap();
    std::fs::read_to_string(path).unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn json_lines(output: &Output) -> Vec<Value> {
    stdout(output)
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

#[test]
fn cancel_requires_valid_reason_by_and_source_status() {
    let dir = repo();
    let path = dir.path();
    write_ticket(
        path,
        "02",
        "replacement",
        "open",
        "blocked_by: []\n",
        "Body.\n",
    );
    write_ticket(
        path,
        "03",
        "already-cancelled",
        "cancelled",
        "blocked_by: []\n",
        "## Cancellation\n\n- Reason: obsolete\n- Note: old\n",
    );
    write_ticket(
        path,
        "05",
        "closed",
        "done",
        "blocked_by: []\n",
        "## Resolution\n\nDone.\n",
    );

    let missing_by = run(
        path,
        &[
            "cancel",
            "01",
            "--reason",
            "duplicate",
            "--note",
            "same work",
        ],
    );
    assert_eq!(missing_by.status.code(), Some(1));
    assert!(stderr(&missing_by).contains("--by"));
    let invalid_reason = run(
        path,
        &[
            "cancel",
            "01",
            "--reason",
            "finished",
            "--note",
            "same work",
        ],
    );
    assert_eq!(invalid_reason.status.code(), Some(2));

    for by in ["99", "03", "01"] {
        let invalid = run(
            path,
            &[
                "cancel",
                "01",
                "--reason",
                "duplicate",
                "--note",
                "same work",
                "--by",
                by,
            ],
        );
        assert_eq!(invalid.status.code(), Some(1));
    }

    let done = run(
        path,
        &[
            "cancel",
            "05",
            "--reason",
            "obsolete",
            "--note",
            "not needed",
        ],
    );
    assert_eq!(done.status.code(), Some(1));
    assert!(text(path, "05").contains("status: done"));

    let valid = ok(
        path,
        &[
            "cancel",
            "01",
            "--reason",
            "duplicate",
            "--note",
            "same work",
            "--by",
            "02",
        ],
    );
    assert!(valid.contains("Cancelled"));
    let canceled = text(path, "01");
    assert!(canceled.contains("status: cancelled"));
    assert!(canceled.contains("## Cancellation"));
    assert!(canceled.contains("Reason: duplicate"));
    assert!(canceled.contains("By: 02"));
    assert!(canceled.contains("Note: same work"));
    assert!(!canceled.contains("## Resolution"));
    assert!(canceled.contains("- [ ] First criterion"));
    let manifest: Value = serde_json::from_str(&ok(path, &["capabilities"])).unwrap();
    assert!(manifest["commands"].get("cancel").is_some());
    assert!(manifest["commands"].get("reopen").is_some());
    assert!(
        manifest["commands"]["query"]["inputSchema"]["properties"]["status"]["enum"]
            .as_array()
            .unwrap()
            .iter()
            .any(|status| status == "cancelled")
    );
    assert_eq!(
        manifest["commands"]["cancel"]["inputSchema"]["properties"]["reason"]["enum"],
        serde_json::json!(["duplicate", "superseded", "obsolete", "wontfix"])
    );
    ok(
        path,
        &[
            "cancel",
            "02",
            "--reason",
            "obsolete",
            "--note",
            "retired replacement",
        ],
    );
    assert!(ok(path, &["validate"]).contains("\"status\":\"pass\""));

    for status in ["backlog", "open", "in_progress"] {
        write_ticket(
            path,
            "10",
            "lifecycle",
            status,
            "blocked_by: []\n",
            "Body.\n",
        );
        ok(
            path,
            &["cancel", "10", "--reason", "obsolete", "--note", "old"],
        );
        assert!(text(path, "10").contains("status: cancelled"));
        // Reopen before the next source-state case.
        ok(path, &["reopen", "10"]);
    }
}

#[test]
fn dependents_require_and_apply_an_explicit_edge_disposition() {
    let dir = repo();
    let path = dir.path();
    write_ticket(
        path,
        "02",
        "dependent",
        "open",
        "blocked_by: [\"01\"]\n",
        "Body.\n",
    );
    write_ticket(
        path,
        "03",
        "second",
        "in_progress",
        "blocked_by: [\"01\"]\n",
        "Body.\n",
    );
    write_ticket(
        path,
        "04",
        "replacement",
        "open",
        "blocked_by: []\n",
        "Body.\n",
    );
    write_ticket(
        path,
        "05",
        "terminal-dependent",
        "done",
        "blocked_by: [\"01\"]\n",
        "## Resolution\n\nDone.\n",
    );

    let blocked = run(
        path,
        &["cancel", "01", "--reason", "obsolete", "--note", "old"],
    );
    assert_eq!(blocked.status.code(), Some(1));
    assert!(stderr(&blocked).contains("02"));
    assert!(stderr(&blocked).contains("03"));
    assert!(text(path, "01").contains("status: open"));

    let repointed = ok(
        path,
        &[
            "cancel",
            "01",
            "--reason",
            "superseded",
            "--note",
            "replaced",
            "--by",
            "04",
            "--repoint",
        ],
    );
    for id in ["02", "03", "05"] {
        assert!(repointed.contains(id), "{repointed}");
        let contents = text(path, id);
        assert!(!contents.contains("blocked_by: [\"01\"]"));
        assert!(contents.contains("\"04\""));
    }

    let changed = git(path, &["show", "--format=", "--name-only", "HEAD"]);
    assert!(changed.contains(".tickets/01-target.md"));
    assert!(changed.contains(".tickets/02-dependent.md"));
    assert!(changed.contains(".tickets/03-second.md"));
    assert!(changed.contains(".tickets/05-terminal-dependent.md"));
    assert!(changed.contains("docs/plan.md"));
}

#[test]
fn repoint_refuses_to_create_a_dependency_cycle() {
    let dir = repo();
    let path = dir.path();
    write_ticket(
        path,
        "02",
        "dependent",
        "open",
        "blocked_by: [\"01\"]\n",
        "Body.\n",
    );
    write_ticket(
        path,
        "03",
        "replacement",
        "open",
        "blocked_by: [\"02\"]\n",
        "Body.\n",
    );
    let output = run(
        path,
        &[
            "cancel",
            "01",
            "--reason",
            "obsolete",
            "--note",
            "old",
            "--by",
            "03",
            "--repoint",
        ],
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("dependency cycle"));
    assert!(text(path, "01").contains("status: open"));
    assert!(text(path, "02").contains("blocked_by: [\"01\"]"));
}

#[test]
fn terminal_dependents_are_cleaned_without_changing_their_history() {
    let dir = repo();
    let path = dir.path();
    let body =
        "## Resolution (2026-10-30)\n\nCompleted independently.\n\n- [ ] Historical criterion\n";
    write_ticket(
        path,
        "04",
        "terminal-dependent",
        "done",
        "blocked_by: [\"01\"]\n",
        body,
    );
    let before = text(path, "04");
    let output = ok(
        path,
        &["cancel", "01", "--reason", "obsolete", "--note", "old"],
    );
    assert!(output.contains("Dropped blocked_by for 04"));
    let after = text(path, "04");
    assert!(after.contains("blocked_by: []"));
    assert!(after.contains("## Resolution (2026-10-30)"));
    assert!(after.contains("Completed independently."));
    assert!(after.contains("- [ ] Historical criterion"));
    assert_eq!(
        before.split_once("---\n\n").unwrap().1,
        after.split_once("---\n\n").unwrap().1
    );
    let changed = git(path, &["show", "--format=", "--name-only", "HEAD"]);
    assert!(changed.contains(".tickets/01-target.md"));
    assert!(changed.contains(".tickets/04-terminal-dependent.md"));
}

#[test]
fn drop_edges_repoints_no_tickets_and_reopen_keeps_history() {
    let dir = repo();
    let path = dir.path();
    write_ticket(
        path,
        "02",
        "dependent",
        "open",
        "blocked_by: [\"01\"]\n",
        "Body.\n",
    );
    let not_cancelled = run(path, &["reopen", "02"]);
    assert_eq!(not_cancelled.status.code(), Some(1));
    ok(
        path,
        &[
            "cancel",
            "01",
            "--reason",
            "obsolete",
            "--note",
            "old",
            "--drop-edges",
        ],
    );
    assert!(text(path, "02").contains("blocked_by: []"));
    let old = text(path, "01");
    assert!(old.contains("status: cancelled"));

    ok(path, &["reopen", "01"]);
    let reopened = text(path, "01");
    assert!(reopened.contains("status: open"));
    assert!(reopened.contains("## Cancellation"));
    assert!(reopened.contains("Reason: obsolete"));
    assert_eq!(
        git(path, &["log", "-1", "--format=%s"]),
        "chore(tickets): reopen 01"
    );
}

#[test]
fn views_validation_audit_and_plan_treat_cancelled_as_terminal() {
    let dir = repo();
    let path = dir.path();
    write_ticket(
        path,
        "02",
        "blocked",
        "open",
        "blocked_by: [\"01\"]\n",
        "Body.\n",
    );
    ok(
        path,
        &[
            "cancel",
            "01",
            "--reason",
            "wontfix",
            "--note",
            "not planned",
            "--drop-edges",
        ],
    );
    assert!(!ok(path, &["ready"]).contains("01"));
    assert!(!ok(path, &["blocked"]).contains("01"));
    assert!(json_lines(&run(path, &["query"]))
        .iter()
        .any(|item| { item["id"] == "01" && item["status"] == "cancelled" }));
    assert_eq!(
        json_lines(&run(path, &["query", "--status", "cancelled"])).len(),
        1
    );
    let plan_path = path.join("docs/plan.md");
    let plan = std::fs::read_to_string(&plan_path).unwrap();
    assert!(plan.contains("cancelled (wontfix)"));
    std::fs::write(&plan_path, plan.replace("cancelled (wontfix)", "open")).unwrap();
    assert!(ok(path, &["sync-plan"]).contains("cancelled (wontfix)"));
    ok(path, &["sync-plan", "--fix"]);
    assert!(std::fs::read_to_string(&plan_path)
        .unwrap()
        .contains("cancelled (wontfix)"));
    assert!(ok(path, &["audit"]).contains("\"status\":\"pass\""));
    let validation = run(path, &["validate"]);
    assert!(
        validation.status.success(),
        "{}{}",
        stdout(&validation),
        stderr(&validation)
    );
    assert!(stdout(&validation).contains("\"status\":\"pass\""));

    let invalid = repo();
    let invalid_path = invalid.path();
    write_ticket(
        invalid_path,
        "02",
        "bad-cancel",
        "cancelled",
        "blocked_by: []\nvalidation_criteria: [\"success\"]\n",
        "- [ ] TBD\n",
    );
    write_ticket(
        invalid_path,
        "03",
        "bad-edge",
        "open",
        "blocked_by: [\"02\"]\n",
        "Body.\n",
    );
    write_ticket(
        invalid_path,
        "06",
        "bad-by",
        "cancelled",
        "blocked_by: []\n",
        "## Cancellation\n\n- Reason: duplicate\n- By: 99\n- Note: duplicate\n- Date: 2026-11-02\n",
    );
    let invalid_validation = run(invalid_path, &["validate"]);
    assert_eq!(invalid_validation.status.code(), Some(1));
    let report: Value = serde_json::from_str(&stdout(&invalid_validation)).unwrap();
    let rules: Vec<&str> = report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| finding["rule"].as_str().unwrap())
        .collect();
    assert!(rules.contains(&"missing-cancellation"));
    assert!(rules.contains(&"blocked-by-cancelled"));
    assert!(rules.contains(&"invalid-cancellation-by"));
    let audit: Value = serde_json::from_str(&ok(invalid_path, &["audit"])).unwrap();
    assert!(audit["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|finding| { finding["rule"] == "cancellation-quality" }));
    assert!(
        !audit["findings"].as_array().unwrap().iter().any(|finding| {
            [
                "missing-resolution",
                "all-acs-unchecked-on-done",
                "low-evidence-closure",
            ]
            .contains(&finding["rule"].as_str().unwrap())
        })
    );
}

#[test]
fn close_and_edit_cannot_bypass_cancel_route_and_push_disabled_still_commits_locally() {
    let dir = repo_with_push("enabled = false", true);
    let path = dir.path();
    let remote = path.join("remote.git");
    let remote_before = git(&remote, &["rev-parse", "HEAD"]);
    ok(
        path,
        &["cancel", "01", "--reason", "obsolete", "--note", "retired"],
    );
    let remote_after = git(&remote, &["rev-parse", "HEAD"]);
    assert_eq!(remote_before, remote_after);
    assert_ne!(git(path, &["rev-parse", "HEAD"]), remote_before);

    for args in [
        vec!["close", "01", "--resolution", "retired"],
        vec!["edit", "01", "--status", "cancelled"],
        vec!["edit", "01", "--status", "open"],
        vec!["new", "not-cancel", "--status", "cancelled"],
        vec!["batch", "one", "--status", "cancelled"],
    ] {
        let output = run(path, &args);
        assert!(!output.status.success(), "{args:?}");
        if args[0] == "close" {
            assert!(stderr(&output).contains("cancelled and cannot be closed"));
        }
    }
    assert!(text(path, "01").contains("status: cancelled"));
}

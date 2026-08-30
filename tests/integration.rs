//! Integration tests — exercises tkt commands against real git repos.
//!
//! Each test creates a tempdir with a bare remote + clone, seeds .tickets/,
//! and runs the tkt binary via Command.

use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempDir;

/// Path to the tkt binary (cargo builds it before running tests).
fn tkt_bin() -> PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // remove test binary name
    path.pop(); // remove 'deps'
    path.push("tkt");
    if cfg!(windows) {
        path.set_extension("exe");
    }
    path
}

/// Run tkt in a given directory, return (exit_code, stdout+stderr).
fn run_tkt(dir: &Path, args: &[&str]) -> (i32, String) {
    let output = Command::new(tkt_bin())
        .args(args)
        .current_dir(dir)
        .env("DO_NOT_TRACK", "1")
        .env("TKT_NO_USER_CONFIG", "1")
        .env_remove("TKT_DEBUG")
        .output()
        .expect("failed to execute tkt");
    let code = output.status.code().unwrap_or(1);
    let out = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    (code, out)
}

fn git(dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(["-C", &dir.to_string_lossy()])
        .args(args)
        .output()
        .expect("git failed");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

/// Set up a bare remote + clone with a seeded .tickets/ directory.
fn setup_repo() -> (TempDir, PathBuf) {
    let tmp = TempDir::new().unwrap();
    let remote = tmp.path().join("remote.git");
    let clone = tmp.path().join("work");

    // Create bare remote
    Command::new("git")
        .args(["init", "--bare", "-q", "-b", "main"])
        .arg(&remote)
        .output()
        .unwrap();

    // Clone
    Command::new("git")
        .args(["clone", "-q"])
        .arg(&remote)
        .arg(&clone)
        .output()
        .unwrap();

    git(&clone, &["config", "user.email", "test@test"]);
    git(&clone, &["config", "user.name", "test"]);
    git(&clone, &["config", "core.autocrlf", "false"]);

    // Create .tickets/ with a seed ticket
    std::fs::create_dir_all(clone.join(".tickets")).unwrap();
    std::fs::write(
        clone.join(".tickets/01-seed.md"),
        "---\nid: \"01\"\ntitle: \"Seed ticket\"\nstatus: done\nblocked_by: []\n---\n\n# Seed\n\n- [x] Done\n\n## Resolution (2026-01-01)\n\nSeeded for tests.\n",
    ).unwrap();

    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-qm", "seed"]);
    git(&clone, &["push", "-q", "origin", "HEAD:main"]);

    (tmp, clone)
}

#[test]
fn test_ready_shows_frontier() {
    let (_tmp, clone) = setup_repo();

    // Add an open ticket that depends on the done seed
    std::fs::write(
        clone.join(".tickets/02-feature.md"),
        "---\nid: \"02\"\ntitle: \"Feature\"\nstatus: open\nblocked_by: [\"01\"]\n---\n\n# Feature\n\n- [ ] TBD\n",
    ).unwrap();
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-qm", "add feature"]);

    let (code, out) = run_tkt(&clone, &["ready"]);
    assert_eq!(code, 0, "ready should succeed: {}", out);
    assert!(
        out.contains("02"),
        "should show ticket 02 on frontier: {}",
        out
    );
    assert!(out.contains("Feature"), "should show title: {}", out);
}

#[test]
fn test_ready_excludes_blocked() {
    let (_tmp, clone) = setup_repo();

    // Ticket blocked by non-existent dep
    std::fs::write(
        clone.join(".tickets/02-blocked.md"),
        "---\nid: \"02\"\ntitle: \"Blocked\"\nstatus: open\nblocked_by: [\"99\"]\n---\n\n# Blocked\n",
    ).unwrap();
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-qm", "add blocked"]);

    let (code, out) = run_tkt(&clone, &["ready"]);
    assert_eq!(code, 0);
    assert!(
        !out.contains("02"),
        "blocked ticket should not be on frontier: {}",
        out
    );
}

#[test]
fn test_new_creates_and_pushes() {
    let (_tmp, clone) = setup_repo();

    let (code, out) = run_tkt(&clone, &["new", "my-feature", "--title", "My Feature"]);
    assert_eq!(code, 0, "new should succeed: {}", out);
    assert!(out.contains("created"), "should say created: {}", out);
    assert!(
        out.contains("02") && out.contains("my-feature"),
        "should create 02 my-feature: {}",
        out
    );

    // Verify file exists
    assert!(clone.join(".tickets/02-my-feature.md").exists());

    // Verify it was pushed (commit is on remote)
    let log = git(&clone, &["log", "--oneline", "origin/main"]);
    assert!(
        log.contains("new 02 my-feature"),
        "should be pushed: {}",
        log
    );
}

#[test]
fn test_lifecycle_new_claim_close() {
    let (_tmp, clone) = setup_repo();

    // New
    let (code, _) = run_tkt(&clone, &["new", "lifecycle-test", "--title", "Lifecycle"]);
    assert_eq!(code, 0);

    // Claim
    let (code, out) = run_tkt(&clone, &["claim", "02"]);
    assert_eq!(code, 0, "claim should succeed: {}", out);
    assert!(out.contains("claimed"), "should say claimed: {}", out);

    // Verify status changed
    let content = std::fs::read_to_string(clone.join(".tickets/02-lifecycle-test.md")).unwrap();
    assert!(content.contains("status: in_progress"));

    // Close
    let (code, out) = run_tkt(&clone, &["close", "02", "--note", "All done", "--force"]);
    assert_eq!(code, 0, "close should succeed: {}", out);
    assert!(out.contains("closed"), "should say closed: {}", out);

    // Verify status and resolution
    let content = std::fs::read_to_string(clone.join(".tickets/02-lifecycle-test.md")).unwrap();
    assert!(content.contains("status: done"));
    assert!(content.contains("## Resolution"));
    assert!(content.contains("All done"));
}

#[test]
fn test_claim_rejects_non_open() {
    let (_tmp, clone) = setup_repo();

    // Trying to claim the seed ticket (already done)
    let (code, out) = run_tkt(&clone, &["claim", "01"]);
    assert_ne!(code, 0, "claim of done ticket should fail");
    assert!(
        out.contains("done") || out.contains("not open"),
        "should explain why: {}",
        out
    );
}

#[test]
fn test_close_rejects_already_done() {
    let (_tmp, clone) = setup_repo();

    let (code, out) = run_tkt(&clone, &["close", "01"]);
    assert_ne!(code, 0, "close of done ticket should fail");
    assert!(
        out.contains("already done"),
        "should say already done: {}",
        out
    );
}

#[test]
fn test_validate_finds_issues() {
    let (_tmp, clone) = setup_repo();

    // Add a ticket with a dangling blocked_by
    std::fs::write(
        clone.join(".tickets/02-dangling.md"),
        "---\nid: \"02\"\ntitle: \"Dangling\"\nstatus: open\nblocked_by: [\"99\"]\n---\n\n# Dangling\n",
    ).unwrap();
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-qm", "add dangling"]);

    let (code, out) = run_tkt(&clone, &["validate", "--brief"]);
    assert_eq!(code, 1, "validate should fail with dangling ref: {}", out);
    assert!(
        out.contains("dangling-blocked-by"),
        "should report dangling ref: {}",
        out
    );
}

#[test]
fn test_validate_passes_clean_corpus() {
    let (_tmp, clone) = setup_repo();

    let (code, out) = run_tkt(&clone, &["validate", "--brief"]);
    assert_eq!(code, 0, "validate should pass clean corpus: {}", out);
    assert!(out.contains("pass"), "should say pass: {}", out);
}

#[test]
fn test_sync_plan_detects_drift() {
    let (_tmp, clone) = setup_repo();

    // Add an open ticket and a plan that says it's done
    std::fs::write(
        clone.join(".tickets/02-drifted.md"),
        "---\nid: \"02\"\ntitle: \"Drifted\"\nstatus: open\nblocked_by: []\n---\n\n# Drifted\n",
    )
    .unwrap();
    std::fs::create_dir_all(clone.join("docs")).unwrap();
    std::fs::write(
        clone.join("docs/plan.md"),
        "| 01 | Seed | ✅ done |\n| 02 | Drifted | ✅ done |\n",
    )
    .unwrap();
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-qm", "add drifted"]);

    let (code, out) = run_tkt(&clone, &["sync-plan", "--check", "--brief"]);
    assert_eq!(code, 1, "should detect drift: {}", out);
    assert!(
        out.contains("plan-status-drift"),
        "should report drift: {}",
        out
    );
}

#[test]
fn test_sync_plan_fix_corrects_status() {
    let (_tmp, clone) = setup_repo();

    // Add a done ticket but plan says open
    std::fs::write(
        clone.join(".tickets/02-fixable.md"),
        "---\nid: \"02\"\ntitle: \"Fixable\"\nstatus: done\nblocked_by: []\n---\n\n# Fixable\n",
    )
    .unwrap();
    std::fs::create_dir_all(clone.join("docs")).unwrap();
    std::fs::write(
        clone.join("docs/plan.md"),
        "| 01 | Seed | ✅ done |\n| 02 | Fixable | open |\n",
    )
    .unwrap();
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-qm", "add fixable"]);

    let (code, out) = run_tkt(&clone, &["sync-plan", "--fix", "--brief"]);
    assert_eq!(code, 0, "fix should succeed: {}", out);

    // Verify the plan was updated
    let plan = std::fs::read_to_string(clone.join("docs/plan.md")).unwrap();
    assert!(
        plan.contains("✅ done"),
        "plan should now say done for ticket 02: {}",
        plan
    );
}

#[test]
fn test_sync_plan_advisory_default() {
    let (_tmp, clone) = setup_repo();

    // Open ticket, plan says done → derivable drift
    std::fs::write(
        clone.join(".tickets/02-drifted.md"),
        "---\nid: \"02\"\ntitle: \"Drifted\"\nstatus: open\nblocked_by: []\n---\n\n# Drifted\n",
    )
    .unwrap();
    std::fs::create_dir_all(clone.join("docs")).unwrap();
    std::fs::write(
        clone.join("docs/plan.md"),
        "| 01 | Seed | ✅ done |\n| 02 | Drifted | ✅ done |\n",
    )
    .unwrap();
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-qm", "add drifted"]);

    // Default run (no --check/--strict): derivable drift is advisory → exit 0
    let (code, out) = run_tkt(&clone, &["sync-plan", "--brief"]);
    assert_eq!(code, 0, "default run should be advisory (exit 0): {}", out);
    assert!(
        out.contains("plan-status-drift"),
        "drift should still be reported as advisory: {}",
        out
    );
}

#[test]
fn test_sync_plan_check_gate() {
    let (_tmp, clone) = setup_repo();

    std::fs::write(
        clone.join(".tickets/02-drifted.md"),
        "---\nid: \"02\"\ntitle: \"Drifted\"\nstatus: open\nblocked_by: []\n---\n\n# Drifted\n",
    )
    .unwrap();
    std::fs::create_dir_all(clone.join("docs")).unwrap();
    std::fs::write(
        clone.join("docs/plan.md"),
        "| 01 | Seed | ✅ done |\n| 02 | Drifted | ✅ done |\n",
    )
    .unwrap();
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-qm", "add drifted"]);

    // --check repurposed as the CI gate: drift now fails
    let (code, out) = run_tkt(&clone, &["sync-plan", "--check", "--brief"]);
    assert_eq!(code, 1, "--check should gate on drift (exit 1): {}", out);
}

#[test]
fn test_sync_plan_orphan_row_errors() {
    let (_tmp, clone) = setup_repo();

    // Plan references id 99 with no matching ticket → non-derivable conflict (error)
    std::fs::create_dir_all(clone.join("docs")).unwrap();
    std::fs::write(
        clone.join("docs/plan.md"),
        "| 01 | Seed | ✅ done |\n| 99 | Ghost | open |\n",
    )
    .unwrap();
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-qm", "add orphan plan row"]);

    // Even the default (advisory) run fails on a genuine conflict
    let (code, out) = run_tkt(&clone, &["sync-plan", "--brief"]);
    assert_eq!(
        code, 1,
        "orphan plan row should error even by default: {}",
        out
    );
    assert!(
        out.contains("plan-orphan-row"),
        "should report orphan row: {}",
        out
    );
}

#[test]
fn test_sync_plan_fix_dryrun() {
    let (_tmp, clone) = setup_repo();

    std::fs::write(
        clone.join(".tickets/02-fixable.md"),
        "---\nid: \"02\"\ntitle: \"Fixable\"\nstatus: done\nblocked_by: []\n---\n\n# Fixable\n",
    )
    .unwrap();
    std::fs::create_dir_all(clone.join("docs")).unwrap();
    let original = "| 01 | Seed | ✅ done |\n| 02 | Fixable | open |\n";
    std::fs::write(clone.join("docs/plan.md"), original).unwrap();
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-qm", "add fixable"]);

    // --fix --dry-run must NOT write the plan file
    let (code, out) = run_tkt(&clone, &["sync-plan", "--fix", "--dry-run", "--brief"]);
    assert_eq!(code, 0, "dry-run fix should exit 0: {}", out);
    let plan = std::fs::read_to_string(clone.join("docs/plan.md")).unwrap();
    assert_eq!(
        plan, original,
        "dry-run must not modify the plan file: {}",
        plan
    );
}

#[test]
fn test_edit_changes_field() {
    let (_tmp, clone) = setup_repo();

    // Add an open ticket
    std::fs::write(
        clone.join(".tickets/02-editable.md"),
        "---\nid: \"02\"\ntitle: \"Editable\"\nstatus: open\nblocked_by: []\n---\n\n# Editable\n\n- [ ] AC1\n",
    ).unwrap();
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-qm", "add editable"]);

    let (code, out) = run_tkt(&clone, &["edit", "02", "--priority", "high"]);
    assert_eq!(code, 0, "edit should succeed: {}", out);

    let content = std::fs::read_to_string(clone.join(".tickets/02-editable.md")).unwrap();
    assert!(
        content.contains("priority: high"),
        "should have priority: {}",
        content
    );

    // Ready should show it with HIGH flag
    let (_, out) = run_tkt(&clone, &["ready"]);
    assert!(
        out.contains("[HIGH]") || out.contains("02"),
        "should show on frontier: {}",
        out
    );
}

#[test]
fn test_validate_detects_self_cycle() {
    let (_tmp, clone) = setup_repo();

    // Remove seed ticket to avoid id conflict
    std::fs::remove_file(clone.join(".tickets/01-seed.md")).unwrap();

    // Ticket that depends on itself
    std::fs::write(
        clone.join(".tickets/01-self-cycle.md"),
        "---\nid: \"01\"\ntitle: \"Self cycle\"\nstatus: open\nblocked_by: [\"01\"]\n---\n\n# Self\n",
    ).unwrap();
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-qm", "add self-cycle"]);

    let (code, out) = run_tkt(&clone, &["validate", "--brief"]);
    assert_eq!(code, 1, "should fail with cycle: {}", out);
    assert!(out.contains("cycle"), "should mention cycle: {}", out);
    assert!(out.contains("01"), "should mention id 01: {}", out);
}

#[test]
fn test_validate_detects_two_node_cycle() {
    let (_tmp, clone) = setup_repo();

    // Remove seed ticket to avoid id conflict
    std::fs::remove_file(clone.join(".tickets/01-seed.md")).unwrap();

    std::fs::write(
        clone.join(".tickets/01-alpha.md"),
        "---\nid: \"01\"\ntitle: \"Alpha\"\nstatus: open\nblocked_by: [\"02\"]\n---\n\n# A\n",
    )
    .unwrap();
    std::fs::write(
        clone.join(".tickets/02-beta.md"),
        "---\nid: \"02\"\ntitle: \"Beta\"\nstatus: open\nblocked_by: [\"01\"]\n---\n\n# B\n",
    )
    .unwrap();
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-qm", "add cycle pair"]);

    let (code, out) = run_tkt(&clone, &["validate", "--brief"]);
    assert_eq!(code, 1, "should fail with cycle: {}", out);
    assert!(out.contains("cycle"), "should mention cycle: {}", out);
    assert!(
        out.contains("01") && out.contains("02"),
        "should mention both ids: {}",
        out
    );
}

#[test]
fn test_validate_no_false_positive_on_acyclic() {
    let (_tmp, clone) = setup_repo();

    // Remove seed ticket to set up clean chain
    std::fs::remove_file(clone.join(".tickets/01-seed.md")).unwrap();

    // Linear chain: 03 depends on 02 depends on 01, all valid
    std::fs::write(
        clone.join(".tickets/01-first.md"),
        "---\nid: \"01\"\ntitle: \"First\"\nstatus: done\nblocked_by: []\n---\n\n# F\n",
    )
    .unwrap();
    std::fs::write(
        clone.join(".tickets/02-second.md"),
        "---\nid: \"02\"\ntitle: \"Second\"\nstatus: open\nblocked_by: [\"01\"]\n---\n\n# S\n",
    )
    .unwrap();
    std::fs::write(
        clone.join(".tickets/03-third.md"),
        "---\nid: \"03\"\ntitle: \"Third\"\nstatus: open\nblocked_by: [\"02\"]\n---\n\n# T\n",
    )
    .unwrap();
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-qm", "add chain"]);

    let (code, out) = run_tkt(&clone, &["validate", "--brief"]);
    assert_eq!(code, 0, "acyclic chain should pass: {}", out);
    assert!(!out.contains("cycle"), "should not mention cycle: {}", out);
}

#[test]
fn test_exit_code_2_on_crash() {
    // Running tkt in a directory with no git repo should exit 2 (operational crash)
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path().to_path_buf();
    std::fs::create_dir_all(dir.join(".tickets")).unwrap();
    std::fs::write(
        dir.join(".tickets/01-test.md"),
        "---\nid: \"01\"\ntitle: \"Test\"\nstatus: open\nblocked_by: []\n---\n\n# Test\n",
    )
    .unwrap();

    // No git init — so git commands will fail (operational crash)
    let (code, out) = run_tkt(&dir, &["ready"]);
    assert_eq!(code, 2, "should exit 2 on git crash: {}", out);
    assert!(out.contains("crash"), "error should say crash: {}", out);
}

#[test]
fn test_exit_code_1_on_domain_error() {
    let (_tmp, clone) = setup_repo();

    // Claim a ticket that is already done → domain error → exit 1
    let (code, out) = run_tkt(&clone, &["claim", "01"]);
    assert_eq!(code, 1, "domain error should exit 1: {}", out);
    assert!(
        !out.contains("crash"),
        "domain error should not say crash: {}",
        out
    );
}

#[test]
fn test_query_outputs_json_lines() {
    let (_tmp, clone) = setup_repo();

    // Add a second ticket with optional fields
    std::fs::write(
        clone.join(".tickets/02-feature.md"),
        "---\nid: \"02\"\ntitle: \"A feature\"\nstatus: open\nblocked_by: [\"01\"]\nenv: corp\npriority: high\nspec: \"my-spec\"\n---\n\n# Feature\n",
    ).unwrap();
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-qm", "add feature"]);

    let (code, out) = run_tkt(&clone, &["query"]);
    assert_eq!(code, 0, "query should succeed: {}", out);

    // Should have 2 lines (one per ticket)
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines.len(), 2, "should have 2 tickets: {}", out);

    // Each line should be valid JSON (parseable with basic checks)
    for line in &lines {
        assert!(
            line.starts_with('{') && line.ends_with('}'),
            "should be JSON object: {}",
            line
        );
        assert!(line.contains("\"id\""), "should have id: {}", line);
        assert!(line.contains("\"title\""), "should have title: {}", line);
        assert!(line.contains("\"status\""), "should have status: {}", line);
        assert!(
            line.contains("\"blocked_by\""),
            "should have blocked_by: {}",
            line
        );
    }

    // Second ticket should have optional fields
    let line2 = lines[1];
    assert!(line2.contains("\"env\""), "should have env: {}", line2);
    assert!(
        line2.contains("\"priority\""),
        "should have priority: {}",
        line2
    );
    assert!(line2.contains("\"spec\""), "should have spec: {}", line2);
}

#[test]
fn test_competing_allocation_no_collision() {
    // Two clones allocate: first pushes, second gets rejected and reallocates
    let tmp = TempDir::new().unwrap();
    let remote = tmp.path().join("remote.git");
    let clone_a = tmp.path().join("clone-a");
    let clone_b = tmp.path().join("clone-b");

    // Create bare remote
    Command::new("git")
        .args(["init", "--bare", "-q", "-b", "main"])
        .arg(&remote)
        .output()
        .unwrap();

    // Clone A
    Command::new("git")
        .args(["clone", "-q"])
        .arg(&remote)
        .arg(&clone_a)
        .output()
        .unwrap();
    git(&clone_a, &["config", "user.email", "a@test"]);
    git(&clone_a, &["config", "user.name", "a"]);
    git(&clone_a, &["config", "core.autocrlf", "false"]);
    std::fs::create_dir_all(clone_a.join(".tickets")).unwrap();
    std::fs::write(
        clone_a.join(".tickets/01-seed.md"),
        "---\nid: \"01\"\ntitle: \"Seed\"\nstatus: done\nblocked_by: []\n---\n\n# Seed\n",
    )
    .unwrap();
    git(&clone_a, &["add", "-A"]);
    git(&clone_a, &["commit", "-qm", "seed"]);
    git(&clone_a, &["push", "-q", "origin", "HEAD:main"]);

    // Clone B
    Command::new("git")
        .args(["clone", "-q"])
        .arg(&remote)
        .arg(&clone_b)
        .output()
        .unwrap();
    git(&clone_b, &["config", "user.email", "b@test"]);
    git(&clone_b, &["config", "user.name", "b"]);
    git(&clone_b, &["config", "core.autocrlf", "false"]);

    // A allocates and pushes first
    let (code_a, out_a) = run_tkt(&clone_a, &["new", "alpha", "--title", "Alpha ticket"]);
    assert_eq!(code_a, 0, "A should succeed: {}", out_a);
    assert!(
        out_a.contains("02") && out_a.contains("alpha"),
        "A should get 02 alpha: {}",
        out_a
    );

    // B allocates — will try 02, get rejected, rebase, get 03
    let (code_b, out_b) = run_tkt(&clone_b, &["new", "beta", "--title", "Beta ticket"]);
    assert_eq!(code_b, 0, "B should succeed after retry: {}", out_b);
    // B should NOT get 02 (that's taken by A)
    assert!(
        out_b.contains("03") && out_b.contains("beta"),
        "B should get 03 beta: {}",
        out_b
    );
}

#[test]
fn test_stale_claim_fails_cleanly() {
    // Clone A closes a ticket, then Clone B (stale) tries to claim it
    let tmp = TempDir::new().unwrap();
    let remote = tmp.path().join("remote.git");
    let clone_a = tmp.path().join("clone-a");
    let clone_b = tmp.path().join("clone-b");

    Command::new("git")
        .args(["init", "--bare", "-q", "-b", "main"])
        .arg(&remote)
        .output()
        .unwrap();

    // Clone A: create an open ticket
    Command::new("git")
        .args(["clone", "-q"])
        .arg(&remote)
        .arg(&clone_a)
        .output()
        .unwrap();
    git(&clone_a, &["config", "user.email", "a@test"]);
    git(&clone_a, &["config", "user.name", "a"]);
    git(&clone_a, &["config", "core.autocrlf", "false"]);
    std::fs::create_dir_all(clone_a.join(".tickets")).unwrap();
    std::fs::write(
        clone_a.join(".tickets/01-target.md"),
        "---\nid: \"01\"\ntitle: \"Target\"\nstatus: open\nblocked_by: []\n---\n\n# Target\n",
    )
    .unwrap();
    git(&clone_a, &["add", "-A"]);
    git(&clone_a, &["commit", "-qm", "seed"]);
    git(&clone_a, &["push", "-q", "origin", "HEAD:main"]);

    // Clone B: from same state
    Command::new("git")
        .args(["clone", "-q"])
        .arg(&remote)
        .arg(&clone_b)
        .output()
        .unwrap();
    git(&clone_b, &["config", "user.email", "b@test"]);
    git(&clone_b, &["config", "user.name", "b"]);
    git(&clone_b, &["config", "core.autocrlf", "false"]);

    // A closes the ticket and pushes
    let (code, _) = run_tkt(&clone_a, &["close", "01"]);
    assert_eq!(code, 0, "A close should succeed");

    // B tries to claim (should fail because preflight fetch reveals ticket is now done)
    let (code_b, out_b) = run_tkt(&clone_b, &["claim", "01"]);
    assert_eq!(code_b, 1, "B claim should fail: {}", out_b);
    assert!(
        out_b.contains("not open") || out_b.contains("done"),
        "should say not open: {}",
        out_b
    );
}

#[test]
fn test_no_remote_works_locally() {
    // A repo with no remote should allow all local operations
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path().join("local-only");

    Command::new("git")
        .args(["init", "-q", "-b", "main"])
        .arg(&dir)
        .output()
        .unwrap();
    git(&dir, &["config", "user.email", "test@test"]);
    git(&dir, &["config", "user.name", "test"]);
    git(&dir, &["config", "core.autocrlf", "false"]);
    std::fs::create_dir_all(dir.join(".tickets")).unwrap();
    std::fs::write(
        dir.join(".tickets/01-local.md"),
        "---\nid: \"01\"\ntitle: \"Local ticket\"\nstatus: open\nblocked_by: []\n---\n\n# Local\n",
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-qm", "init"]);

    // New should work with "no remote" messaging
    let (code, out) = run_tkt(&dir, &["new", "feature", "--title", "A feature"]);
    assert_eq!(code, 0, "new should succeed locally: {}", out);
    assert!(
        out.contains("local only"),
        "should mention local only: {}",
        out
    );

    // Claim should work
    let (code, out) = run_tkt(&dir, &["claim", "01"]);
    assert_eq!(code, 0, "claim should succeed locally: {}", out);

    // Ready should work
    let (code, _) = run_tkt(&dir, &["ready"]);
    assert_eq!(code, 0, "ready should succeed locally");
}

#[test]
fn test_argument_boundary_safety() {
    let (_tmp, clone) = setup_repo();

    // Title with quotes and special chars
    let (code, out) = run_tkt(
        &clone,
        &["new", "special", "--title", "Fix \"ready\" & stuff"],
    );
    assert_eq!(code, 0, "new with special title should succeed: {}", out);

    // Verify the file content is valid
    let files: Vec<_> = std::fs::read_dir(clone.join(".tickets"))
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().contains("special"))
        .collect();
    assert_eq!(files.len(), 1, "should create one file");
    let content = std::fs::read_to_string(files[0].path()).unwrap();
    assert!(content.contains("---"), "should have valid frontmatter");
    assert!(content.contains("title:"), "should have title field");

    // Title with backslash
    let (code, _) = run_tkt(&clone, &["new", "backslash", "--title", "path\\to\\thing"]);
    assert_eq!(code, 0, "backslash title should succeed");
}

#[test]
fn test_push_failure_no_rebase_on_unreachable() {
    // Configure a remote that doesn't exist — push failure should not trigger rebase
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path().join("bad-remote");

    Command::new("git")
        .args(["init", "-q", "-b", "main"])
        .arg(&dir)
        .output()
        .unwrap();
    git(&dir, &["config", "user.email", "test@test"]);
    git(&dir, &["config", "user.name", "test"]);
    git(&dir, &["config", "core.autocrlf", "false"]);
    git(
        &dir,
        &[
            "remote",
            "add",
            "origin",
            "https://nonexistent.invalid/repo.git",
        ],
    );
    std::fs::create_dir_all(dir.join(".tickets")).unwrap();
    std::fs::write(
        dir.join(".tickets/01-test.md"),
        "---\nid: \"01\"\ntitle: \"Test\"\nstatus: open\nblocked_by: []\n---\n\n# Test\n",
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-qm", "init"]);

    // Edit should fail because push to unreachable remote fails
    let (code, out) = run_tkt(&dir, &["edit", "01", "--priority", "high"]);
    assert_ne!(code, 0, "should fail with unreachable remote: {}", out);
    // Should NOT silently succeed
    assert!(
        !out.contains("✓ edited"),
        "should not report success: {}",
        out
    );
}

/// Run tkt with extra environment variables, capturing stdout and stderr separately.
/// Pass empty string as value to remove the variable from the child's environment.
fn run_tkt_env(dir: &Path, args: &[&str], env: &[(&str, &str)]) -> (i32, String, String) {
    let mut cmd = Command::new(tkt_bin());
    cmd.args(args).current_dir(dir);
    // Default: suppress telemetry unless caller explicitly overrides
    if !env.iter().any(|(k, _)| *k == "DO_NOT_TRACK") {
        cmd.env("DO_NOT_TRACK", "1");
    }
    cmd.env_remove("TKT_DEBUG");
    cmd.env("TKT_NO_USER_CONFIG", "1");
    for (k, v) in env {
        if v.is_empty() {
            cmd.env_remove(k);
        } else {
            cmd.env(k, v);
        }
    }
    let output = cmd.output().expect("failed to execute tkt");
    let code = output.status.code().unwrap_or(1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (code, stdout, stderr)
}

#[test]
fn test_debug_mode_emits_to_stderr() {
    let (_tmp, clone) = setup_repo();
    // Add an open ticket so ready has something to show
    std::fs::write(
        clone.join(".tickets/02-open.md"),
        "---\nid: \"02\"\ntitle: \"Open task\"\nstatus: open\nblocked_by: [\"01\"]\n---\n\n# Open\n",
    )
    .unwrap();
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-m", "add open ticket"]);
    git(&clone, &["push"]);

    let (code, stdout, stderr) = run_tkt_env(&clone, &["ready"], &[("TKT_DEBUG", "stderr")]);
    assert_eq!(code, 0);
    // Stdout has normal output
    assert!(
        stdout.contains("02"),
        "stdout should list ticket: {}",
        stdout
    );
    // Stderr has debug trace
    assert!(
        stderr.contains("[tkt:debug]"),
        "stderr should have debug prefix: {}",
        stderr
    );
    assert!(
        stderr.contains("cmd=ready"),
        "stderr should show command: {}",
        stderr
    );
    assert!(
        stderr.contains("exit=0"),
        "stderr should show exit code: {}",
        stderr
    );
    assert!(
        stderr.contains("corpus loaded"),
        "stderr should show corpus stats: {}",
        stderr
    );
}

#[test]
fn test_debug_mode_json_format() {
    let (_tmp, clone) = setup_repo();
    let (code, _stdout, stderr) = run_tkt_env(&clone, &["ready"], &[("TKT_DEBUG", "json")]);
    assert_eq!(code, 0);
    // Stderr should have JSON lines
    let debug_lines: Vec<&str> = stderr
        .lines()
        .filter(|l| l.starts_with('{') && l.contains("\"level\":\"debug\""))
        .collect();
    assert!(
        !debug_lines.is_empty(),
        "should have JSON debug lines in stderr: {}",
        stderr
    );
    // Each line should be valid JSON
    for line in &debug_lines {
        let _: serde_json::Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("invalid JSON in debug output: {} — line: {}", e, line));
    }
}

#[test]
fn test_telemetry_enable_disable_cycle() {
    let (_tmp, clone) = setup_repo();

    // Clear any inherited telemetry env vars for a clean test
    let clean_env: &[(&str, &str)] = &[("TKT_TELEMETRY", ""), ("DO_NOT_TRACK", ""), ("CI", "")];

    // First disable to reset any persisted state from prior runs
    let (code, _, _) = run_tkt_env(&clone, &["telemetry", "--disable"], clean_env);
    assert_eq!(code, 0);

    // Status: should now show disabled
    let (code, out, _) = run_tkt_env(&clone, &["telemetry", "--status"], clean_env);
    assert_eq!(code, 0);
    assert!(out.contains("disabled"), "should be disabled: {}", out);

    // Enable
    let (code, out, _) = run_tkt_env(&clone, &["telemetry", "--enable"], clean_env);
    assert_eq!(code, 0);
    assert!(out.contains("enabled"), "should confirm enable: {}", out);

    // Status: now enabled
    let (code, out, _) = run_tkt_env(&clone, &["telemetry", "--status"], clean_env);
    assert_eq!(code, 0);
    assert!(
        out.contains("enabled") && out.contains("consent.toml"),
        "should show enabled via config: {}",
        out
    );

    // Disable again
    let (code, out, _) = run_tkt_env(&clone, &["telemetry", "--disable"], clean_env);
    assert_eq!(code, 0);
    assert!(out.contains("disabled"), "should confirm disable: {}", out);

    // Status: disabled
    let (code, out, _) = run_tkt_env(&clone, &["telemetry", "--status"], clean_env);
    assert_eq!(code, 0);
    assert!(
        out.contains("disabled"),
        "should be disabled again: {}",
        out
    );
}

#[test]
fn test_close_errors_on_all_unchecked_acs() {
    let (_tmp, clone) = setup_repo();

    // Create a ticket with unchecked ACs
    std::fs::write(
        clone.join(".tickets/02-testclose.md"),
        "---\nid: \"02\"\ntitle: \"Test close\"\nstatus: open\nblocked_by: [\"01\"]\n---\n\n# Test\n\n## Acceptance criteria\n\n- [ ] First AC\n- [ ] Second AC\n",
    )
    .unwrap();
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-m", "add ticket"]);
    git(&clone, &["push"]);

    // Close without --force should fail
    let (code, out) = run_tkt(&clone, &["close", "02", "--note", "Done"]);
    assert_eq!(code, 1, "should fail with all unchecked: {}", out);
    assert!(
        out.contains("unchecked") || out.contains("acceptance"),
        "should mention unchecked ACs: {}",
        out
    );

    // Close with --force should succeed
    let (code, out) = run_tkt(&clone, &["close", "02", "--note", "Done", "--force"]);
    assert_eq!(code, 0, "should succeed with --force: {}", out);
    assert!(out.contains("closed"), "should say closed: {}", out);
}

#[test]
fn test_close_allows_partially_checked_acs() {
    let (_tmp, clone) = setup_repo();

    // Create a ticket with mixed ACs (one checked, one not)
    std::fs::write(
        clone.join(".tickets/02-partial.md"),
        "---\nid: \"02\"\ntitle: \"Partial\"\nstatus: open\nblocked_by: [\"01\"]\n---\n\n# Test\n\n## Acceptance criteria\n\n- [x] Done AC\n- [ ] Pending AC\n",
    )
    .unwrap();
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-m", "add ticket"]);
    git(&clone, &["push"]);

    // Close should succeed (not all unchecked)
    let (code, out) = run_tkt(&clone, &["close", "02", "--note", "Shipped"]);
    assert_eq!(code, 0, "should succeed with partial ACs: {}", out);
    assert!(out.contains("closed"), "should say closed: {}", out);
    assert!(out.contains("1/2 checked"), "should show AC count: {}", out);
}

#[test]
fn test_close_gates_batched() {
    let (_tmp, clone) = setup_repo();

    // Enable multiple gates: resolution + evidence, plus default checked-ACs
    std::fs::write(
        clone.join(".tickets/config.toml"),
        "[close]\nrequire_resolution = true\nrequire_validation_evidence = \"true\"\n",
    )
    .unwrap();
    // Ticket with criteria AND all-unchecked ACs
    std::fs::write(
        clone.join(".tickets/02-multi.md"),
        "---\nid: \"02\"\ntitle: \"Multi\"\nstatus: open\nblocked_by: []\nvalidation_criteria:\n  - \"must verify\"\n---\n\n# Multi\n\n## Acceptance criteria\n\n- [ ] A\n- [ ] B\n",
    )
    .unwrap();
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-qm", "add multi"]);
    git(&clone, &["push", "-q", "origin", "HEAD:main"]);

    // Bare close: THREE gates unmet (resolution, evidence, unchecked ACs).
    // They must all be reported in ONE message.
    let (code, out) = run_tkt(&clone, &["close", "02"]);
    assert_eq!(code, 1, "should be blocked: {}", out);
    assert!(
        out.contains("3 unmet gate"),
        "should batch all three gates: {}",
        out
    );
    assert!(out.contains("--resolution"), "names resolution: {}", out);
    assert!(out.contains("--evidence"), "names evidence: {}", out);
    assert!(
        out.contains("--check-all") || out.contains("acceptance criteria"),
        "names AC remedy: {}",
        out
    );
}

#[test]
fn test_close_gate_hint_populated() {
    let (_tmp, clone) = setup_repo();

    std::fs::write(
        clone.join(".tickets/config.toml"),
        "[close]\nrequire_resolution = true\n",
    )
    .unwrap();
    std::fs::write(
        clone.join(".tickets/02-hint.md"),
        "---\nid: \"02\"\ntitle: \"Hint\"\nstatus: open\nblocked_by: []\n---\n\n# Hint\n\n## Acceptance criteria\n\n- [x] A\n",
    )
    .unwrap();
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-qm", "add hint ticket"]);
    git(&clone, &["push", "-q", "origin", "HEAD:main"]);

    // JSON envelope must carry a hint naming the missing flag
    let (code, out) = run_tkt(&clone, &["-o", "json", "close", "02"]);
    assert_eq!(code, 1, "should be blocked: {}", out);
    assert!(
        out.contains("\"hint\""),
        "gate error should populate a hint: {}",
        out
    );
    assert!(
        out.contains("\"kind\":\"gate_failed\""),
        "should be gate_failed kind: {}",
        out
    );
}

#[test]
fn test_close_partial_evidence_kind_is_gate_failed() {
    let (_tmp, clone) = setup_repo();

    std::fs::write(
        clone.join(".tickets/config.toml"),
        "[close]\nrequire_validation_evidence = \"true\"\n",
    )
    .unwrap();
    // Two criteria, but we'll supply evidence for only one → partial evidence gate (G5)
    std::fs::write(
        clone.join(".tickets/02-partialev.md"),
        "---\nid: \"02\"\ntitle: \"PartialEv\"\nstatus: open\nblocked_by: []\nvalidation_criteria:\n  - \"first\"\n  - \"second\"\n---\n\n# PartialEv\n\n## Acceptance criteria\n\n- [x] A\n",
    )
    .unwrap();
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-qm", "add partialev"]);
    git(&clone, &["push", "-q", "origin", "HEAD:main"]);

    // Supply 1 of 2 evidence items → G5 partial-evidence gate fires.
    // Must surface as gate_failed (previously mis-kinded as validation).
    let (code, out) = run_tkt(
        &clone,
        &[
            "-o",
            "json",
            "close",
            "02",
            "--note",
            "done",
            "--evidence",
            "1=only first",
        ],
    );
    assert_eq!(code, 1, "partial evidence should be blocked: {}", out);
    assert!(
        out.contains("\"kind\":\"gate_failed\""),
        "G5 must be gate_failed, not validation: {}",
        out
    );
}

#[test]
fn test_close_resolution_flag_works() {
    let (_tmp, clone) = setup_repo();

    // Create a ticket with no ACs (body without checkboxes)
    std::fs::write(
        clone.join(".tickets/02-noacflag.md"),
        "---\nid: \"02\"\ntitle: \"No AC\"\nstatus: open\nblocked_by: [\"01\"]\n---\n\n# No AC ticket\n\nJust a description.\n",
    )
    .unwrap();
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-m", "add ticket"]);
    git(&clone, &["push"]);

    // Close with --resolution flag
    let (code, out) = run_tkt(
        &clone,
        &["close", "02", "--resolution", "Implemented the feature"],
    );
    assert_eq!(code, 0, "should succeed: {}", out);
    assert!(out.contains("closed"), "should say closed: {}", out);

    // Verify resolution text in file
    let content = std::fs::read_to_string(clone.join(".tickets/02-noacflag.md")).unwrap();
    assert!(content.contains("Implemented the feature"));
}

#[test]
fn test_close_ac_flag_bypasses_all_unchecked_error() {
    let (_tmp, clone) = setup_repo();

    // Create a ticket with all unchecked ACs
    std::fs::write(
        clone.join(".tickets/02-acbypass.md"),
        "---\nid: \"02\"\ntitle: \"AC bypass\"\nstatus: open\nblocked_by: [\"01\"]\n---\n\n# Test\n\n## Acceptance criteria\n\n- [ ] First\n- [ ] Second\n",
    )
    .unwrap();
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-m", "add ticket"]);
    git(&clone, &["push"]);

    // Close with --ac 1 should succeed (checks one, so not all-unchecked after)
    let (code, out) = run_tkt(&clone, &["close", "02", "--note", "Done", "--ac", "1"]);
    assert_eq!(code, 0, "should succeed with --ac: {}", out);
    assert!(out.contains("1/2 checked"), "should show AC count: {}", out);
}

#[test]
fn test_close_shows_unblocked_tickets() {
    let (_tmp, clone) = setup_repo();

    // Create ticket 02 (open, blocked by 01 which is done) and 03 (blocked by 02)
    std::fs::write(
        clone.join(".tickets/02-blocker.md"),
        "---\nid: \"02\"\ntitle: \"Blocker\"\nstatus: open\nblocked_by: [\"01\"]\n---\n\n# Blocker\n\n## Acceptance criteria\n\n- [ ] Done\n",
    ).unwrap();
    std::fs::write(
        clone.join(".tickets/03-blocked.md"),
        "---\nid: \"03\"\ntitle: \"Was Blocked\"\nstatus: open\nblocked_by: [\"02\"]\n---\n\n# Was Blocked\n\n## Acceptance criteria\n\n- [ ] Done\n",
    ).unwrap();
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-qm", "add tickets"]);
    git(&clone, &["push"]);

    // Close 02 → should unblock 03
    let (code, out) = run_tkt(
        &clone,
        &["close", "02", "--check-all", "--resolution", "Done"],
    );
    assert_eq!(code, 0, "close should succeed: {}", out);
    assert!(
        out.contains("unblocked"),
        "should show unblocked tickets: {}",
        out
    );
    assert!(
        out.contains("03") && out.contains("Was Blocked"),
        "should name the unblocked ticket: {}",
        out
    );
}

#[test]
fn test_ready_hierarchy_format() {
    let (_tmp, clone) = setup_repo();

    // Add two open tickets
    std::fs::write(
        clone.join(".tickets/02-alpha.md"),
        "---\nid: \"02\"\ntitle: \"Alpha\"\nstatus: open\nblocked_by: [\"01\"]\n---\n\n# Alpha\n",
    )
    .unwrap();
    std::fs::write(
        clone.join(".tickets/03-beta.md"),
        "---\nid: \"03\"\ntitle: \"Beta\"\nstatus: open\nblocked_by: [\"01\"]\npriority: high\n---\n\n# Beta\n",
    ).unwrap();
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-qm", "add tickets"]);

    let (code, out) = run_tkt(&clone, &["ready"]);
    assert_eq!(code, 0, "ready should succeed: {}", out);
    assert!(
        out.contains("Ready (2):"),
        "should show header with count: {}",
        out
    );
    assert!(
        out.contains("  03") && out.contains("  02"),
        "should show indented items: {}",
        out
    );
    assert!(out.contains("[HIGH]"), "should show priority flag: {}", out);
}

#[test]
fn test_new_quiet_outputs_bare_id() {
    let (_tmp, clone) = setup_repo();

    let (code, out) = run_tkt(
        &clone,
        &["new", "quiet-test", "--title", "Quiet Test", "-q"],
    );
    assert_eq!(code, 0, "new -q should succeed: {}", out);
    let stdout_only = String::from_utf8_lossy(
        &Command::new(tkt_bin())
            .args(["new", "another", "--title", "Another", "-q"])
            .current_dir(&clone)
            .env("DO_NOT_TRACK", "1")
            .env("TKT_NO_USER_CONFIG", "1")
            .output()
            .unwrap()
            .stdout,
    )
    .to_string();
    let trimmed = stdout_only.trim();
    // Should be exactly a numeric ID with no other text
    assert!(
        trimmed.chars().all(|c| c.is_ascii_digit()),
        "quiet output should be bare ID only, got: {:?}",
        trimmed
    );
    assert!(!trimmed.is_empty(), "quiet output should not be empty");
}

#[test]
fn test_audit_reports_quality_issues() {
    let (_tmp, clone) = setup_repo();

    // Create a done ticket with all ACs unchecked and no resolution
    std::fs::write(
        clone.join(".tickets/02-bad.md"),
        "---\nid: \"02\"\ntitle: \"Badly closed\"\nstatus: done\nblocked_by: []\n---\n\n# Badly closed\n\n## Acceptance criteria\n\n- [ ] Never checked\n- [ ] Also unchecked\n",
    ).unwrap();
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-qm", "add bad ticket"]);

    // Audit should find issues
    let (code, out) = run_tkt(&clone, &["audit", "--brief"]);
    assert_eq!(code, 0, "audit should pass (warnings only): {}", out);
    assert!(
        out.contains("all-acs-unchecked-on-done"),
        "should report unchecked ACs: {}",
        out
    );
    assert!(
        out.contains("missing-resolution"),
        "should report missing resolution: {}",
        out
    );
    assert!(
        out.contains("02-bad.md"),
        "should name the offending file: {}",
        out
    );
}

#[test]
fn test_rebase_resolves_id_collision() {
    let tmp = TempDir::new().unwrap();
    let remote = tmp.path().join("remote.git");
    let clone_a = tmp.path().join("clone_a");
    let clone_b = tmp.path().join("clone_b");

    // Create bare remote
    Command::new("git")
        .args(["init", "--bare", "-q", "-b", "main"])
        .arg(&remote)
        .output()
        .unwrap();

    // Clone A and B
    for clone in [&clone_a, &clone_b] {
        Command::new("git")
            .args(["clone", "-q"])
            .arg(&remote)
            .arg(clone)
            .output()
            .unwrap();
        git(clone, &["config", "user.email", "test@test"]);
        git(clone, &["config", "user.name", "test"]);
    }

    // Seed a ticket in A and push
    std::fs::create_dir_all(clone_a.join(".tickets")).unwrap();
    std::fs::write(
        clone_a.join(".tickets/01-seed.md"),
        "---\nid: \"01\"\ntitle: \"Seed\"\nstatus: done\nblocked_by: []\n---\n\n# Seed\n",
    )
    .unwrap();
    git(&clone_a, &["add", "-A"]);
    git(&clone_a, &["commit", "-qm", "seed"]);
    git(&clone_a, &["push", "-q"]);

    // A creates ticket 02 and pushes (claims it)
    std::fs::write(
        clone_a.join(".tickets/02-alpha.md"),
        "---\nid: \"02\"\ntitle: \"Alpha\"\nstatus: open\nblocked_by: []\n---\n\n# Alpha\n",
    )
    .unwrap();
    git(&clone_a, &["add", "-A"]);
    git(&clone_a, &["commit", "-qm", "alpha"]);
    git(&clone_a, &["push", "-q"]);

    // B (stale, doesn't know about A's 02) creates its own 02
    std::fs::create_dir_all(clone_b.join(".tickets")).unwrap();
    // First pull the seed
    git(&clone_b, &["pull", "-q"]);
    // Now create a conflicting 02 locally (without fetching A's push)
    std::fs::write(
        clone_b.join(".tickets/02-beta.md"),
        "---\nid: \"02\"\ntitle: \"Beta\"\nstatus: open\nblocked_by: []\n---\n\n# Beta\n",
    )
    .unwrap();
    // Also create 03 that depends on 02
    std::fs::write(
        clone_b.join(".tickets/03-gamma.md"),
        "---\nid: \"03\"\ntitle: \"Gamma\"\nstatus: open\nblocked_by: [\"02\"]\n---\n\n# Gamma\n",
    )
    .unwrap();
    git(&clone_b, &["add", "-A"]);
    git(&clone_b, &["commit", "-qm", "beta and gamma"]);

    // Now A pushes another ticket (so B is behind)
    // B runs tkt rebase — should detect collision on 02 and renumber
    let (code, out) = run_tkt(&clone_b, &["rebase", "--dry-run"]);
    assert_eq!(code, 0, "rebase --dry-run should succeed: {}", out);
    assert!(
        out.contains("02") && out.contains("beta"),
        "should identify the collision: {}",
        out
    );

    // Now do the real rebase
    let (code, out) = run_tkt(&clone_b, &["rebase"]);
    assert_eq!(code, 0, "rebase should succeed: {}", out);
    assert!(
        out.contains("Renumbered"),
        "should report renumbering: {}",
        out
    );

    // Verify: 02-beta.md should no longer exist, a new ID should
    assert!(
        !clone_b.join(".tickets/02-beta.md").exists(),
        "old file should be gone"
    );
    // The new file should be 03-beta.md or 04-beta.md (next available)
    let files: Vec<String> = std::fs::read_dir(clone_b.join(".tickets"))
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.contains("beta"))
        .collect();
    assert_eq!(
        files.len(),
        1,
        "should have exactly one beta file: {:?}",
        files
    );
    let beta_file = &files[0];
    assert!(
        !beta_file.starts_with("02-"),
        "beta should have a new ID, not 02: {}",
        beta_file
    );

    // Verify blocked_by was updated in gamma
    let gamma_content = std::fs::read_to_string(clone_b.join(".tickets/03-gamma.md")).unwrap();
    // gamma's blocked_by should now reference the new beta ID, not "02"
    assert!(
        !gamma_content.contains("blocked_by: [\"02\"]"),
        "gamma's blocked_by should be updated: {}",
        gamma_content
    );
}

// --- Config command tests ---

/// Run tkt with a custom XDG_CONFIG_HOME (isolates config file from real user).
fn run_tkt_with_config(dir: &Path, args: &[&str], config_home: &Path) -> (i32, String) {
    let output = Command::new(tkt_bin())
        .args(args)
        .current_dir(dir)
        .env("DO_NOT_TRACK", "1")
        .env_remove("TKT_NO_USER_CONFIG")
        .env_remove("TKT_DEBUG")
        .env_remove("TKT_DEBUG_FORMAT")
        .env("XDG_CONFIG_HOME", config_home)
        .output()
        .expect("failed to execute tkt");
    let code = output.status.code().unwrap_or(1);
    let out = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    (code, out)
}

#[test]
fn test_config_set_get_list() {
    let (_tmp, clone) = setup_repo();
    let config_dir = _tmp.path().join("config-home");

    // Initially, get returns default
    let (code, out) = run_tkt_with_config(&clone, &["config", "--get", "debug"], &config_dir);
    assert_eq!(code, 0);
    assert_eq!(out.trim(), "false");

    // Set debug = true
    let (code, out) = run_tkt_with_config(&clone, &["config", "--set", "debug=true"], &config_dir);
    assert_eq!(code, 0);
    assert!(out.contains("debug"), "set output: {}", out);

    // Get now returns true
    let output = Command::new(tkt_bin())
        .args(["config", "--get", "debug"])
        .current_dir(&clone)
        .env("DO_NOT_TRACK", "1")
        .env_remove("TKT_NO_USER_CONFIG")
        .env_remove("TKT_DEBUG")
        .env_remove("TKT_DEBUG_FORMAT")
        .env("XDG_CONFIG_HOME", &config_dir)
        .output()
        .expect("failed to execute tkt");
    assert_eq!(output.status.code().unwrap_or(1), 0);
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "true");

    // List shows the config entry
    let (code, out) = run_tkt_with_config(&clone, &["config", "--list"], &config_dir);
    assert_eq!(code, 0);
    assert!(out.contains("debug"), "list output: {}", out);
    assert!(out.contains("config"), "should show config source: {}", out);
}

#[test]
fn test_config_unset_reverts_to_default() {
    let (_tmp, clone) = setup_repo();
    let config_dir = _tmp.path().join("config-home");

    // Set then unset
    run_tkt_with_config(&clone, &["config", "--set", "debug=true"], &config_dir);
    let (code, out) = run_tkt_with_config(&clone, &["config", "--unset", "debug"], &config_dir);
    assert_eq!(code, 0);
    assert!(out.contains("unset"), "unset output: {}", out);

    // Get returns default again
    let (code, out) = run_tkt_with_config(&clone, &["config", "--get", "debug"], &config_dir);
    assert_eq!(code, 0);
    assert_eq!(out.trim(), "false");
}

#[test]
fn test_config_env_overrides_config_file() {
    let (_tmp, clone) = setup_repo();
    let config_dir = _tmp.path().join("config-home");

    // Set debug=false in config
    run_tkt_with_config(&clone, &["config", "--set", "debug=false"], &config_dir);

    // But env var should override — run with TKT_DEBUG=1 and check output
    let output = Command::new(tkt_bin())
        .args(["config", "--get", "debug"])
        .current_dir(&clone)
        .env("DO_NOT_TRACK", "1")
        .env("TKT_NO_USER_CONFIG", "1")
        .env("TKT_DEBUG", "1")
        .env("XDG_CONFIG_HOME", &config_dir)
        .output()
        .expect("failed to execute tkt");
    let out = String::from_utf8_lossy(&output.stdout).to_string();
    // config --get respects precedence: env TKT_DEBUG=1 → returns "1"
    assert_eq!(out.trim(), "1");
}

#[test]
fn test_config_debug_enables_debug_output() {
    let (_tmp, clone) = setup_repo();
    let config_dir = _tmp.path().join("config-home");

    // Enable debug via config
    run_tkt_with_config(&clone, &["config", "--set", "debug=true"], &config_dir);

    // Run ready — should produce debug output on stderr
    let output = Command::new(tkt_bin())
        .args(["ready"])
        .current_dir(&clone)
        .env("DO_NOT_TRACK", "1")
        .env_remove("TKT_NO_USER_CONFIG")
        .env_remove("TKT_DEBUG")
        .env("XDG_CONFIG_HOME", &config_dir)
        .output()
        .expect("failed to execute tkt");
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(
        stderr.contains("[tkt:debug]"),
        "debug from config should produce debug output: stderr={:?}",
        stderr
    );
}

// --- Project config tests ---

#[test]
fn test_project_config_require_resolution_blocks_bare_close() {
    let (_tmp, clone) = setup_repo();

    // Add project config requiring resolution
    std::fs::write(
        clone.join(".tickets/config.toml"),
        "[close]\nrequire_resolution = true\n",
    )
    .unwrap();

    // Add an open ticket
    std::fs::write(
        clone.join(".tickets/02-feature.md"),
        "---\nid: \"02\"\ntitle: \"Feature\"\nstatus: open\nblocked_by: []\n---\n\n# Feature\n\n## Acceptance criteria\n\n- [x] Done\n",
    ).unwrap();
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-qm", "add config and ticket"]);
    git(&clone, &["push", "-q", "origin", "HEAD:main"]);

    // Try to close without --resolution — should fail
    let (code, out) = run_tkt(&clone, &["close", "02"]);
    assert_eq!(code, 1, "bare close should fail: {}", out);
    assert!(
        out.contains("--resolution"),
        "should mention resolution requirement: {}",
        out
    );

    // Close with --resolution — should succeed
    let (code, out) = run_tkt(&clone, &["close", "02", "--resolution", "Done"]);
    assert_eq!(code, 0, "close with resolution should succeed: {}", out);
}

#[test]
fn test_project_config_push_disabled_skips_push() {
    let (_tmp, clone) = setup_repo();

    // Add project config disabling push
    std::fs::write(
        clone.join(".tickets/config.toml"),
        "[push]\nenabled = false\n",
    )
    .unwrap();
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-qm", "add config"]);
    git(&clone, &["push", "-q", "origin", "HEAD:main"]);

    // Add an open ticket
    std::fs::write(
        clone.join(".tickets/02-task.md"),
        "---\nid: \"02\"\ntitle: \"Task\"\nstatus: open\nblocked_by: []\n---\n\n# Task\n\n## Acceptance criteria\n\n- [x] AC\n",
    ).unwrap();
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-qm", "add task"]);
    git(&clone, &["push", "-q", "origin", "HEAD:main"]);

    // Close the ticket — should succeed locally without pushing
    let (code, out) = run_tkt(&clone, &["close", "02", "--resolution", "Done"]);
    assert_eq!(code, 0, "close should succeed: {}", out);

    // Verify the close commit is local but NOT pushed
    let local_head = git(&clone, &["rev-parse", "HEAD"]);
    let remote_head = git(&clone, &["rev-parse", "origin/main"]);
    assert_ne!(
        local_head, remote_head,
        "push.enabled=false should skip push (local should be ahead)"
    );
}

#[test]
fn test_project_config_show_dumps_settings() {
    let (_tmp, clone) = setup_repo();

    // Add project config
    std::fs::write(
        clone.join(".tickets/config.toml"),
        "[close]\nrequire_resolution = true\n\n[push]\nenabled = false\n",
    )
    .unwrap();
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-qm", "add config"]);

    let (code, out) = run_tkt(&clone, &["config", "--show"]);
    assert_eq!(code, 0, "config --show should succeed: {}", out);
    assert!(
        out.contains("close.require_resolution"),
        "should show close.require_resolution: {}",
        out
    );
    assert!(
        out.contains("push.enabled"),
        "should show push.enabled: {}",
        out
    );
    assert!(
        out.contains("project"),
        "should annotate source as project: {}",
        out
    );
}

#[test]
fn test_project_config_unknown_key_warns() {
    let (_tmp, clone) = setup_repo();

    // Add project config with unknown key
    std::fs::write(
        clone.join(".tickets/config.toml"),
        "[mystery]\nfoo = true\n",
    )
    .unwrap();
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-qm", "add config"]);

    let (code, out) = run_tkt(&clone, &["config", "--show"]);
    assert_eq!(code, 0, "should succeed with warning: {}", out);
    assert!(
        out.contains("unknown config key"),
        "should warn about unknown key: {}",
        out
    );
}

// --- Color/symbol tests ---

#[test]
fn test_no_color_produces_no_ansi() {
    let (_tmp, clone) = setup_repo();

    // Run with NO_COLOR=1 — output should contain no ANSI escape codes
    let (code, _stdout, stderr) = run_tkt_env(&clone, &["close", "01"], &[("NO_COLOR", "1")]);
    // Ticket 01 is already done, so this will produce an error
    assert_eq!(code, 1);
    // Error output should have no ANSI escape sequences
    assert!(
        !stderr.contains("\x1b["),
        "NO_COLOR=1 should suppress ANSI codes in stderr: {:?}",
        stderr
    );
}

#[test]
fn test_color_always_produces_ansi() {
    let (_tmp, clone) = setup_repo();

    // Run with --color=always — error output should contain ANSI codes
    let (code, _stdout, stderr) = run_tkt_env(&clone, &["--color=always", "close", "01"], &[]);
    assert_eq!(code, 1);
    // Error output should have ANSI escape sequences (red for ✗)
    assert!(
        stderr.contains("\x1b["),
        "--color=always should produce ANSI codes in stderr: {:?}",
        stderr
    );
}

#[test]
fn test_ascii_mode_produces_ascii_symbols() {
    let (_tmp, clone) = setup_repo();

    // Run with TKT_ASCII=1 — should use [err] instead of ✗
    let (code, _stdout, stderr) = run_tkt_env(&clone, &["close", "01"], &[("TKT_ASCII", "1")]);
    assert_eq!(code, 1);
    assert!(
        stderr.contains("[err]"),
        "TKT_ASCII=1 should produce [err] instead of ✗: {:?}",
        stderr
    );
    assert!(
        !stderr.contains("✗"),
        "TKT_ASCII=1 should NOT contain ✗: {:?}",
        stderr
    );
}

#[test]
fn test_error_output_includes_program_name() {
    let (_tmp, clone) = setup_repo();

    let (code, _stdout, stderr) = run_tkt_env(&clone, &["close", "01"], &[]);
    assert_eq!(code, 1);
    assert!(
        stderr.contains("tkt:"),
        "error output should include program name: {:?}",
        stderr
    );
}

// --- Filter and view tests ---

#[test]
fn test_query_status_filter() {
    let (_tmp, clone) = setup_repo();

    // Seed has status: done. Add an open ticket.
    std::fs::write(
        clone.join(".tickets/02-open.md"),
        "---\nid: \"02\"\ntitle: \"Open task\"\nstatus: open\nblocked_by: []\n---\n\n# Open\n",
    )
    .unwrap();
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-qm", "add open"]);

    // Filter for done — should return only ticket 01
    let (code, out) = run_tkt(&clone, &["query", "--status", "done"]);
    assert_eq!(code, 0);
    assert!(
        out.contains("\"01\""),
        "should include done ticket: {}",
        out
    );
    assert!(
        !out.contains("\"02\""),
        "should exclude open ticket: {}",
        out
    );

    // Filter for open — should return only ticket 02
    let (code, out) = run_tkt(&clone, &["query", "--status", "open"]);
    assert_eq!(code, 0);
    assert!(
        out.contains("\"02\""),
        "should include open ticket: {}",
        out
    );
    assert!(
        !out.contains("\"01\""),
        "should exclude done ticket: {}",
        out
    );
}

#[test]
fn test_blocked_shows_blockers() {
    let (_tmp, clone) = setup_repo();

    // Add a ticket blocked by the done seed (should be on frontier, NOT blocked)
    std::fs::write(
        clone.join(".tickets/02-unblocked.md"),
        "---\nid: \"02\"\ntitle: \"Unblocked\"\nstatus: open\nblocked_by: [\"01\"]\n---\n\n# Unblocked\n",
    )
    .unwrap();
    // Add a ticket blocked by something NOT done
    std::fs::write(
        clone.join(".tickets/03-blocked.md"),
        "---\nid: \"03\"\ntitle: \"Blocked task\"\nstatus: open\nblocked_by: [\"02\"]\n---\n\n# Blocked\n",
    )
    .unwrap();
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-qm", "add tickets"]);

    let (code, out) = run_tkt(&clone, &["blocked"]);
    assert_eq!(code, 0);
    assert!(
        out.contains("03") && out.contains("Blocked task"),
        "should show blocked ticket 03: {}",
        out
    );
    assert!(
        out.contains("blocked by: 02"),
        "should show the blocker: {}",
        out
    );
    assert!(
        !out.contains("02  Unblocked"),
        "should NOT show unblocked ticket 02 as blocked: {}",
        out
    );
}

#[test]
fn test_validate_fix_quotes_ids_and_removes_invalid_env() {
    let (_tmp, clone) = setup_repo();

    // Create a ticket with unquoted id and invalid env
    std::fs::write(
        clone.join(".tickets/02-unquoted.md"),
        "---\nid: 02\ntitle: \"Unquoted\"\nstatus: open\nblocked_by: [01]\nenv: custom-invalid\n---\n\n# Unquoted\n",
    )
    .unwrap();
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-qm", "add unquoted ticket"]);

    // Dry run first
    let (code, out) = run_tkt(&clone, &["validate", "--fix", "--dry-run"]);
    assert_eq!(code, 0, "dry-run should succeed: {}", out);
    assert!(
        out.contains("Would fix"),
        "should show dry-run plan: {}",
        out
    );
    assert!(
        out.contains("quoted id"),
        "should plan to quote id: {}",
        out
    );
    assert!(
        out.contains("removed invalid env"),
        "should plan to remove env: {}",
        out
    );

    // Verify file NOT modified (dry-run)
    let content = std::fs::read_to_string(clone.join(".tickets/02-unquoted.md")).unwrap();
    assert!(content.contains("id: 02"), "dry-run should not modify file");

    // Now apply
    let (code, out) = run_tkt(&clone, &["validate", "--fix"]);
    assert_eq!(code, 0, "fix should succeed: {}", out);
    assert!(out.contains("Fixed"), "should report fixes: {}", out);

    // Verify file IS modified
    let content = std::fs::read_to_string(clone.join(".tickets/02-unquoted.md")).unwrap();
    assert!(
        content.contains("id: \"02\""),
        "id should be quoted: {}",
        content
    );
    assert!(
        !content.contains("env:"),
        "invalid env should be removed: {}",
        content
    );
    assert!(
        content.contains("blocked_by: [\"01\"]"),
        "blocked_by should be quoted: {}",
        content
    );
}

#[test]
fn test_fix_normalizes_blocked_by_padding_and_slug() {
    // #162: validate --fix and lint --fix resolve underpadded + slug blocked_by
    // refs against the corpus, and leave genuinely-dangling refs untouched.
    let (_tmp, clone) = setup_repo();
    // Corpus already seeds 01-seed.md (id "01").
    std::fs::write(
        clone.join(".tickets/05-target.md"),
        "---\nid: \"05\"\ntitle: \"Target\"\nstatus: open\nblocked_by: []\n---\n\n# Target\n",
    )
    .unwrap();
    // Underpadded ref "5" (should pad to "05") and a dangling "99" (left alone).
    std::fs::write(
        clone.join(".tickets/06-padref.md"),
        "---\nid: \"06\"\ntitle: \"PadRef\"\nstatus: open\nblocked_by: [\"5\", \"99\"]\n---\n\n# PadRef\n",
    )
    .unwrap();
    // Numeric-prefixed slug ref -> bare id "05".
    std::fs::write(
        clone.join(".tickets/07-slugref.md"),
        "---\nid: \"07\"\ntitle: \"SlugRef\"\nstatus: open\nblocked_by: [\"05-target\"]\n---\n\n# SlugRef\n",
    )
    .unwrap();
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-qm", "add ref tickets"]);

    // Before: validate flags dangling-by-format.
    let (code, out) = run_tkt(&clone, &["validate"]);
    assert_eq!(code, 1, "validate should flag dangling refs: {}", out);
    assert!(
        out.contains("dangling-blocked-by"),
        "expect dangling: {}",
        out
    );

    // Apply the fix. Post-fix validation re-runs and still reports the genuinely
    // dangling "99" (correctly left alone), so the overall exit is 1 — but the
    // resolvable refs are repaired.
    let (_code, out) = run_tkt(&clone, &["validate", "--fix"]);
    assert!(out.contains("Fixed"), "should report fixes: {}", out);
    assert!(
        out.contains("dangling-blocked-by"),
        "99 still dangling after fix: {}",
        out
    );

    let padref = std::fs::read_to_string(clone.join(".tickets/06-padref.md")).unwrap();
    assert!(
        padref.contains("blocked_by: [\"05\", \"99\"]"),
        "5 padded to 05, 99 left dangling: {}",
        padref
    );
    let slugref = std::fs::read_to_string(clone.join(".tickets/07-slugref.md")).unwrap();
    assert!(
        slugref.contains("blocked_by: [\"05\"]"),
        "slug ref resolved to 05: {}",
        slugref
    );

    // Idempotence: a second fix makes no further change to the resolved refs.
    let (_code, _) = run_tkt(&clone, &["validate", "--fix"]);
    let padref2 = std::fs::read_to_string(clone.join(".tickets/06-padref.md")).unwrap();
    assert!(
        padref2.contains("blocked_by: [\"05\", \"99\"]"),
        "idempotent: {}",
        padref2
    );

    // lint --check should now agree there is nothing left to normalize on 07.
    let (code, _) = run_tkt(&clone, &["lint", "--check", "07"]);
    assert_eq!(code, 0, "07 should be canonical after fix");
}

#[test]
fn test_fix_regression_gate_aborts_on_new_finding() {
    // #166: a fix that introduces a NEW finding must abort with exit 1 and advise revert.
    // Natural case: status closed->done (Tier-2 map) introduces missing-resolution on a
    // done ticket that has no Resolution section.
    let (_tmp, clone) = setup_repo();
    std::fs::write(
        clone.join(".tickets/02-closed.md"),
        "---\nid: \"02\"\ntitle: \"Closed\"\nstatus: closed\nblocked_by: []\n---\n\n# Closed\n",
    )
    .unwrap();
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-qm", "add closed ticket"]);

    let (code, out) = run_tkt(&clone, &["validate", "--fix"]);
    assert_eq!(code, 1, "regression must abort with exit 1: {}", out);
    assert!(
        out.contains("introduced") && out.contains("new finding"),
        "should report the regression: {}",
        out
    );
    assert!(
        out.contains("git checkout"),
        "should advise revert: {}",
        out
    );
}

#[test]
fn test_validation_criteria_and_evidence_flow() {
    let (_tmp, clone) = setup_repo();

    // Create a ticket with validation criteria
    let (code, out) = run_tkt(
        &clone,
        &[
            "new",
            "auth",
            "--title",
            "Implement auth",
            "--validation",
            "cargo test passes",
            "--validation",
            "login returns JWT",
        ],
    );
    assert_eq!(code, 0, "new should succeed: {}", out);

    // Verify the file has validation_criteria
    let content = std::fs::read_to_string(clone.join(".tickets/02-auth.md")).unwrap();
    assert!(
        content.contains("validation_criteria:"),
        "should have vc field: {}",
        content
    );
    assert!(
        content.contains("cargo test passes"),
        "should have first criterion: {}",
        content
    );

    // Close with evidence (positional)
    let (code, out) = run_tkt(
        &clone,
        &[
            "close",
            "02",
            "--note",
            "All verified",
            "--evidence",
            "49 passed, 0 failed",
            "--evidence",
            "POST /login returns JWT",
            "--check-all",
        ],
    );
    assert_eq!(code, 0, "close with evidence should succeed: {}", out);

    // Verify the resolution section has evidence
    let content = std::fs::read_to_string(clone.join(".tickets/02-auth.md")).unwrap();
    assert!(
        content.contains("### Verification"),
        "should have Verification section: {}",
        content
    );
    assert!(
        content.contains("✓ cargo test passes"),
        "should have first criterion with checkmark: {}",
        content
    );
    assert!(
        content.contains("49 passed, 0 failed"),
        "should have first evidence: {}",
        content
    );
}

#[test]
fn test_evidence_named_mapping() {
    let (_tmp, clone) = setup_repo();

    // Create ticket with 3 criteria
    let (code, _) = run_tkt(
        &clone,
        &[
            "new",
            "multi",
            "--title",
            "Multi criteria",
            "--validation",
            "A passes",
            "--validation",
            "B passes",
            "--validation",
            "C passes",
        ],
    );
    assert_eq!(code, 0);

    // Close with named evidence (out of order)
    let (code, out) = run_tkt(
        &clone,
        &[
            "close",
            "02",
            "--note",
            "Done",
            "--evidence",
            "3=C verified",
            "--evidence",
            "1=A verified",
            "--evidence",
            "2=B verified",
            "--check-all",
        ],
    );
    assert_eq!(code, 0, "named evidence should succeed: {}", out);

    let content = std::fs::read_to_string(clone.join(".tickets/02-multi.md")).unwrap();
    assert!(
        content.contains("✓ A passes — \"A verified\""),
        "criterion 1 should map to evidence 1: {}",
        content
    );
    assert!(
        content.contains("✓ C passes — \"C verified\""),
        "criterion 3 should map to evidence 3: {}",
        content
    );
}

#[test]
fn test_evidence_gate_blocks_when_configured() {
    let (_tmp, clone) = setup_repo();

    // Set config to require evidence
    std::fs::create_dir_all(clone.join(".tickets")).ok();
    std::fs::write(
        clone.join(".tickets/config.toml"),
        "[close]\nrequire_validation_evidence = \"true\"\n",
    )
    .unwrap();

    // Create ticket with criteria
    let (code, _) = run_tkt(
        &clone,
        &[
            "new",
            "gated",
            "--title",
            "Gated ticket",
            "--validation",
            "must verify",
        ],
    );
    assert_eq!(code, 0);

    // Try to close without evidence — should fail
    let (code, out) = run_tkt(&clone, &["close", "02", "--note", "Done", "--check-all"]);
    assert_eq!(code, 1, "should be blocked: {}", out);
    assert!(
        out.contains("no --evidence provided"),
        "should explain why blocked: {}",
        out
    );

    // Close with --force should work
    let (code, out) = run_tkt(
        &clone,
        &["close", "02", "--note", "Done", "--check-all", "--force"],
    );
    assert_eq!(code, 0, "force should override: {}", out);
}

#[test]
fn test_evidence_duplicate_named_index_rejected() {
    let (_tmp, clone) = setup_repo();
    // Create a ticket with 2 validation criteria
    let ticket_content = "---\nid: \"02\"\ntitle: \"Two criteria\"\nstatus: open\nblocked_by: []\nvalidation_criteria:\n  - \"first check\"\n  - \"second check\"\n---\n\n# Two criteria\n";
    std::fs::write(clone.join(".tickets/02-two-criteria.md"), ticket_content).unwrap();
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-qm", "add ticket"]);

    // Enable evidence gate
    std::fs::write(
        clone.join(".tickets/config.toml"),
        "[close]\nrequire_validation_evidence = \"true\"\n",
    )
    .unwrap();
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-qm", "config"]);

    // Try to close with duplicate named index (1=foo, 1=bar) — should fail
    let (code, out) = run_tkt(
        &clone,
        &[
            "close",
            "02",
            "--check-all",
            "--note",
            "done",
            "--evidence",
            "1=first evidence",
            "--evidence",
            "1=duplicate first",
        ],
    );
    assert_eq!(code, 1, "duplicate evidence index should fail: {}", out);
    assert!(
        out.contains("duplicate evidence"),
        "should mention duplicate: {}",
        out
    );
}

#[test]
fn test_init_creates_tickets_dir_and_config() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(&repo)
        .output()
        .unwrap();

    let (code, out) = run_tkt(&repo, &["init"]);
    assert_eq!(code, 0, "init should succeed: {}", out);
    assert!(
        repo.join(".tickets").exists(),
        ".tickets/ should be created"
    );
    assert!(
        repo.join(".tickets/config.toml").exists(),
        "config.toml should be created"
    );
}

#[test]
fn test_init_idempotent() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(&repo)
        .output()
        .unwrap();

    // Run init twice
    let (code, _) = run_tkt(&repo, &["init"]);
    assert_eq!(code, 0);
    let (code, out) = run_tkt(&repo, &["init"]);
    assert_eq!(code, 0, "second init should succeed: {}", out);
    assert!(
        out.contains("already exists"),
        "should say already exists: {}",
        out
    );
}

#[test]
fn test_init_write_creates_agents_md_with_markers() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(&repo)
        .output()
        .unwrap();

    // Write some existing content
    std::fs::write(repo.join("AGENTS.md"), "# My Project\n\nCustom content.\n").unwrap();

    let (code, _) = run_tkt(&repo, &["init", "--write"]);
    assert_eq!(code, 0);

    let content = std::fs::read_to_string(repo.join("AGENTS.md")).unwrap();
    assert!(
        content.contains("# My Project"),
        "should preserve existing content: {}",
        content
    );
    assert!(
        content.contains("<!-- tkt:begin -->"),
        "should have begin marker: {}",
        content
    );
    assert!(
        content.contains("<!-- tkt:end -->"),
        "should have end marker: {}",
        content
    );
    assert!(
        content.contains("tkt ready"),
        "should have tkt commands: {}",
        content
    );

    // Run again — should update, not duplicate
    let (code, _) = run_tkt(&repo, &["init", "--write"]);
    assert_eq!(code, 0);
    let content = std::fs::read_to_string(repo.join("AGENTS.md")).unwrap();
    let marker_count = content.matches("<!-- tkt:begin -->").count();
    assert_eq!(marker_count, 1, "should have exactly one marker block");
}

#[test]
fn test_hand_edits_do_not_eject_tickets() {
    // Ticket 132: BOM, comment lines, and space-before-colon must not eject tickets.
    let (_tmp, clone) = setup_repo();

    // BOM-prefixed file (bytes: EF BB BF then "---\n...")
    std::fs::write(
        clone.join(".tickets/02-bom.md"),
        "\u{FEFF}---\nid: \"02\"\ntitle: \"BOM ticket\"\nstatus: open\nblocked_by: []\n---\n\n# Body\n",
    )
    .unwrap();
    // Comment line inside frontmatter
    std::fs::write(
        clone.join(".tickets/03-comment.md"),
        "---\n# a hand note\nid: \"03\"\ntitle: \"Comment ticket\"\nstatus: open\nblocked_by: []\n---\n\n# Body\n",
    )
    .unwrap();
    // Space before colon
    std::fs::write(
        clone.join(".tickets/04-spaced.md"),
        "---\nid : \"04\"\ntitle : \"Spaced ticket\"\nstatus : open\nblocked_by : []\n---\n\n# Body\n",
    )
    .unwrap();

    let (code, out, _err) = run_tkt_env(&clone, &["query"], &[]);
    assert_eq!(code, 0, "query should succeed: {}", out);
    let ids: Vec<&str> = out.trim().lines().collect();
    // seed (01) + 3 hand-edited = 4 tickets, none ejected
    assert_eq!(ids.len(), 4, "all hand-edited tickets should load: {}", out);
    assert!(out.contains("\"id\":\"02\""), "BOM ticket present: {}", out);
    assert!(
        out.contains("\"id\":\"03\""),
        "comment ticket present: {}",
        out
    );
    assert!(
        out.contains("\"id\":\"04\""),
        "space-colon ticket present: {}",
        out
    );
}

#[test]
fn test_broken_file_still_skipped_with_warning() {
    // Ticket 132: genuinely-broken files must still be skipped, with a stderr warning.
    let (_tmp, clone) = setup_repo();

    // No closing fence — unparseable
    std::fs::write(
        clone.join(".tickets/02-broken.md"),
        "---\nid: \"02\"\ntitle: \"Broken\"\nstatus: open\nblocked_by: []\n\n# never closed\n",
    )
    .unwrap();

    let (code, out, err) = run_tkt_env(&clone, &["query"], &[]);
    assert_eq!(code, 0, "query should still succeed on survivors: {}", out);
    // Only the seed ticket loads
    assert_eq!(
        out.trim().lines().count(),
        1,
        "broken file skipped: {}",
        out
    );
    assert!(
        err.contains("skipping") && err.contains("02-broken.md"),
        "stderr should warn about skipped file: {}",
        err
    );
}

#[test]
fn test_validate_flags_hand_flipped_done() {
    // Ticket 154: a done ticket with no ## Resolution (hand-flipped, not closed via
    // tkt close) is flagged by validate — warning by default, error under --strict.
    let (_tmp, clone) = setup_repo();

    // Hand-flipped: status done, no Resolution section
    std::fs::write(
        clone.join(".tickets/02-handflipped.md"),
        "---\nid: \"02\"\ntitle: \"Hand flipped\"\nstatus: done\nblocked_by: []\n---\n\n# Body\n\n- [x] did it\n",
    )
    .unwrap();
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-qm", "add hand-flipped"]);

    // Default: warning, exit 0
    let (code, out) = run_tkt(&clone, &["validate", "--brief"]);
    assert_eq!(code, 0, "warning should not fail default validate: {}", out);
    assert!(
        out.contains("missing-resolution"),
        "should flag missing resolution: {}",
        out
    );

    // Strict: warning promoted to failure, exit 1
    let (code, out) = run_tkt(&clone, &["validate", "--brief", "--strict"]);
    assert_eq!(code, 1, "strict should fail on missing resolution: {}", out);
}

#[test]
fn test_validate_fix_advises_on_hand_flipped_done() {
    // Ticket 154: --fix must NOT fabricate a resolution — it advises the agent to
    // re-close properly.
    let (_tmp, clone) = setup_repo();

    std::fs::write(
        clone.join(".tickets/02-handflipped.md"),
        "---\nid: \"02\"\ntitle: \"Hand flipped\"\nstatus: done\nblocked_by: []\n---\n\n# Body\n\n- [x] did it\n",
    )
    .unwrap();
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-qm", "add hand-flipped"]);

    let (code, out) = run_tkt(&clone, &["validate", "--fix"]);
    assert_eq!(code, 1, "advisory present → exit 1: {}", out);
    assert!(
        out.contains("no resolution recorded"),
        "should advise on missing resolution: {}",
        out
    );
    assert!(
        out.contains("tkt close"),
        "suggestion should name tkt close: {}",
        out
    );

    // The file body must be UNCHANGED — no fabricated resolution
    let content = std::fs::read_to_string(clone.join(".tickets/02-handflipped.md")).unwrap();
    assert!(
        !content.contains("## Resolution"),
        "fix must not fabricate a resolution: {}",
        content
    );
}

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
        "---\nid: \"01\"\ntitle: \"Seed ticket\"\nstatus: done\nblocked_by: []\n---\n\n# Seed\n\n- [x] Done\n",
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
    assert!(out.contains("02"), "should show ticket 02 on frontier: {}", out);
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
    assert!(!out.contains("02"), "blocked ticket should not be on frontier: {}", out);
}

#[test]
fn test_new_creates_and_pushes() {
    let (_tmp, clone) = setup_repo();

    let (code, out) = run_tkt(&clone, &["new", "my-feature", "--title", "My Feature"]);
    assert_eq!(code, 0, "new should succeed: {}", out);
    assert!(out.contains("allocated"), "should say allocated: {}", out);
    assert!(out.contains("02-my-feature.md"), "should create 02: {}", out);

    // Verify file exists
    assert!(clone.join(".tickets/02-my-feature.md").exists());

    // Verify it was pushed (commit is on remote)
    let log = git(&clone, &["log", "--oneline", "origin/main"]);
    assert!(log.contains("new 02 my-feature"), "should be pushed: {}", log);
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
    let (code, out) = run_tkt(&clone, &["close", "02", "--note", "All done"]);
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
    assert!(out.contains("done") || out.contains("not open"), "should explain why: {}", out);
}

#[test]
fn test_close_rejects_already_done() {
    let (_tmp, clone) = setup_repo();

    let (code, out) = run_tkt(&clone, &["close", "01"]);
    assert_ne!(code, 0, "close of done ticket should fail");
    assert!(out.contains("already done"), "should say already done: {}", out);
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
    assert!(out.contains("dangling-blocked-by"), "should report dangling ref: {}", out);
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
    ).unwrap();
    std::fs::create_dir_all(clone.join("docs")).unwrap();
    std::fs::write(
        clone.join("docs/plan.md"),
        "| 01 | Seed | ✅ done |\n| 02 | Drifted | ✅ done |\n",
    ).unwrap();
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-qm", "add drifted"]);

    let (code, out) = run_tkt(&clone, &["sync-plan", "--check", "--brief"]);
    assert_eq!(code, 1, "should detect drift: {}", out);
    assert!(out.contains("plan-status-drift"), "should report drift: {}", out);
}

#[test]
fn test_sync_plan_fix_corrects_status() {
    let (_tmp, clone) = setup_repo();

    // Add a done ticket but plan says open
    std::fs::write(
        clone.join(".tickets/02-fixable.md"),
        "---\nid: \"02\"\ntitle: \"Fixable\"\nstatus: done\nblocked_by: []\n---\n\n# Fixable\n",
    ).unwrap();
    std::fs::create_dir_all(clone.join("docs")).unwrap();
    std::fs::write(
        clone.join("docs/plan.md"),
        "| 01 | Seed | ✅ done |\n| 02 | Fixable | open |\n",
    ).unwrap();
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-qm", "add fixable"]);

    let (code, out) = run_tkt(&clone, &["sync-plan", "--fix", "--brief"]);
    assert_eq!(code, 0, "fix should succeed: {}", out);

    // Verify the plan was updated
    let plan = std::fs::read_to_string(clone.join("docs/plan.md")).unwrap();
    assert!(plan.contains("✅ done"), "plan should now say done for ticket 02: {}", plan);
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
    assert!(content.contains("priority: high"), "should have priority: {}", content);

    // Ready should show it with HIGH flag
    let (_, out) = run_tkt(&clone, &["ready"]);
    assert!(out.contains("[HIGH]") || out.contains("02"), "should show on frontier: {}", out);
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
    ).unwrap();
    std::fs::write(
        clone.join(".tickets/02-beta.md"),
        "---\nid: \"02\"\ntitle: \"Beta\"\nstatus: open\nblocked_by: [\"01\"]\n---\n\n# B\n",
    ).unwrap();
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-qm", "add cycle pair"]);

    let (code, out) = run_tkt(&clone, &["validate", "--brief"]);
    assert_eq!(code, 1, "should fail with cycle: {}", out);
    assert!(out.contains("cycle"), "should mention cycle: {}", out);
    assert!(out.contains("01") && out.contains("02"), "should mention both ids: {}", out);
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
    ).unwrap();
    std::fs::write(
        clone.join(".tickets/02-second.md"),
        "---\nid: \"02\"\ntitle: \"Second\"\nstatus: open\nblocked_by: [\"01\"]\n---\n\n# S\n",
    ).unwrap();
    std::fs::write(
        clone.join(".tickets/03-third.md"),
        "---\nid: \"03\"\ntitle: \"Third\"\nstatus: open\nblocked_by: [\"02\"]\n---\n\n# T\n",
    ).unwrap();
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-qm", "add chain"]);

    let (code, out) = run_tkt(&clone, &["validate", "--brief"]);
    assert_eq!(code, 0, "acyclic chain should pass: {}", out);
    assert!(!out.contains("cycle"), "should not mention cycle: {}", out);
}

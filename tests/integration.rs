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

#[test]
fn test_exit_code_2_on_crash() {
    // Running tkt in a directory with no git repo should exit 2 (operational crash)
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path().to_path_buf();
    std::fs::create_dir_all(dir.join(".tickets")).unwrap();
    std::fs::write(
        dir.join(".tickets/01-test.md"),
        "---\nid: \"01\"\ntitle: \"Test\"\nstatus: open\nblocked_by: []\n---\n\n# Test\n",
    ).unwrap();

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
    assert!(!out.contains("crash"), "domain error should not say crash: {}", out);
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
        assert!(line.starts_with('{') && line.ends_with('}'), "should be JSON object: {}", line);
        assert!(line.contains("\"id\""), "should have id: {}", line);
        assert!(line.contains("\"title\""), "should have title: {}", line);
        assert!(line.contains("\"status\""), "should have status: {}", line);
        assert!(line.contains("\"blocked_by\""), "should have blocked_by: {}", line);
    }

    // Second ticket should have optional fields
    let line2 = lines[1];
    assert!(line2.contains("\"env\""), "should have env: {}", line2);
    assert!(line2.contains("\"priority\""), "should have priority: {}", line2);
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
    Command::new("git").args(["init", "--bare", "-q", "-b", "main"]).arg(&remote).output().unwrap();

    // Clone A
    Command::new("git").args(["clone", "-q"]).arg(&remote).arg(&clone_a).output().unwrap();
    git(&clone_a, &["config", "user.email", "a@test"]);
    git(&clone_a, &["config", "user.name", "a"]);
    git(&clone_a, &["config", "core.autocrlf", "false"]);
    std::fs::create_dir_all(clone_a.join(".tickets")).unwrap();
    std::fs::write(
        clone_a.join(".tickets/01-seed.md"),
        "---\nid: \"01\"\ntitle: \"Seed\"\nstatus: done\nblocked_by: []\n---\n\n# Seed\n",
    ).unwrap();
    git(&clone_a, &["add", "-A"]);
    git(&clone_a, &["commit", "-qm", "seed"]);
    git(&clone_a, &["push", "-q", "origin", "HEAD:main"]);

    // Clone B
    Command::new("git").args(["clone", "-q"]).arg(&remote).arg(&clone_b).output().unwrap();
    git(&clone_b, &["config", "user.email", "b@test"]);
    git(&clone_b, &["config", "user.name", "b"]);
    git(&clone_b, &["config", "core.autocrlf", "false"]);

    // A allocates and pushes first
    let (code_a, out_a) = run_tkt(&clone_a, &["new", "alpha", "--title", "Alpha ticket"]);
    assert_eq!(code_a, 0, "A should succeed: {}", out_a);
    assert!(out_a.contains("02-alpha.md"), "A should get 02: {}", out_a);

    // B allocates — will try 02, get rejected, rebase, get 03
    let (code_b, out_b) = run_tkt(&clone_b, &["new", "beta", "--title", "Beta ticket"]);
    assert_eq!(code_b, 0, "B should succeed after retry: {}", out_b);
    // B should NOT get 02 (that's taken by A)
    assert!(!out_b.contains("02-beta.md"), "B should not collide with A's 02: {}", out_b);
    assert!(out_b.contains("03-beta.md"), "B should get 03: {}", out_b);
}

#[test]
fn test_stale_claim_fails_cleanly() {
    // Clone A closes a ticket, then Clone B (stale) tries to claim it
    let tmp = TempDir::new().unwrap();
    let remote = tmp.path().join("remote.git");
    let clone_a = tmp.path().join("clone-a");
    let clone_b = tmp.path().join("clone-b");

    Command::new("git").args(["init", "--bare", "-q", "-b", "main"]).arg(&remote).output().unwrap();

    // Clone A: create an open ticket
    Command::new("git").args(["clone", "-q"]).arg(&remote).arg(&clone_a).output().unwrap();
    git(&clone_a, &["config", "user.email", "a@test"]);
    git(&clone_a, &["config", "user.name", "a"]);
    git(&clone_a, &["config", "core.autocrlf", "false"]);
    std::fs::create_dir_all(clone_a.join(".tickets")).unwrap();
    std::fs::write(
        clone_a.join(".tickets/01-target.md"),
        "---\nid: \"01\"\ntitle: \"Target\"\nstatus: open\nblocked_by: []\n---\n\n# Target\n",
    ).unwrap();
    git(&clone_a, &["add", "-A"]);
    git(&clone_a, &["commit", "-qm", "seed"]);
    git(&clone_a, &["push", "-q", "origin", "HEAD:main"]);

    // Clone B: from same state
    Command::new("git").args(["clone", "-q"]).arg(&remote).arg(&clone_b).output().unwrap();
    git(&clone_b, &["config", "user.email", "b@test"]);
    git(&clone_b, &["config", "user.name", "b"]);
    git(&clone_b, &["config", "core.autocrlf", "false"]);

    // A closes the ticket and pushes
    let (code, _) = run_tkt(&clone_a, &["close", "01"]);
    assert_eq!(code, 0, "A close should succeed");

    // B tries to claim (should fail because preflight fetch reveals ticket is now done)
    let (code_b, out_b) = run_tkt(&clone_b, &["claim", "01"]);
    assert_eq!(code_b, 1, "B claim should fail: {}", out_b);
    assert!(out_b.contains("not open") || out_b.contains("done"), "should say not open: {}", out_b);
}


#[test]
fn test_no_remote_works_locally() {
    // A repo with no remote should allow all local operations
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path().join("local-only");

    Command::new("git").args(["init", "-q", "-b", "main"]).arg(&dir).output().unwrap();
    git(&dir, &["config", "user.email", "test@test"]);
    git(&dir, &["config", "user.name", "test"]);
    git(&dir, &["config", "core.autocrlf", "false"]);
    std::fs::create_dir_all(dir.join(".tickets")).unwrap();
    std::fs::write(
        dir.join(".tickets/01-local.md"),
        "---\nid: \"01\"\ntitle: \"Local ticket\"\nstatus: open\nblocked_by: []\n---\n\n# Local\n",
    ).unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-qm", "init"]);

    // New should work with "no remote" messaging
    let (code, out) = run_tkt(&dir, &["new", "feature", "--title", "A feature"]);
    assert_eq!(code, 0, "new should succeed locally: {}", out);
    assert!(out.contains("no remote"), "should mention no remote: {}", out);

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
    let (code, out) = run_tkt(&clone, &["new", "special", "--title", "Fix \"ready\" & stuff"]);
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

    Command::new("git").args(["init", "-q", "-b", "main"]).arg(&dir).output().unwrap();
    git(&dir, &["config", "user.email", "test@test"]);
    git(&dir, &["config", "user.name", "test"]);
    git(&dir, &["config", "core.autocrlf", "false"]);
    git(&dir, &["remote", "add", "origin", "https://nonexistent.invalid/repo.git"]);
    std::fs::create_dir_all(dir.join(".tickets")).unwrap();
    std::fs::write(
        dir.join(".tickets/01-test.md"),
        "---\nid: \"01\"\ntitle: \"Test\"\nstatus: open\nblocked_by: []\n---\n\n# Test\n",
    ).unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-qm", "init"]);

    // Edit should fail because push to unreachable remote fails
    let (code, out) = run_tkt(&dir, &["edit", "01", "--priority", "high"]);
    assert_ne!(code, 0, "should fail with unreachable remote: {}", out);
    // Should NOT silently succeed
    assert!(!out.contains("edited 01"), "should not report success: {}", out);
}

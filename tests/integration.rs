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

    let (code, stdout, stderr) = run_tkt_env(&clone, &["ready"], &[("TKT_DEBUG", "1")]);
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
    let clean_env: &[(&str, &str)] = &[("TKT_TELEMETRY", ""), ("DO_NOT_TRACK", "")];

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

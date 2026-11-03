use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

fn repo() -> (TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().to_path_buf();
    git(&root, &["init", "-q"]);
    git(&root, &["config", "user.email", "test@example.invalid"]);
    git(&root, &["config", "user.name", "Test"]);
    std::fs::create_dir(root.join(".tickets")).unwrap();
    std::fs::write(
        root.join(".tickets/config.toml"),
        "[push]\nenabled = false\n[close]\nrequire_resolution = true\nrequire_checked_acs = true\n",
    )
    .unwrap();
    write_open_ticket(&root);
    git(&root, &["add", "."]);
    git(&root, &["commit", "-qm", "seed"]);
    (temp, root)
}

fn write_open_ticket(root: &Path) {
    std::fs::write(
        root.join(".tickets/01-example.md"),
        "---\nid: \"01\"\ntitle: \"Example\"\nstatus: open\nblocked_by: []\n---\n\n# Example\n\n## Acceptance criteria\n\n- [ ] Still pending\n",
    )
    .unwrap();
}

fn git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(["-C", root.to_str().unwrap()])
        .args(args)
        .env("DO_NOT_TRACK", "1")
        .output()
        .unwrap();
    assert!(output.status.success(), "git {args:?}: {output:?}");
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

fn cli(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tkt"))
        .args(args)
        .current_dir(root)
        .env("DO_NOT_TRACK", "1")
        .env("TKT_NO_USER_CONFIG", "1")
        .env("TKT_UPDATE_CHECK", "0")
        .env_remove("TKT_DEBUG")
        .output()
        .unwrap()
}

fn assert_unchanged(root: &Path, head: &str, status: &str, ticket: Option<&[u8]>) {
    assert_eq!(git(root, &["rev-parse", "HEAD"]), head);
    assert_eq!(git(root, &["status", "--porcelain"]), status);
    if let Some(ticket) = ticket {
        assert_eq!(
            std::fs::read(root.join(".tickets/01-example.md")).unwrap(),
            ticket
        );
    }
}

fn assert_done_rejected(output: &Output) {
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = combined.lines().find(|line| line.starts_with('{')).unwrap();
    let envelope: serde_json::Value = serde_json::from_str(json).unwrap();
    assert_eq!(envelope["error"]["kind"], "validation");
    assert!(envelope["error"]["message"]
        .as_str()
        .unwrap()
        .contains("done"));
    assert!(envelope["error"]["hint"]
        .as_str()
        .unwrap()
        .contains("close"));
}

#[test]
fn edit_done_rejection_preserves_ticket_index_and_head_including_dry_run() {
    let (_temp, root) = repo();
    for args in [
        vec!["-o", "json", "edit", "01", "--status", "done"],
        vec!["--dry-run", "-o", "json", "edit", "01", "--status", "done"],
    ] {
        let before = std::fs::read(root.join(".tickets/01-example.md")).unwrap();
        let head = git(&root, &["rev-parse", "HEAD"]);
        let status = git(&root, &["status", "--porcelain"]);
        assert_done_rejected(&cli(&root, &args));
        assert_unchanged(&root, &head, &status, Some(&before));
    }
}

#[test]
fn new_and_batch_done_rejection_preserve_index_and_head_including_dry_run() {
    let (_temp, root) = repo();
    for args in [
        vec!["-o", "json", "batch", "one", "two", "--status", "done"],
        vec![
            "--dry-run",
            "-o",
            "json",
            "batch",
            "one",
            "two",
            "--status",
            "done",
        ],
        vec!["-o", "json", "new", "new-ticket", "--status", "done"],
        vec![
            "--dry-run",
            "-o",
            "json",
            "new",
            "new-ticket",
            "--status",
            "done",
        ],
    ] {
        let head = git(&root, &["rev-parse", "HEAD"]);
        let status = git(&root, &["status", "--porcelain"]);
        assert_done_rejected(&cli(&root, &args));
        assert_unchanged(&root, &head, &status, None);
        assert!(!root.join(".tickets/02-new-ticket.md").exists());
        assert!(!root.join(".tickets/02-one.md").exists());
    }
}

#[test]
fn supported_generic_statuses_still_work_and_close_keeps_its_gates() {
    let (_temp, root) = repo();
    let close = cli(&root, &["-o", "json", "close", "01"]);
    assert_eq!(close.status.code(), Some(1));
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&close.stdout),
        String::from_utf8_lossy(&close.stderr)
    );
    let json = combined.lines().find(|line| line.starts_with('{')).unwrap();
    let error: serde_json::Value = serde_json::from_str(json).unwrap();
    assert_eq!(error["error"]["kind"], "gate_failed");
    assert!(git(&root, &["show", "HEAD:.tickets/01-example.md"]).contains("status: open"));

    let edit = cli(&root, &["edit", "01", "--status", "backlog"]);
    assert!(
        edit.status.success(),
        "{}",
        String::from_utf8_lossy(&edit.stderr)
    );
    let edit = cli(&root, &["edit", "01", "--status", "open"]);
    assert!(
        edit.status.success(),
        "{}",
        String::from_utf8_lossy(&edit.stderr)
    );
    let created = cli(&root, &["new", "backlog-ticket", "--status", "backlog"]);
    assert!(
        created.status.success(),
        "{}",
        String::from_utf8_lossy(&created.stderr)
    );
    let batch = cli(
        &root,
        &["batch", "in-progress-ticket", "--status", "in_progress"],
    );
    assert!(
        batch.status.success(),
        "{}",
        String::from_utf8_lossy(&batch.stderr)
    );

    let closed = cli(
        &root,
        &["close", "01", "--check-all", "--resolution", "verified"],
    );
    assert!(
        closed.status.success(),
        "{}",
        String::from_utf8_lossy(&closed.stderr)
    );
    let content = std::fs::read_to_string(root.join(".tickets/01-example.md")).unwrap();
    assert!(content.contains("status: done"));
    assert!(content.contains("## Resolution"));
}

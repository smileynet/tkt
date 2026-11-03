use std::path::Path;
use std::process::{Command, Output};

fn cli(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tkt"))
        .current_dir(dir)
        .args(args)
        .env("DO_NOT_TRACK", "1")
        .env("TKT_NO_USER_CONFIG", "1")
        .env("TKT_UPDATE_CHECK", "0")
        .env_remove("TKT_DEBUG")
        .output()
        .unwrap()
}

fn git(dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap();
    assert!(output.status.success(), "{args:?}: {output:?}");
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

fn repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    git_init(dir.path());
    let tickets = dir.path().join(".tickets");
    std::fs::create_dir(&tickets).unwrap();
    std::fs::write(tickets.join("config.toml"), "[push]\nenabled = false\n").unwrap();
    std::fs::write(
        tickets.join("01-example.md"),
        "---\nid: \"01\"\ntitle: Example\nstatus: open\nblocked_by: []\ntags: [\"old\", \"shared\"]\nrequires: [\"gpu\"]\nagents: [\"codex\"]\ncustom: preserve-me\n---\n\nBody.\n",
    )
    .unwrap();
    git(dir.path(), &["add", ".tickets"]);
    git(dir.path(), &["commit", "-qm", "seed ticket"]);
    dir
}

fn git_init(dir: &Path) {
    let init = Command::new("git")
        .current_dir(dir)
        .args(["init", "-q"])
        .output()
        .unwrap();
    assert!(init.status.success(), "{init:?}");
    git(dir, &["config", "user.name", "Test User"]);
    git(dir, &["config", "user.email", "test@example.invalid"]);
}

fn run_edit(dir: &Path, args: &[&str]) -> Output {
    let mut command = vec!["edit", "01"];
    command.extend_from_slice(args);
    cli(dir, &command)
}

#[test]
fn replace_tags_accepts_comma_and_repeated_values_and_preserves_other_fields() {
    let dir = repo();
    assert!(cli(dir.path(), &["context", "+contextual"])
        .status
        .success());

    let output = run_edit(
        dir.path(),
        &["--tags", "release,quality", "--tags", "stream"],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let ticket = std::fs::read_to_string(dir.path().join(".tickets/01-example.md")).unwrap();
    assert!(ticket.contains("tags: [\"release\", \"quality\", \"stream\"]"));
    assert!(ticket.contains("requires: [\"gpu\"]"));
    assert!(ticket.contains("agents: [\"codex\"]"));
    assert!(ticket.contains("custom: preserve-me"));
    assert_eq!(
        git(dir.path(), &["status", "--porcelain"]),
        "?? .tickets/.context"
    );
    assert!(git(dir.path(), &["log", "-1", "--format=%s"]).contains("(tags)"));

    let context = cli(dir.path(), &["context"]);
    assert!(String::from_utf8_lossy(&context.stdout).contains("+contextual"));
    assert!(!ticket.contains("contextual"));
}

#[test]
fn add_and_remove_tags_adjust_existing_values() {
    let dir = repo();
    let output = run_edit(
        dir.path(),
        &[
            "--add-tag",
            "new,shared",
            "--add-tag",
            "later",
            "--remove-tag",
            "old,shared",
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let ticket = std::fs::read_to_string(dir.path().join(".tickets/01-example.md")).unwrap();
    assert!(ticket.contains("tags: [\"new\", \"later\"]"), "{ticket}");
}

#[test]
fn empty_tags_value_clears_the_field() {
    let dir = repo();
    let output = run_edit(dir.path(), &["--tags", ""]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let ticket = std::fs::read_to_string(dir.path().join(".tickets/01-example.md")).unwrap();
    assert!(
        !ticket.lines().any(|line| line.starts_with("tags:")),
        "{ticket}"
    );
    assert!(ticket.contains("requires: [\"gpu\"]"));
    assert!(ticket.contains("agents: [\"codex\"]"));
}

#[test]
fn dry_run_tag_changes_leave_ticket_and_git_unchanged() {
    let dir = repo();
    let path = dir.path().join(".tickets/01-example.md");
    let original = std::fs::read(&path).unwrap();
    let head = git(dir.path(), &["rev-parse", "HEAD"]);

    let output = cli(
        dir.path(),
        &[
            "--dry-run",
            "edit",
            "01",
            "--add-tag",
            "extra",
            "--remove-tag",
            "old",
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("tags"));
    assert_eq!(std::fs::read(&path).unwrap(), original);
    assert_eq!(git(dir.path(), &["rev-parse", "HEAD"]), head);
    assert!(git(dir.path(), &["status", "--porcelain"]).is_empty());
}

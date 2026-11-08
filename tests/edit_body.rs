use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use tempfile::TempDir;

fn command(program: &str) -> Command {
    let mut command = Command::new(program);
    command
        .env("DO_NOT_TRACK", "1")
        .env("TKT_NO_USER_CONFIG", "1")
        .env("TKT_UPDATE_CHECK", "0")
        .env("TKT_ASCII", "1")
        .env_remove("TKT_DEBUG")
        .env_remove("TKT_DEBUG_FORMAT")
        .env_remove("TKT_CONTEXT")
        .env_remove("TKT_AGENT");
    command
}

fn git(root: &Path, args: &[&str]) -> String {
    let output = command("git")
        .args(["-C", root.to_str().unwrap()])
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

fn cli(root: &Path, args: &[&str], input: Option<&str>) -> Output {
    let mut child = command(env!("CARGO_BIN_EXE_tkt"))
        .args(args)
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    if let Some(input) = input {
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
    }
    child.wait_with_output().unwrap()
}

fn good(root: &Path, args: &[&str], input: Option<&str>) -> Output {
    let output = cli(root, args, input);
    assert!(
        output.status.success(),
        "{args:?}: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn repo() -> (TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("repo");
    std::fs::create_dir(&root).unwrap();
    git(&root, &["init", "-q", "-b", "main"]);
    git(&root, &["config", "user.email", "test@example.invalid"]);
    git(&root, &["config", "user.name", "Test"]);
    git(&root, &["config", "maintenance.auto", "false"]);
    git(&root, &["config", "gc.auto", "0"]);
    std::fs::create_dir(root.join(".tickets")).unwrap();
    std::fs::write(
        root.join(".tickets/config.toml"),
        "[push]\nenabled = true\n[validate]\nrequire_source_reference = true\n",
    )
    .unwrap();
    std::fs::write(
        root.join(".tickets/01-task.md"),
        "---\nid: \"01\"\ntitle: \"Original title\"\nstatus: open\nblocked_by: []\n---\n\n# Original body\n\nSource: docs/original-spec.md\n\n## Acceptance criteria\n\n- [ ] Original behavior is replaced.\n",
    )
    .unwrap();
    git(&root, &["add", "."]);
    git(&root, &["commit", "-qm", "seed"]);
    (temp, root)
}

fn ticket(root: &Path) -> String {
    std::fs::read_to_string(root.join(".tickets/01-task.md")).unwrap()
}

#[test]
fn edit_body_file_replaces_specification_preserves_notes_and_commits_once() {
    let (_temp, root) = repo();
    good(&root, &["validate", "--strict"], None);
    good(
        &root,
        &[
            "--agent",
            "kimi",
            "note",
            "01",
            "--append",
            "First decision",
        ],
        None,
    );
    good(&root, &["note", "01", "--append", "Second decision"], None);
    let with_notes = ticket(&root);
    let note_start = with_notes.find("## Note — ").unwrap();
    let note_suffix = with_notes[note_start..].to_owned();
    assert_eq!(note_suffix.matches("tkt-note:v1").count(), 2);

    let input = root.join("replacement body.md");
    std::fs::write(
        &input,
        "# Replacement body\n\n## Acceptance criteria\n\n- [ ] Replacement behavior is verified.\n",
    )
    .unwrap();
    let before = git(&root, &["rev-parse", "HEAD"]);
    good(
        &root,
        &["edit", "01", "--body", "@replacement body.md"],
        None,
    );

    let edited = ticket(&root);
    assert!(edited.contains("# Replacement body"), "{edited}");
    assert!(!edited.contains("# Original body"), "{edited}");
    assert!(edited.ends_with(&note_suffix), "{edited}");
    assert_eq!(edited.matches("tkt-note:v1").count(), 2);
    assert_eq!(
        git(&root, &["rev-list", "--count", &format!("{before}..HEAD")]),
        "1"
    );
    assert_eq!(
        git(&root, &["show", "-s", "--format=%s", "HEAD"]),
        "chore(tickets): edit 01 (body)"
    );

    good(&root, &["note", "01", "--append", "Follow-up works"], None);
    let after_note = ticket(&root);
    assert!(after_note.contains(&note_suffix), "{after_note}");
    assert!(after_note.contains("Follow-up works"), "{after_note}");
    assert_eq!(after_note.matches("tkt-note:v1").count(), 3);

    let validation = cli(&root, &["validate", "--strict"], None);
    assert_eq!(validation.status.code(), Some(1), "{validation:?}");
    assert!(
        String::from_utf8_lossy(&validation.stdout).contains("no-source-reference"),
        "{validation:?}"
    );
}

#[test]
fn edit_body_accepts_literal_and_stdin_and_revalidates_the_new_text() {
    let (_temp, root) = repo();
    good(&root, &["note", "01", "--append", "Retain this note"], None);
    let with_note = ticket(&root);
    let suffix = with_note[with_note.find("## Note — ").unwrap()..].to_owned();

    good(
        &root,
        &[
            "edit",
            "01",
            "--body",
            "Source: docs/literal-spec.md\n\n# Literal replacement\n",
        ],
        None,
    );
    let literal = ticket(&root);
    assert!(literal.contains("# Literal replacement"), "{literal}");
    assert!(literal.ends_with(&suffix), "{literal}");
    good(&root, &["validate", "--strict"], None);

    good(
        &root,
        &["edit", "01", "--body", "-"],
        Some("Source: docs/stdin-spec.md\n\n# Stdin replacement\n"),
    );
    let stdin = ticket(&root);
    assert!(stdin.contains("# Stdin replacement"), "{stdin}");
    assert!(!stdin.contains("# Literal replacement"), "{stdin}");
    assert!(stdin.ends_with(&suffix), "{stdin}");
    good(&root, &["validate", "--strict"], None);
}

#[test]
fn edit_body_dry_run_and_no_push_preserve_their_publication_contracts() {
    let (_temp, root) = repo();
    std::fs::write(
        root.join("replacement.md"),
        "Source: docs/replacement.md\n\n# Preview\n",
    )
    .unwrap();
    let before_text = ticket(&root);
    let before_head = git(&root, &["rev-parse", "HEAD"]);
    let preview = good(
        &root,
        &[
            "-q",
            "-o",
            "json",
            "--dry-run",
            "edit",
            "01",
            "--body",
            "@replacement.md",
        ],
        None,
    );
    let json: serde_json::Value = serde_json::from_slice(&preview.stdout).unwrap();
    assert!(json["result"].as_str().unwrap().contains("body"), "{json}");
    assert_eq!(ticket(&root), before_text);
    assert_eq!(git(&root, &["rev-parse", "HEAD"]), before_head);

    let remote = root.parent().unwrap().join("remote.git");
    git(
        root.parent().unwrap(),
        &[
            "init",
            "--bare",
            "-q",
            "-b",
            "main",
            remote.to_str().unwrap(),
        ],
    );
    git(
        &root,
        &["remote", "add", "origin", remote.to_str().unwrap()],
    );
    git(&root, &["push", "-qu", "origin", "main"]);
    let remote_before = git(&root, &["rev-parse", "origin/main"]);
    good(
        &root,
        &[
            "--no-push",
            "edit",
            "01",
            "--body",
            "Source: docs/local-only.md\n\n# Local only\n",
        ],
        None,
    );
    assert!(ticket(&root).contains("# Local only"));
    assert_ne!(git(&root, &["rev-parse", "HEAD"]), before_head);
    assert_eq!(git(&root, &["rev-parse", "origin/main"]), remote_before);
}

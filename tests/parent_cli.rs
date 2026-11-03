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
        "[push]\nenabled = false\n",
    )
    .unwrap();
    std::fs::write(
        root.join(".tickets/01-epic.md"),
        "---\nid: \"01\"\ntitle: \"Epic\"\nstatus: open\nblocked_by: []\n---\n",
    )
    .unwrap();
    std::fs::write(
        root.join(".tickets/02-child.md"),
        "---\nid: \"02\"\ntitle: \"Child\"\nstatus: open\nblocked_by: []\nparent: \"01\"\n---\n",
    )
    .unwrap();
    git(&root, &["add", "."]);
    git(&root, &["commit", "-qm", "seed"]);
    (temp, root)
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

fn assert_failed_without_changes(root: &Path, args: &[&str], ticket: Option<&Path>) {
    let head = git(root, &["rev-parse", "HEAD"]);
    let status = git(root, &["status", "--porcelain"]);
    let bytes = ticket.map(std::fs::read).transpose().unwrap();
    let output = cli(root, args);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(text.to_ascii_lowercase().contains("parent"), "{text}");
    assert_eq!(git(root, &["rev-parse", "HEAD"]), head);
    assert_eq!(git(root, &["status", "--porcelain"]), status);
    if let (Some(ticket), Some(bytes)) = (ticket, bytes) {
        assert_eq!(std::fs::read(ticket).unwrap(), bytes);
    }
}

#[test]
fn parent_can_be_set_cleared_and_filtered_across_cli_surfaces() {
    let (_temp, root) = repo();
    let created = cli(&root, &["new", "third", "--parent", "01"]);
    assert!(created.status.success(), "{created:?}");
    assert!(std::fs::read_to_string(root.join(".tickets/03-third.md"))
        .unwrap()
        .contains("parent: \"01\""));

    let batch = cli(&root, &["batch", "four", "five", "--parent", "01"]);
    assert!(batch.status.success(), "{batch:?}");
    for path in [".tickets/04-four.md", ".tickets/05-five.md"] {
        assert!(std::fs::read_to_string(root.join(path))
            .unwrap()
            .contains("parent: \"01\""));
    }

    let edit = cli(&root, &["edit", "03", "--parent", "02"]);
    assert!(edit.status.success(), "{edit:?}");
    let edit = cli(&root, &["edit", "03", "--parent", ""]);
    assert!(edit.status.success(), "{edit:?}");
    let content = std::fs::read_to_string(root.join(".tickets/03-third.md")).unwrap();
    assert!(!content.contains("parent:"));

    let query = cli(&root, &["query"]);
    assert!(query.status.success(), "{query:?}");
    let rows: Vec<serde_json::Value> = String::from_utf8(query.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(
        rows.iter().find(|row| row["id"] == "02").unwrap()["parent"],
        "01"
    );
    let query = cli(&root, &["query", "--parent", "01"]);
    assert!(query.status.success(), "{query:?}");
    let rows: Vec<serde_json::Value> = String::from_utf8(query.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert!(rows.iter().all(|row| row["parent"] == "01"));
    assert_eq!(rows.len(), 3);
}

#[test]
fn nonexistent_self_and_cyclic_parents_are_rejected_before_writes() {
    let (_temp, root) = repo();
    assert_failed_without_changes(&root, &["new", "missing", "--parent", "99"], None);
    assert_failed_without_changes(
        &root,
        &["--dry-run", "new", "missing-dry", "--parent", "99"],
        None,
    );
    assert_failed_without_changes(&root, &["new", "self", "--parent", "03"], None);
    assert_failed_without_changes(&root, &["batch", "missing", "--parent", "99"], None);
    assert_failed_without_changes(
        &root,
        &["--dry-run", "batch", "missing-dry", "--parent", "99"],
        None,
    );
    assert_failed_without_changes(
        &root,
        &["batch", "self-one", "self-two", "--parent", "03"],
        None,
    );
    let child = root.join(".tickets/02-child.md");
    assert_failed_without_changes(&root, &["edit", "02", "--parent", "99"], Some(&child));
    assert_failed_without_changes(
        &root,
        &["--dry-run", "edit", "02", "--parent", "99"],
        Some(&child),
    );
    assert_failed_without_changes(&root, &["edit", "02", "--parent", "02"], Some(&child));
    let epic = root.join(".tickets/01-epic.md");
    assert_failed_without_changes(&root, &["edit", "01", "--parent", "02"], Some(&epic));
}

#[test]
fn parent_must_resolve_to_a_parsed_ticket_not_just_a_filename_id() {
    let (_temp, root) = repo();
    std::fs::write(
        root.join(".tickets/98-mismatched.md"),
        "---\nid: \"99\"\ntitle: \"Mismatched filename\"\nstatus: open\nblocked_by: []\n---\n",
    )
    .unwrap();
    std::fs::write(
        root.join(".tickets/97-unparseable.md"),
        "---\nid: \"97\"\ntitle: \"Broken\"\nstatus: invalid\nblocked_by: []\n---\n",
    )
    .unwrap();
    git(&root, &["add", "."]);
    git(&root, &["commit", "-qm", "add malformed parent candidates"]);

    assert_failed_without_changes(&root, &["new", "wrong-id", "--parent", "98"], None);
    assert_failed_without_changes(&root, &["new", "broken-id", "--parent", "97"], None);
    assert_failed_without_changes(&root, &["batch", "wrong-id", "--parent", "98"], None);
    assert_failed_without_changes(&root, &["batch", "broken-id", "--parent", "97"], None);
}

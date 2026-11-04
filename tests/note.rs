use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

fn command(program: &str) -> Command {
    let mut c = Command::new(program);
    c.env("DO_NOT_TRACK", "1")
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
        .env_remove("TKT_AGENT")
        .env_remove("TKT_CONTEXT");
    c
}
fn git(root: &Path, args: &[&str]) -> String {
    let out = command("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .unwrap();
    assert!(out.status.success(), "{args:?}: {out:?}");
    String::from_utf8(out.stdout).unwrap().trim().into()
}
fn cli(root: &Path, args: &[&str]) -> Command {
    let mut c = command(env!("CARGO_BIN_EXE_tkt"));
    c.args(args)
        .current_dir(root)
        .env("XDG_CONFIG_HOME", root.join("private-config"))
        .env("XDG_STATE_HOME", root.join("private-state"));
    c
}
fn good(root: &Path, args: &[&str]) -> Output {
    let out = cli(root, args).output().unwrap();
    assert!(out.status.success(), "{args:?}: {out:?}");
    out
}
fn fixture(status: &str, push: bool, hooks: bool) -> (tempfile::TempDir, PathBuf, Vec<u8>) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("repo");
    std::fs::create_dir(&root).unwrap();
    git(&root, &["init", "-q", "-b", "main"]);
    git(&root, &["config", "user.name", "Note fixture"]);
    git(&root, &["config", "user.email", "fixture@example.invalid"]);
    std::fs::create_dir(root.join(".tickets")).unwrap();
    std::fs::create_dir(root.join("docs")).unwrap();
    std::fs::write(
        root.join(".tickets/config.toml"),
        format!(
            "[push]\nenabled = {push}\n[commit]\nhooks = \"{}\"\n",
            if hooks { "run" } else { "skip" }
        ),
    )
    .unwrap();
    let mut ticket=format!("\u{feff}---\r\nid: \"01\"\r\ntitle: \"Preserve Ω\"\r\nstatus: {status}\r\nblocked_by: []\r\nassignee: \"prior-owner\"\r\ncustom_extension: \"raw: value\"\r\n---\r\n\r\n# Specification\r\n\r\nPreserve this text and trailing spaces.  \r\n\r\n## Acceptance criteria\r\n\r\n- [x] Existing behavior verified.\r\n");
    if status == "done" {
        ticket.push_str("\r\n## Resolution (2026-10-01)\r\n\r\nShipped the original behavior.\r\n\r\n### Verification\r\n\r\nOriginal evidence remains intact.\r\n");
    }
    if status == "cancelled" {
        ticket.push_str("\r\n## Cancellation\r\n\r\n- Reason: obsolete\r\n- Note: Explicitly retired.\r\n- Date: 2026-10-01\r\n");
    }
    std::fs::write(root.join(".tickets/01-seed.md"), ticket.as_bytes()).unwrap();
    std::fs::write(root.join("docs/plan.md"), "Owner plan remains unchanged.\n").unwrap();
    std::fs::write(root.join("README.md"), "Owner source\n").unwrap();
    git(&root, &["add", "."]);
    git(&root, &["commit", "-qm", "seed"]);
    if push {
        let remote = temp.path().join("remote.git");
        let out = command("git")
            .args(["init", "--bare", "-q"])
            .arg(&remote)
            .output()
            .unwrap();
        assert!(out.status.success());
        git(
            &root,
            &["remote", "add", "origin", remote.to_str().unwrap()],
        );
        git(&root, &["push", "-qu", "origin", "main"]);
    }
    (temp, root, ticket.into_bytes())
}
fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn walk(base: &Path, root: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for e in std::fs::read_dir(root).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                walk(base, &p, out)
            } else {
                out.insert(
                    p.strip_prefix(base).unwrap().to_path_buf(),
                    std::fs::read(p).unwrap(),
                );
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(root, root, &mut out);
    out
}
fn hook(path: &Path, text: &str) {
    std::fs::write(path, text).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
}

#[test]
fn notes_preserve_original_bytes_lifecycle_evidence_and_unrelated_staged_work() {
    for status in ["open", "in_progress", "done", "backlog", "cancelled"] {
        let (_temp, root, original) = fixture(status, false, false);
        std::fs::write(root.join("README.md"), "Unrelated staged edit\n").unwrap();
        git(&root, &["add", "README.md"]);
        let out = cli(
            &root,
            &[
                "--agent",
                "codex-noter",
                "-q",
                "-o",
                "json",
                "note",
                "01",
                "--append",
                "Decision: keep the current interface. Ω",
            ],
        )
        .env("EDITOR", "definitely-not-an-editor")
        .output()
        .unwrap();
        assert!(out.status.success(), "{out:?}");
        let result: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(result["actor"], "codex-noter");
        assert_eq!(result["dry_run"], false);
        let after = std::fs::read(root.join(".tickets/01-seed.md")).unwrap();
        assert!(after.starts_with(&original));
        let text = String::from_utf8(after).unwrap();
        assert!(text.contains("By: codex-noter"));
        assert!(text.contains("Decision: keep the current interface. Ω"));
        let query = good(&root, &["query", "--all-context"]);
        let row: serde_json::Value = serde_json::from_slice(&query.stdout).unwrap();
        assert_eq!(row["status"], status);
        assert_eq!(row["assignee"], "prior-owner");
        assert_eq!(
            git(&root, &["diff", "--cached", "--name-only"]),
            "README.md"
        );
        assert_eq!(
            git(
                &root,
                &["diff-tree", "--no-commit-id", "--name-only", "-r", "HEAD"]
            ),
            ".tickets/01-seed.md"
        );
        assert_eq!(
            std::fs::read_to_string(root.join("docs/plan.md")).unwrap(),
            "Owner plan remains unchanged.\n"
        );
        assert!(!root.join(".git/tkt-note.lock").exists());
    }
}

#[test]
fn file_stdin_preview_missing_invalid_and_reserved_inputs_have_explicit_effects() {
    let (_temp, root, original) = fixture("open", false, false);
    let input = root.join("note.txt");
    std::fs::write(&input, "File decision\nSecond line\n").unwrap();
    let before = snapshot(&root);
    good(&root, &["--dry-run", "note", "01", "--append", "@note.txt"]);
    assert_eq!(snapshot(&root), before);
    for args in [
        vec!["note", "999", "--append", "Missing target"],
        vec!["note", "01", "--append", "  \n"],
        vec![
            "note",
            "01",
            "--append",
            "```md\n## Resolution\nExample\n```",
        ],
        vec!["note", "01", "--append", "Quoted `## Resolution` marker"],
        vec!["note", "01", "--append", "@absent.txt"],
    ] {
        let before = snapshot(&root);
        let out = cli(&root, &args).output().unwrap();
        assert!(!out.status.success(), "{args:?}: {out:?}");
        assert_eq!(snapshot(&root), before);
    }
    std::fs::write(&input, vec![b'x'; 65537]).unwrap();
    let before = snapshot(&root);
    let out = cli(&root, &["note", "01", "--append", "@note.txt"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    assert_eq!(snapshot(&root), before);
    std::fs::write(&input, [0xff, 0xfe]).unwrap();
    let before = snapshot(&root);
    let out = cli(&root, &["note", "01", "--append", "@note.txt"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    assert_eq!(snapshot(&root), before);
    std::fs::write(&input, "File decision\nSecond line\n").unwrap();
    good(&root, &["note", "01", "--append", "@note.txt"]);
    let mut child = cli(&root, &["note", "01", "--append", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all("Piped decision Ω\n".as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "{out:?}");
    let bytes = std::fs::read(root.join(".tickets/01-seed.md")).unwrap();
    assert!(bytes.starts_with(&original));
    let text = String::from_utf8(bytes).unwrap();
    assert!(text.contains("File decision\nSecond line\n"));
    assert!(text.contains("Piped decision Ω\n"));
    assert!(text.contains("By: anonymous"));
    good(
        &root,
        &["close", "01", "--resolution", "Actual close evidence"],
    );
    let text = std::fs::read_to_string(root.join(".tickets/01-seed.md")).unwrap();
    assert!(text.contains("Actual close evidence"));
}

#[test]
fn failed_commit_preserves_appended_note_for_explicit_recovery() {
    let (_temp, root, original) = fixture("open", false, true);
    let head = git(&root, &["rev-parse", "HEAD"]);
    std::fs::write(root.join(".git/hooks/pre-commit"), "#!/bin/sh\nexit 1\n").unwrap();
    let out = cli(
        &root,
        &["note", "01", "--append", "Retained after hook failure"],
    )
    .output()
    .unwrap();
    assert!(!out.status.success(), "{out:?}");
    assert_eq!(git(&root, &["rev-parse", "HEAD"]), head);
    let bytes = std::fs::read(root.join(".tickets/01-seed.md")).unwrap();
    assert!(bytes.starts_with(&original));
    assert!(String::from_utf8_lossy(&bytes).contains("Retained after hook failure"));
    assert!(!git(&root, &["diff", "--cached", "--name-only"]).is_empty());
    assert!(!root.join(".git/tkt-note.lock").exists());
    let before = snapshot(&root);
    let out = cli(&root, &["note", "01", "--append", "Do not append again"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    assert_eq!(snapshot(&root), before);
}

#[test]
fn ignored_untracked_ticket_is_refused_without_writing() {
    let (_temp, root, original) = fixture("open", false, false);
    git(&root, &["rm", "--cached", ".tickets/01-seed.md"]);
    std::fs::write(root.join(".gitignore"), ".tickets/01-seed.md\n").unwrap();
    git(&root, &["add", ".gitignore"]);
    git(&root, &["commit", "-qm", "ignore ticket fixture"]);
    assert!(git(&root, &["status", "--porcelain"]).is_empty());
    let before = snapshot(&root);
    let out = cli(&root, &["note", "01", "--append", "Must remain absent"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    assert!(String::from_utf8_lossy(&out.stderr).contains("must already be tracked"));
    assert_eq!(
        std::fs::read(root.join(".tickets/01-seed.md")).unwrap(),
        original
    );
    assert_eq!(snapshot(&root), before);
}

#[test]
fn dirty_target_and_foreign_or_interrupted_marker_are_preserved() {
    let (_temp, root, _) = fixture("open", false, false);
    let target = root.join(".tickets/01-seed.md");
    let original = std::fs::read(&target).unwrap();
    std::fs::write(
        &target,
        [original.clone(), b"\nUncommitted specification\n".to_vec()].concat(),
    )
    .unwrap();
    let before = snapshot(&root);
    let out = cli(
        &root,
        &[
            "note",
            "01",
            "--append",
            "Must not commit unrelated specification",
        ],
    )
    .output()
    .unwrap();
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    assert_eq!(snapshot(&root), before);
    std::fs::write(target, original).unwrap();
    std::fs::write(
        root.join(".git/tkt-note.lock"),
        "Foreign or interrupted marker\n",
    )
    .unwrap();
    let before = snapshot(&root);
    let out = cli(&root, &["note", "01", "--append", "Blocked note"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    assert_eq!(snapshot(&root), before);
}

#[test]
fn cooperating_note_processes_serialize_and_retry_without_losing_text() {
    let (_temp, root, _) = fixture("open", false, true);
    hook(&root.join(".git/hooks/pre-commit"),"#!/bin/sh\nprintf ready > .git/hook-ready\ni=0\nwhile [ ! -e .git/hook-release ] && [ $i -lt 250 ]; do sleep 0.02; i=$((i+1)); done\n[ -e .git/hook-release ]\n");
    let mut first = cli(
        &root,
        &["note", "01", "--append", "First concurrent decision"],
    )
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(6);
    while !root.join(".git/hook-ready").exists() && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    if !root.join(".git/hook-ready").exists() {
        let _ = first.kill();
        let out = first.wait_with_output().unwrap();
        panic!("first note did not reach the holding hook: {out:?}")
    }
    let second = cli(
        &root,
        &["note", "01", "--append", "Second concurrent decision"],
    )
    .output()
    .unwrap();
    std::fs::write(root.join(".git/hook-release"), "release").unwrap();
    let out = first.wait_with_output().unwrap();
    assert!(out.status.success(), "{out:?}");
    assert_eq!(second.status.code(), Some(1), "{second:?}");
    std::fs::remove_file(root.join(".git/hooks/pre-commit")).unwrap();
    good(
        &root,
        &["note", "01", "--append", "Second concurrent decision"],
    );
    let text = std::fs::read_to_string(root.join(".tickets/01-seed.md")).unwrap();
    assert_eq!(text.matches("First concurrent decision").count(), 1);
    assert_eq!(text.matches("Second concurrent decision").count(), 1);
    assert!(!root.join(".git/tkt-note.lock").exists());
}

#[test]
fn linked_worktree_lock_path_and_publication_retry_preserve_single_append() {
    let (temp, root, _) = fixture("open", false, false);
    let worktree = temp.path().join("worktree");
    git(
        &root,
        &[
            "worktree",
            "add",
            "-qb",
            "note-worktree",
            worktree.to_str().unwrap(),
        ],
    );
    assert!(worktree.join(".git").is_file());
    good(
        &worktree,
        &["note", "01", "--append", "Linked worktree decision"],
    );
    let lock = PathBuf::from(git(
        &worktree,
        &["rev-parse", "--git-path", "tkt-note.lock"],
    ));
    assert!(!lock.exists());
    assert!(!std::fs::read_to_string(root.join(".tickets/01-seed.md"))
        .unwrap()
        .contains("Linked worktree decision"));
    let (temp, root, _) = fixture("open", true, false);
    let remote = temp.path().join("remote.git");
    let previous = git(&root, &["rev-parse", "HEAD"]);
    hook(&remote.join("hooks/pre-receive"), "#!/bin/sh\nexit 1\n");
    let out = cli(
        &root,
        &["note", "01", "--append", "Durable publication decision"],
    )
    .output()
    .unwrap();
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    let local = git(&root, &["rev-parse", "HEAD"]);
    assert_ne!(local, previous);
    assert_eq!(git(&remote, &["rev-parse", "refs/heads/main"]), previous);
    std::fs::remove_file(remote.join("hooks/pre-receive")).unwrap();
    git(&root, &["push", "-q", "origin", "main"]);
    assert_eq!(git(&remote, &["rev-parse", "refs/heads/main"]), local);
    assert_eq!(
        std::fs::read_to_string(root.join(".tickets/01-seed.md"))
            .unwrap()
            .matches("Durable publication decision")
            .count(),
        1
    );
    git(
        &root,
        &[
            "remote",
            "set-url",
            "origin",
            temp.path().join("unreachable.git").to_str().unwrap(),
        ],
    );
    good(
        &root,
        &[
            "--no-push",
            "note",
            "01",
            "--append",
            "Explicit local-only decision",
        ],
    );
}

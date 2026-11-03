use std::path::Path;
use std::process::{Command, Output};
use tempfile::TempDir;

fn git(root: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(args)
        .current_dir(root)
        .env("DO_NOT_TRACK", "1")
        .output()
        .unwrap();
    assert!(out.status.success(), "{args:?}: {out:?}");
    String::from_utf8(out.stdout).unwrap().trim().into()
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
fn repo() -> TempDir {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    git(root, &["init", "-q"]);
    git(root, &["config", "user.name", "Test"]);
    git(root, &["config", "user.email", "test@example.invalid"]);
    std::fs::create_dir(root.join(".tickets")).unwrap();
    std::fs::write(root.join(".tickets/config.toml"),"[push]\nenabled = false\n[close]\nrequire_resolution = true\nrequire_checked_acs = true\nrequire_validation_criteria = true\nrequire_validation_evidence = true\n").unwrap();
    for (id, text) in [("01", "Implement feature"), ("02", "Other work")] {
        std::fs::write(root.join(format!(".tickets/{id}-fixture.md")),format!("---\nid: \"{id}\"\ntitle: \"Fixture\"\nstatus: open\nblocked_by: []\nvalidation_criteria:\n  - \"Retained proof\"\ncustom: preserve\n---\n\n## What to build\n\nDeliver specified behavior.\n\n## Acceptance criteria\n\n- [ ] {text}\n\n## Notes\n\nKeep history.\n")).unwrap();
    }
    git(root, &["add", "."]);
    git(root, &["commit", "-qm", "seed"]);
    temp
}
fn read(root: &Path, id: &str) -> String {
    std::fs::read_to_string(root.join(format!(".tickets/{id}-fixture.md"))).unwrap()
}
fn assert_ok(out: Output) -> String {
    assert!(out.status.success(), "{out:?}");
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn linked_transfer_is_distinct_from_done_and_retry_is_a_noop() {
    let temp = repo();
    let root = temp.path();
    std::fs::write(root.join("unrelated.txt"), "keep staged").unwrap();
    git(root, &["add", "unrelated.txt"]);
    assert_ok(cli(
        root,
        &[
            "move-ac", "01", "--ac", "1", "--to", "02", "-q", "-o", "json",
        ],
    ));
    let source = read(root, "01");
    let target = read(root, "02");
    assert!(source.contains("- [>] Implement feature <!-- tkt-moved:02 -->"));
    assert!(target.contains("- [ ] Implement feature <!-- tkt-from:01:1 -->"));
    assert!(source.contains("status: open"));
    assert!(source.contains("custom: preserve"));
    assert!(source.contains("- \"Retained proof\""));
    assert!(target.contains("- \"Retained proof\""));
    assert_eq!(
        git(root, &["diff", "--cached", "--name-only"]),
        "unrelated.txt"
    );
    let head = git(root, &["rev-parse", "HEAD"]);
    assert_ok(cli(root, &["move-ac", "01", "--ac", "1", "--to", "02"]));
    assert_eq!(git(root, &["rev-parse", "HEAD"]), head);
    assert_eq!(read(root, "01"), source);
    assert_eq!(read(root, "02"), target);
    assert_ok(cli(root, &["validate", "--strict", "--brief"]));
    assert_ok(cli(
        root,
        &[
            "close",
            "01",
            "--resolution",
            "Transferred feature, no implementation claimed",
            "--evidence",
            "Body work transferred; retained proof independently addressed",
        ],
    ));
    assert!(read(root, "01").contains("- [>]"));
    assert!(!read(root, "01").contains("- [x] Implement feature"));
    assert_ok(cli(root, &["audit", "--strict", "--brief"]));
    assert!(read(root, "02").contains("- [ ] Implement feature"));
}

#[test]
fn interrupted_destination_first_write_recovers_without_losing_or_duplicating_work() {
    let temp = repo();
    let root = temp.path();
    // This is the exact on-disk intermediate state after destination atomic rename
    // and before source rename (process interruption or source write failure).
    let target = read(root, "02").replace(
        "## Notes",
        "- [ ] Implement feature <!-- tkt-from:01:1 -->\n\n## Notes",
    );
    std::fs::write(root.join(".tickets/02-fixture.md"), target).unwrap();
    assert_eq!(
        cli(root, &["validate", "--strict", "--brief"])
            .status
            .code(),
        Some(1)
    );
    assert_ok(cli(root, &["move-ac", "01", "--ac", "1", "--to", "02"]));
    assert_eq!(read(root, "02").matches("tkt-from:01:1").count(), 1);
    assert!(read(root, "01").contains("- [>] Implement feature"));
    assert_ok(cli(root, &["validate", "--strict", "--brief"]));
    // Both writes landed, but process stopped before the commit.
    git(root, &["reset", "--mixed", "HEAD~1"]);
    assert_ok(cli(root, &["move-ac", "01", "--ac", "1", "--to", "02"]));
    assert_eq!(read(root, "02").matches("tkt-from:01:1").count(), 1);
    assert!(git(root, &["status", "--porcelain"]).is_empty());
}

#[test]
fn dry_run_and_invalid_transfers_do_not_write_or_commit() {
    let temp = repo();
    let root = temp.path();
    let head = git(root, &["rev-parse", "HEAD"]);
    let source = read(root, "01");
    let target = read(root, "02");
    assert_ok(cli(
        root,
        &["--dry-run", "move-ac", "01", "--ac", "1", "--to", "02"],
    ));
    for args in [
        vec!["move-ac", "01", "--ac", "0", "--to", "02"],
        vec!["move-ac", "01", "--ac", "2", "--to", "02"],
        vec!["move-ac", "01", "--ac", "1", "--to", "01"],
        vec!["move-ac", "01", "--ac", "1", "--to", "99"],
    ] {
        assert_eq!(cli(root, &args).status.code(), Some(1), "{args:?}");
    }
    assert_eq!(read(root, "01"), source);
    assert_eq!(read(root, "02"), target);
    assert_eq!(git(root, &["rev-parse", "HEAD"]), head);
    std::fs::write(
        root.join(".tickets/01-fixture.md"),
        source.replace("- [ ] Implement", "- [x] Implement"),
    )
    .unwrap();
    assert_eq!(
        cli(root, &["move-ac", "01", "--ac", "1", "--to", "02"])
            .status
            .code(),
        Some(1)
    );
    std::fs::write(
        root.join(".tickets/01-fixture.md"),
        source.replace("Implement feature", ""),
    )
    .unwrap();
    assert_eq!(
        cli(root, &["move-ac", "01", "--ac", "1", "--to", "02"])
            .status
            .code(),
        Some(1)
    );
    assert_eq!(read(root, "02"), target);
}

#[test]
fn broken_or_duplicate_transfer_links_fail_validate_audit_and_close() {
    let temp = repo();
    let root = temp.path();
    assert_ok(cli(root, &["move-ac", "01", "--ac", "1", "--to", "02"]));
    let target = read(root, "02");
    std::fs::write(
        root.join(".tickets/02-fixture.md"),
        target.replace(
            "Implement feature <!-- tkt-from:01:1 -->",
            "Changed <!-- tkt-from:01:1 -->",
        ),
    )
    .unwrap();
    for args in [
        vec!["validate", "--brief"],
        vec!["audit", "--brief"],
        vec!["close", "01", "--force"],
    ] {
        let out = cli(root, &args);
        assert_eq!(out.status.code(), Some(1), "{args:?}: {out:?}");
    }
    assert_eq!(
        cli(root, &["move-ac", "01", "--ac", "1", "--to", "02"])
            .status
            .code(),
        Some(1)
    );
    std::fs::write(
        root.join(".tickets/02-fixture.md"),
        target.replace(
            "## Notes",
            "- [ ] Implement feature <!-- tkt-from:01:1 -->\n\n## Notes",
        ),
    )
    .unwrap();
    assert_eq!(cli(root, &["validate", "--brief"]).status.code(), Some(1));
    std::fs::remove_file(root.join(".tickets/02-fixture.md")).unwrap();
    assert_eq!(
        cli(
            root,
            &[
                "close",
                "01",
                "--resolution",
                "Transferred",
                "--evidence",
                "Retained proof"
            ]
        )
        .status
        .code(),
        Some(1)
    );
}

#[test]
fn crlf_criteria_transfer_and_multiline_rejection_preserve_content() {
    let temp = repo();
    let root = temp.path();
    for id in ["01", "02"] {
        std::fs::write(
            root.join(format!(".tickets/{id}-fixture.md")),
            read(root, id).replace('\n', "\r\n"),
        )
        .unwrap();
    }
    assert_ok(cli(root, &["move-ac", "01", "--ac", "1", "--to", "02"]));
    assert!(read(root, "01").contains("- [>] Implement feature <!-- tkt-moved:02 -->\r\n"));
    assert_ok(cli(root, &["validate", "--strict", "--brief"]));
    let temp = repo();
    let root = temp.path();
    std::fs::write(
        root.join(".tickets/01-fixture.md"),
        read(root, "01").replace(
            "Implement feature",
            "Implement feature\n  and retain this continuation",
        ),
    )
    .unwrap();
    let source = read(root, "01");
    let target = read(root, "02");
    assert_eq!(
        cli(root, &["move-ac", "01", "--ac", "1", "--to", "02"])
            .status
            .code(),
        Some(1)
    );
    assert_eq!(read(root, "01"), source);
    assert_eq!(read(root, "02"), target);
}

#[test]
fn unchanged_upstream_pair_can_publish_and_no_push_remains_local() {
    let temp = repo();
    let root = temp.path();
    let remote = tempfile::tempdir().unwrap();
    git(remote.path(), &["init", "--bare", "-q"]);
    git(
        root,
        &["remote", "add", "origin", remote.path().to_str().unwrap()],
    );
    git(root, &["push", "-u", "origin", "HEAD"]);
    std::fs::write(
        root.join(".tickets/config.toml"),
        "[push]\nenabled = true\n",
    )
    .unwrap();
    git(root, &["add", "."]);
    git(root, &["commit", "-qm", "enable publication"]);
    git(root, &["push"]);
    assert_ok(cli(root, &["move-ac", "01", "--ac", "1", "--to", "02"]));
    assert_eq!(
        git(root, &["rev-parse", "HEAD"]),
        git(root, &["rev-parse", "@{upstream}"])
    );
    std::fs::write(
        root.join(".tickets/01-fixture.md"),
        read(root, "01").replace("## Notes", "- [ ] Another feature\n\n## Notes"),
    )
    .unwrap();
    git(root, &["add", "."]);
    git(root, &["commit", "-qm", "second criterion"]);
    let upstream = git(root, &["rev-parse", "@{upstream}"]);
    assert_ok(cli(
        root,
        &["--no-push", "move-ac", "01", "--ac", "2", "--to", "02"],
    ));
    assert_eq!(git(root, &["rev-parse", "@{upstream}"]), upstream);
    assert_ne!(git(root, &["rev-parse", "HEAD"]), upstream);
}

#[test]
fn fenced_checkbox_examples_and_headings_are_never_transfer_candidates() {
    let temp = repo();
    let root = temp.path();
    let example="````markdown\n## Acceptance criteria\n- [>] Fake <!-- tkt-moved:99 -->\n```\n- [ ] Fake <!-- tkt-from:99:1 -->\n````\n\n";
    let source=read(root,"01").replace("## What to build",&(example.to_owned()+"## What to build"))
        .replace("- [ ] Implement feature",&(example.to_owned()+"- [ ] Implement feature\n\n~~~markdown\n## Fake heading\n- [ ] Another fake\n~~~"));
    std::fs::write(root.join(".tickets/01-fixture.md"), source).unwrap();
    assert_ok(cli(root, &["move-ac", "01", "--ac", "1", "--to", "02"]));
    assert!(read(root, "01").contains("- [>] Implement feature <!-- tkt-moved:02 -->"));
    assert_eq!(read(root, "01").matches(example).count(), 2);
    assert_ok(cli(root, &["validate", "--strict", "--brief"]));
    assert_eq!(
        cli(root, &["move-ac", "01", "--ac", "2", "--to", "02"])
            .status
            .code(),
        Some(1)
    );
}

#[test]
fn source_transfer_markers_cannot_be_relabelled_as_checked_or_unchecked() {
    let temp = repo();
    let root = temp.path();
    assert_ok(cli(root, &["move-ac", "01", "--ac", "1", "--to", "02"]));
    let source = read(root, "01");
    // Even removing the receiver cannot disguise transferred work as completed.
    std::fs::remove_file(root.join(".tickets/02-fixture.md")).unwrap();
    for state in ["x", " "] {
        std::fs::write(
            root.join(".tickets/01-fixture.md"),
            source.replace("[>]", &format!("[{state}]")),
        )
        .unwrap();
        for args in [
            vec!["validate", "--brief"],
            vec!["audit", "--brief"],
            vec!["close", "01", "--force"],
        ] {
            assert_eq!(
                cli(root, &args).status.code(),
                Some(1),
                "{args:?}, state {state}"
            );
        }
    }
}

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

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
fn cli(root: &Path, args: &[&str]) -> Output {
    command(env!("CARGO_BIN_EXE_tkt"))
        .args(args)
        .current_dir(root)
        .env("XDG_CONFIG_HOME", root.join("private-config"))
        .env("XDG_STATE_HOME", root.join("private-state"))
        .output()
        .unwrap()
}
fn good(root: &Path, args: &[&str]) -> Output {
    let out = cli(root, args);
    assert!(out.status.success(), "{args:?}: {out:?}");
    out
}
fn fixture(push: bool, hooks: bool) -> (tempfile::TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("repo");
    std::fs::create_dir(&root).unwrap();
    git(&root, &["init", "-q", "-b", "main"]);
    git(&root, &["config", "maintenance.auto", "false"]);
    git(&root, &["config", "gc.auto", "0"]);
    git(&root, &["config", "user.name", "Undo fixture"]);
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
    std::fs::write(root.join(".tickets/01-seed.md"),"---\nid: \"01\"\ntitle: \"Seed\"\nstatus: open\nblocked_by: []\n---\n\nKeep this specification.\n\n## Acceptance criteria\n\n- [ ] Verify the behavior.\n").unwrap();
    std::fs::write(
        root.join("docs/plan.md"),
        "Owner narrative.\n\n| ID | Work | Status |\n|---|---|---|\n| 01 | Seed | open |\n",
    )
    .unwrap();
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
    (temp, root)
}
fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn walk(base: &Path, root: &Path, rows: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in std::fs::read_dir(root).unwrap() {
            let p = entry.unwrap().path();
            if p.is_dir() {
                walk(base, &p, rows)
            } else {
                rows.insert(
                    p.strip_prefix(base).unwrap().to_path_buf(),
                    std::fs::read(p).unwrap(),
                );
            }
        }
    }
    let mut rows = BTreeMap::new();
    walk(root, root, &mut rows);
    rows
}
fn hook(path: &Path, body: &str) {
    std::fs::write(path, body).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
}
fn amend_message(root: &Path, message: &str) {
    let p = root.join("message.txt");
    std::fs::write(&p, message).unwrap();
    git(
        root,
        &["commit", "--amend", "-q", "-F", p.to_str().unwrap()],
    );
    std::fs::remove_file(p).unwrap();
}

#[test]
fn new_claim_and_close_revert_to_exact_parent_tree_with_read_only_previews() {
    for args in [
        vec!["new", "added", "--title", "Added"],
        vec!["--agent", "codex", "claim", "01"],
        vec![
            "close",
            "01",
            "--check-all",
            "--resolution",
            "Verified fixture behavior",
        ],
    ] {
        let (_temp, root) = fixture(false, false);
        let before = git(&root, &["rev-parse", "HEAD^{tree}"]);
        let start = std::time::Instant::now();
        good(&root, &args);
        eprintln!("generated {args:?} in {:?}", start.elapsed());
        let original = git(&root, &["rev-parse", "HEAD"]);
        assert!(git(&root, &["show", "-s", "--format=%B", "HEAD"]).contains("Tkt-Undo: "));
        let saved = snapshot(&root);
        let preview = good(&root, &["-q", "-o", "json", "--dry-run", "undo"]);
        let result: serde_json::Value = serde_json::from_slice(&preview.stdout).unwrap();
        assert_eq!(result["dry_run"], true);
        assert_eq!(snapshot(&root), saved);
        let output = good(&root, &["-o", "json", "undo"]);
        let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(result["reverted"], original);
        assert_eq!(result["dry_run"], false);
        assert_eq!(git(&root, &["rev-parse", "HEAD^{tree}"]), before);
        assert_eq!(git(&root, &["rev-parse", "HEAD^"]), original);
        assert!(
            git(&root, &["show", "-s", "--format=%s", "HEAD"]).starts_with("chore(tickets): undo ")
        );
        let saved = snapshot(&root);
        let out = cli(&root, &["undo"]);
        assert_eq!(out.status.code(), Some(1), "{out:?}");
        assert_eq!(snapshot(&root), saved);
    }
}

#[test]
fn forged_legacy_duplicate_unknown_and_mismatched_receipts_are_refused() {
    for scenario in [
        "legacy",
        "duplicate",
        "version",
        "tree",
        "mode",
        "semantic",
        "plan",
    ] {
        let (_temp, root) = fixture(false, false);
        good(&root, &["claim", "01"]);
        let message = git(&root, &["show", "-s", "--format=%B", "HEAD"]);
        let (subject, raw) = message.split_once("\n\nTkt-Undo: ").unwrap();
        match scenario {
            "legacy" => amend_message(&root, subject),
            "duplicate" => amend_message(&root, &format!("{message}\nTkt-Undo: {raw}")),
            "version" => {
                let mut v: serde_json::Value = serde_json::from_str(raw).unwrap();
                v["v"] = serde_json::json!(2);
                amend_message(&root, &format!("{subject}\n\nTkt-Undo: {v}"));
            }
            "tree" => {
                std::fs::write(root.join("extra.md"), "Foreign work").unwrap();
                git(&root, &["add", "extra.md"]);
                git(&root, &["commit", "--amend", "--no-edit", "-q"]);
            }
            "mode" => {
                git(
                    &root,
                    &["update-index", "--chmod=+x", ".tickets/01-seed.md"],
                );
                git(&root, &["commit", "--amend", "--no-edit", "-q"]);
            }
            "semantic" | "plan" => {
                let selected = if scenario == "plan" {
                    "docs/plan.md"
                } else {
                    ".tickets/01-seed.md"
                };
                let file = root.join(selected);
                let mut text = std::fs::read_to_string(&file).unwrap();
                text.push_str("\nUnrelated specification change.\n");
                std::fs::write(file, text).unwrap();
                git(&root, &["add", selected]);
                git(&root, &["commit", "--amend", "--no-edit", "-q"]);
                let mut v: serde_json::Value = serde_json::from_str(raw).unwrap();
                v["after_tree"] = serde_json::json!(git(&root, &["rev-parse", "HEAD^{tree}"]));
                let changes = v["changes"].as_array_mut().unwrap();
                for change in changes {
                    if change["path"] == selected {
                        change["after"]["oid"] = serde_json::json!(git(
                            &root,
                            &["rev-parse", &format!("HEAD:{selected}")]
                        ));
                    }
                }
                amend_message(&root, &format!("{subject}\n\nTkt-Undo: {v}"));
            }
            _ => unreachable!(),
        }
        let saved = snapshot(&root);
        let out = cli(&root, &["--dry-run", "undo"]);
        assert_eq!(out.status.code(), Some(1), "{scenario}: {out:?}");
        assert_eq!(snapshot(&root), saved, "{scenario} changed files");
    }
}

#[test]
fn forged_close_checkbox_change_outside_acceptance_is_refused() {
    let (_temp, root) = fixture(false, false);
    let path = root.join(".tickets/01-seed.md");
    let mut body = std::fs::read_to_string(&path).unwrap();
    body.push_str("\n## Background\n\n- [ ] Preserve this unrelated example.\n");
    std::fs::write(&path, body).unwrap();
    git(&root, &["add", ".tickets/01-seed.md"]);
    git(&root, &["commit", "-qm", "seed separate background"]);
    good(
        &root,
        &[
            "close",
            "01",
            "--check-all",
            "--resolution",
            "Verified acceptance",
        ],
    );
    let message = git(&root, &["show", "-s", "--format=%B", "HEAD"]);
    let (subject, raw) = message.split_once("\n\nTkt-Undo: ").unwrap();
    let body = std::fs::read_to_string(&path).unwrap().replace(
        "- [ ] Preserve this unrelated example.",
        "- [x] Preserve this unrelated example.",
    );
    std::fs::write(path, body).unwrap();
    git(&root, &["add", ".tickets/01-seed.md"]);
    git(&root, &["commit", "--amend", "--no-edit", "-q"]);
    let mut receipt: serde_json::Value = serde_json::from_str(raw).unwrap();
    receipt["after_tree"] = serde_json::json!(git(&root, &["rev-parse", "HEAD^{tree}"]));
    for change in receipt["changes"].as_array_mut().unwrap() {
        if change["path"] == ".tickets/01-seed.md" {
            change["after"]["oid"] =
                serde_json::json!(git(&root, &["rev-parse", "HEAD:.tickets/01-seed.md"]));
        }
    }
    amend_message(&root, &format!("{subject}\n\nTkt-Undo: {receipt}"));
    let saved = snapshot(&root);
    let out = cli(&root, &["--dry-run", "undo"]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    assert_eq!(snapshot(&root), saved);
}

#[test]
fn dirty_index_worktree_and_preexisting_git_state_are_preserved() {
    for state in ["worktree", "index", "REVERT_HEAD", "index.lock"] {
        let (_temp, root) = fixture(false, false);
        good(&root, &["claim", "01"]);
        if matches!(state, "worktree" | "index") {
            std::fs::write(root.join("README.md"), "Unrelated change\n").unwrap();
            if state == "index" {
                git(&root, &["add", "README.md"]);
            }
        } else {
            std::fs::write(root.join(".git").join(state), "preexisting state\n").unwrap();
        }
        let saved = snapshot(&root);
        let out = cli(&root, &["undo"]);
        assert_eq!(out.status.code(), Some(1), "{state}: {out:?}");
        assert_eq!(snapshot(&root), saved);
    }
}

#[test]
fn hook_commit_failure_keeps_the_prepared_inverse_and_original_head() {
    let (_temp, root) = fixture(false, true);
    good(&root, &["claim", "01"]);
    let original = git(&root, &["rev-parse", "HEAD"]);
    let prior_tree = git(&root, &["rev-parse", "HEAD^^{tree}"]);
    hook(&root.join(".git/hooks/pre-commit"), "#!/bin/sh\nexit 1\n");
    let out = cli(&root, &["undo"]);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    assert_eq!(git(&root, &["rev-parse", "HEAD"]), original);
    assert_eq!(git(&root, &["write-tree"]), prior_tree);
    assert!(String::from_utf8_lossy(&out.stderr).contains("prepared"));
}

#[test]
fn unexpected_hook_tree_stays_local_and_push_failure_keeps_a_durable_undo_commit() {
    for case in ["hook-tree", "push-failure", "no-push", "published"] {
        let (temp, root) = fixture(true, case == "hook-tree");
        good(&root, &["claim", "01"]);
        let original = git(&root, &["rev-parse", "HEAD"]);
        let prior = git(&root, &["rev-parse", "HEAD^^{tree}"]);
        let remote = temp.path().join("remote.git");
        if case == "hook-tree" {
            hook(&root.join(".git/hooks/pre-commit"),"#!/bin/sh\nprintf '\\nUnexpected hook edit.\\n' >> .tickets/01-seed.md\ngit add .tickets/01-seed.md\n");
        }
        if case == "push-failure" {
            hook(&remote.join("hooks/pre-receive"), "#!/bin/sh\nexit 1\n");
        }
        if case == "no-push" {
            git(
                &root,
                &[
                    "remote",
                    "set-url",
                    "origin",
                    temp.path().join("unreachable.git").to_str().unwrap(),
                ],
            );
        }
        let out = cli(
            &root,
            if case == "no-push" {
                &["--no-push", "undo"]
            } else {
                &["undo"]
            },
        );
        assert_eq!(
            out.status.code(),
            Some(match case {
                "hook-tree" => 1,
                "push-failure" => 2,
                _ => 0,
            }),
            "{case}: {out:?}"
        );
        assert_ne!(git(&root, &["rev-parse", "HEAD"]), original);
        assert_eq!(
            git(&remote, &["rev-parse", "refs/heads/main"]),
            if case == "published" {
                git(&root, &["rev-parse", "HEAD"])
            } else {
                original
            }
        );
        if case != "hook-tree" {
            assert_eq!(git(&root, &["rev-parse", "HEAD^{tree}"]), prior)
        }
    }
}

#[test]
fn root_merge_and_rebased_history_are_refused_without_writes() {
    for scenario in ["root", "merge", "rebase"] {
        let (_temp, root) = fixture(false, false);
        if scenario != "root" {
            good(&root, &["claim", "01"]);
            let message = git(&root, &["show", "-s", "--format=%B", "HEAD"]);
            git(&root, &["checkout", "-qb", "side", "HEAD^"]);
            std::fs::write(root.join("README.md"), "Independent side change\n").unwrap();
            git(&root, &["add", "README.md"]);
            git(&root, &["commit", "-qm", "side"]);
            git(&root, &["checkout", "-q", "main"]);
            if scenario == "merge" {
                git(&root, &["merge", "--no-ff", "-m", &message, "side"]);
            } else {
                git(&root, &["rebase", "side"]);
            }
        }
        let saved = snapshot(&root);
        let out = cli(&root, &["--dry-run", "undo"]);
        assert_eq!(out.status.code(), Some(1), "{scenario}: {out:?}");
        assert_eq!(snapshot(&root), saved);
    }
}

#[test]
fn upstream_advance_keeps_the_exact_local_undo_without_rebase() {
    let (temp, root) = fixture(true, false);
    good(&root, &["claim", "01"]);
    let original = git(&root, &["rev-parse", "HEAD"]);
    git(&root, &["checkout", "-qb", "remote-worker"]);
    std::fs::write(root.join("README.md"), "Remote work\n").unwrap();
    git(&root, &["add", "README.md"]);
    git(&root, &["commit", "-qm", "remote change"]);
    git(&root, &["push", "-q", "origin", "HEAD:main"]);
    let remote = git(&root, &["rev-parse", "HEAD"]);
    git(&root, &["checkout", "-q", "main"]);
    let out = cli(&root, &["undo"]);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    assert_eq!(git(&root, &["rev-parse", "HEAD^"]), original);
    assert_eq!(
        git(
            &temp.path().join("remote.git"),
            &["rev-parse", "refs/heads/main"]
        ),
        remote
    );
    assert!(String::from_utf8_lossy(&out.stderr).contains("does not rebase automatically"));
}

#[cfg(windows)]
#[test]
fn failed_git_revert_preserves_locked_ticket_and_unrelated_work() {
    use std::os::windows::fs::OpenOptionsExt;
    let (_temp, root) = fixture(false, false);
    good(&root, &["claim", "01"]);
    let original = git(&root, &["rev-parse", "HEAD"]);
    let path = root.join(".tickets/01-seed.md");
    let bytes = std::fs::read(&path).unwrap();
    let lock = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(1)
        .open(&path)
        .unwrap();
    let out = cli(&root, &["undo"]);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    assert!(String::from_utf8_lossy(&out.stderr).contains("could not apply the prepared inverse"));
    drop(lock);
    assert_eq!(std::fs::read(path).unwrap(), bytes);
    assert_eq!(git(&root, &["rev-parse", "HEAD"]), original);
    assert_eq!(
        std::fs::read_to_string(root.join("README.md")).unwrap(),
        "Owner source\n"
    );
}

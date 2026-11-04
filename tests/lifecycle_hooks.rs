use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn command(program: impl AsRef<std::ffi::OsStr>) -> Command {
    let mut command = Command::new(program);
    command
        .env("DO_NOT_TRACK", "1")
        .env("TKT_NO_USER_CONFIG", "1")
        .env("TKT_DEBUG", "0")
        .env("TKT_UPDATE_CHECK", "0")
        .env(
            "GIT_CONFIG_GLOBAL",
            if cfg!(windows) { "NUL" } else { "/dev/null" },
        )
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_COUNT", "0")
        .env_remove("GIT_CONFIG_PARAMETERS")
        .env_remove("TKT_HOOK_ACTIVE")
        .env_remove("TKT_AGENT")
        .env_remove("TKT_CONTEXT");
    command
}
fn git(root: &Path, args: &[&str]) -> String {
    let out = command("git")
        .current_dir(root)
        .args(args)
        .output()
        .unwrap();
    assert!(out.status.success(), "{args:?}: {out:?}");
    String::from_utf8(out.stdout).unwrap().trim().into()
}
fn hook(path: &Path, contents: &str) {
    std::fs::write(path, contents).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
}
struct Fixture {
    _temp: tempfile::TempDir,
    root: PathBuf,
}
impl Fixture {
    fn new(trust: bool) -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("repo");
        std::fs::create_dir(&root).unwrap();
        git(&root, &["init", "-q", "-b", "main"]);
        git(&root, &["config", "user.name", "Hook fixture"]);
        git(&root, &["config", "user.email", "fixture@example.invalid"]);
        git(&root, &["config", "maintenance.auto", "false"]);
        git(&root, &["config", "gc.auto", "0"]);
        if trust {
            git(&root, &["config", "--local", "tkt.lifecycleHooks", "true"]);
        }
        std::fs::create_dir(root.join(".tickets")).unwrap();
        let f = Self { _temp: temp, root };
        f.configure(true, false);
        std::fs::write(f.root.join("README.md"), "Fixture\n").unwrap();
        git(&f.root, &["add", "."]);
        git(&f.root, &["commit", "-qm", "fixture"]);
        f
    }
    fn configure(&self, enabled: bool, push: bool) {
        let argv = if enabled {
            serde_json::json!([
                std::env::current_exe().unwrap().to_str().unwrap(),
                "--exact",
                "hook_child",
                "--nocapture"
            ])
        } else {
            serde_json::json!([])
        };
        std::fs::write(self.root.join(".tickets/config.toml"), format!("[push]\nenabled={push}\n[hooks]\non_new={argv}\non_claim={argv}\non_close={argv}\n")).unwrap();
    }
    fn cli(&self, args: &[&str], mode: &str) -> Output {
        command(env!("CARGO_BIN_EXE_tkt"))
            .current_dir(&self.root)
            .env("XDG_CONFIG_HOME", self._temp.path().join("config"))
            .env("HOOK_FIXTURE_MODE", mode)
            .env("HOOK_FIXTURE_LOG", self.root.join(".git/hook-events.jsonl"))
            .args(args)
            .output()
            .unwrap()
    }
    fn events(&self) -> Vec<serde_json::Value> {
        std::fs::read_to_string(self.root.join(".git/hook-events.jsonl"))
            .unwrap_or_default()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }
    fn new_ticket(&self) {
        let out = self.cli(&["new", "work", "--body", "Implement real work"], "ok");
        assert!(out.status.success(), "{out:?}");
    }
}

#[test]
fn hook_child() {
    if std::env::var("TKT_HOOK_ACTIVE").as_deref() != Ok("1") {
        return;
    }
    use std::io::Write;
    let root = PathBuf::from(std::env::var_os("TKT_HOOK_REPO").unwrap());
    let commit = std::env::var("TKT_HOOK_COMMIT").unwrap();
    assert_eq!(std::env::current_dir().unwrap(), root);
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(
            std::env::var_os("HOOK_FIXTURE_LOG")
                .map(PathBuf::from)
                .unwrap_or_else(|| root.join(".git/hook-events.jsonl")),
        )
        .unwrap();
    writeln!(file, "{}", serde_json::json!({"event":std::env::var("TKT_HOOK_EVENT").unwrap(),"id":std::env::var("TKT_HOOK_ID").unwrap(),"commit":commit})).unwrap();
    match std::env::var("HOOK_FIXTURE_MODE").unwrap().as_str() {
        "fail" => std::process::exit(17),
        "timeout" => std::thread::sleep(std::time::Duration::from_secs(20)),
        "nested" => {
            let out = Command::new(env!("CARGO_BIN_EXE_tkt"))
                .current_dir(&root)
                .env("DO_NOT_TRACK", "1")
                .args(["new", "nested", "--body", "Nested fixture work"])
                .output()
                .unwrap();
            assert!(out.status.success(), "{out:?}");
        }
        "published" => {
            assert_eq!(
                git(&root, &["rev-parse", "refs/remotes/origin/main"]),
                commit
            );
        }
        _ => {}
    }
}

#[test]
fn explicit_local_trust_is_required_and_exact() {
    for value in [
        None,
        Some("false"),
        Some("TRUE"),
        Some(" true "),
        Some("1"),
        Some("true"),
    ] {
        let f = Fixture::new(false);
        if let Some(value) = value {
            git(&f.root, &["config", "--local", "tkt.lifecycleHooks", value]);
        }
        f.new_ticket();
        assert_eq!(f.events().len(), usize::from(value == Some("true")));
    }
    let f = Fixture::new(true);
    git(
        &f.root,
        &["config", "--local", "--add", "tkt.lifecycleHooks", "true"],
    );
    f.new_ticket();
    assert!(f.events().is_empty());
    let f = Fixture::new(false);
    git(
        &f.root,
        &["config", "--local", "--add", "tkt.lifecycleHooks", ""],
    );
    git(
        &f.root,
        &["config", "--local", "--add", "tkt.lifecycleHooks", "true"],
    );
    f.new_ticket();
    assert!(f.events().is_empty());
}
#[test]
fn all_events_use_committed_identity_and_preserve_json_output() {
    let f = Fixture::new(true);
    for args in [
        vec!["new", "work", "--body", "Implement real work", "-o", "json"],
        vec!["claim", "01", "-o", "json"],
        vec![
            "close",
            "01",
            "--resolution",
            "Implemented and verified",
            "-o",
            "json",
        ],
    ] {
        let out = f.cli(&args, "ok");
        assert!(out.status.success(), "{out:?}");
        let _: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(
            f.events().last().unwrap()["commit"],
            git(&f.root, &["rev-parse", "HEAD"])
        );
    }
    let events = f.events();
    assert_eq!(
        events
            .iter()
            .map(|v| v["event"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["new", "claim", "close"]
    );
    assert!(events.iter().all(|v| v["id"] == "01"));
}
#[test]
fn dry_run_default_and_failed_commit_never_execute() {
    let f = Fixture::new(true);
    let head = git(&f.root, &["rev-parse", "HEAD"]);
    let out = f.cli(&["--dry-run", "new", "preview", "--body", "Preview"], "ok");
    assert!(out.status.success(), "{out:?}");
    assert!(f.events().is_empty());
    assert_eq!(git(&f.root, &["rev-parse", "HEAD"]), head);
    f.configure(false, false);
    f.new_ticket();
    assert!(f.events().is_empty());
    f.configure(true, false);
    let head = git(&f.root, &["rev-parse", "HEAD"]);
    for args in [
        vec!["--dry-run", "claim", "01"],
        vec![
            "--dry-run",
            "close",
            "01",
            "--resolution",
            "Preview closure",
        ],
    ] {
        let out = f.cli(&args, "ok");
        assert!(out.status.success(), "{out:?}");
        assert!(f.events().is_empty());
        assert_eq!(git(&f.root, &["rev-parse", "HEAD"]), head);
    }
    std::fs::write(
        f.root.join(".tickets/config.toml"),
        std::fs::read_to_string(f.root.join(".tickets/config.toml")).unwrap()
            + "[commit]\nhooks=\"run\"\n",
    )
    .unwrap();
    git(&f.root, &["add", "."]);
    git(&f.root, &["commit", "-qm", "configure"]);
    hook(&f.root.join(".git/hooks/pre-commit"), "#!/bin/sh\nexit 1\n");
    let out = f.cli(&["claim", "01"], "ok");
    assert!(!out.status.success(), "{out:?}");
    assert!(f.events().is_empty());
}
#[test]
fn failure_timeout_and_missing_executable_are_advisory() {
    for mode in ["fail", "timeout", "missing"] {
        let f = Fixture::new(true);
        if mode == "missing" {
            std::fs::write(
                f.root.join(".tickets/config.toml"),
                "[push]\nenabled=false\n[hooks]\non_new=[\"tkt-missing-hook-executable\"]\n",
            )
            .unwrap();
        }
        let start = std::time::Instant::now();
        let out = f.cli(&["new", "work", "--body", "Real work", "-o", "json"], mode);
        assert!(out.status.success(), "{out:?}");
        assert!(start.elapsed() < std::time::Duration::from_secs(10));
        assert!(
            String::from_utf8_lossy(&out.stderr).contains("advisory hook"),
            "{out:?}"
        );
        let _: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
        assert!(f.root.join(".tickets/01-work.md").is_file());
    }
}
#[test]
fn publication_precedes_hooks_and_failed_push_keeps_event() {
    let f = Fixture::new(true);
    let remote = f._temp.path().join("remote.git");
    git(&f.root, &["init", "--bare", remote.to_str().unwrap()]);
    git(
        &f.root,
        &["remote", "add", "origin", remote.to_str().unwrap()],
    );
    git(&f.root, &["push", "-qu", "origin", "main"]);
    f.configure(true, true);
    git(&f.root, &["add", "."]);
    git(&f.root, &["commit", "-qm", "enable push"]);
    git(&f.root, &["push", "-q"]);
    let out = f.cli(&["new", "work", "--body", "Real work"], "published");
    assert!(out.status.success(), "{out:?}");
    hook(&remote.join("hooks/pre-receive"), "#!/bin/sh\nexit 1\n");
    let out = f.cli(&["claim", "01"], "ok");
    assert!(!out.status.success(), "{out:?}");
    assert_eq!(f.events().last().unwrap()["event"], "claim");
    assert_eq!(
        f.events().last().unwrap()["commit"],
        git(&f.root, &["rev-parse", "HEAD"])
    );
}
#[test]
fn recursion_guard_suppresses_nested_events() {
    let f = Fixture::new(true);
    f.new_ticket();
    let out = f.cli(&["claim", "01"], "nested");
    assert!(out.status.success(), "{out:?}");
    assert_eq!(f.events().len(), 2);
    assert!(f.root.join(".tickets/02-nested.md").is_file());
}

#[test]
fn rejected_allocation_only_emits_the_final_ticket() {
    let f = Fixture::new(true);
    let remote = f._temp.path().join("remote.git");
    git(&f.root, &["init", "--bare", remote.to_str().unwrap()]);
    git(
        &f.root,
        &["remote", "add", "origin", remote.to_str().unwrap()],
    );
    git(&f.root, &["push", "-qu", "origin", "main"]);
    f.configure(true, true);
    git(&f.root, &["add", "."]);
    git(&f.root, &["commit", "-qm", "push configuration"]);
    git(&f.root, &["push", "-q"]);
    let other = f._temp.path().join("other");
    git(
        &f.root,
        &[
            "clone",
            "-q",
            "--branch",
            "main",
            remote.to_str().unwrap(),
            other.to_str().unwrap(),
        ],
    );
    git(&other, &["config", "user.name", "Other fixture"]);
    git(&other, &["config", "user.email", "other@example.invalid"]);
    std::fs::write(other.join(".tickets/01-race.md"),"---\nid: \"01\"\ntitle: \"Other work\"\nstatus: open\nblocked_by: []\n---\n\n# Other work\n").unwrap();
    git(&other, &["add", "."]);
    git(&other, &["commit", "-qm", "other allocation"]);
    let raced = git(&other, &["rev-parse", "HEAD"]);
    git(
        &other,
        &["push", "-q", "origin", "HEAD:refs/heads/prepared"],
    );
    let bin = f._temp.path().join("git-probe");
    std::fs::create_dir(&bin).unwrap();
    let git_name = if cfg!(windows) { "git.exe" } else { "git" };
    let real_git = std::env::split_paths(&std::env::var_os("PATH").unwrap())
        .map(|path| path.join(git_name))
        .find(|path| path.is_file())
        .unwrap();
    let compiled = command("rustc")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/support/git_cas_probe.rs"))
        .arg("-o")
        .arg(bin.join(git_name))
        .output()
        .unwrap();
    assert!(compiled.status.success(), "{compiled:?}");
    let race = f._temp.path().join("race.txt");
    let old = git(&f.root, &["rev-parse", "refs/remotes/origin/main"]);
    std::fs::write(
        &race,
        format!("{}\n{raced}\n{old}\nnormal\n", remote.display()),
    )
    .unwrap();
    let path = std::env::join_paths(
        std::iter::once(bin).chain(std::env::split_paths(&std::env::var_os("PATH").unwrap())),
    )
    .unwrap();
    let out = command(env!("CARGO_BIN_EXE_tkt"))
        .current_dir(&f.root)
        .env("HOOK_FIXTURE_MODE", "ok")
        .env("PATH", path)
        .env("TKT_TEST_REAL_GIT", real_git)
        .env("TKT_TEST_GIT_RACE", race)
        .args(["new", "work", "--body", "Final allocated work"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    assert_eq!(f.events().len(), 1);
    assert_eq!(f.events()[0]["id"], "02");
    assert_eq!(
        f.events()[0]["commit"],
        git(&f.root, &["rev-parse", "HEAD"])
    );
}
#[test]
fn invalid_hook_configuration_fails_before_ticket_creation() {
    for value in ["[1]", "[\"\"]", "{}", "[\"bad\\u0000program\"]"] {
        let f = Fixture::new(true);
        std::fs::write(
            f.root.join(".tickets/config.toml"),
            format!("[hooks]\non_new={value}\n"),
        )
        .unwrap();
        let head = git(&f.root, &["rev-parse", "HEAD"]);
        let out = f.cli(&["new", "work", "--body", "Work"], "ok");
        assert!(!out.status.success(), "{out:?}");
        assert_eq!(git(&f.root, &["rev-parse", "HEAD"]), head);
        assert!(!f.root.join(".tickets/01-work.md").exists());
        assert!(f.events().is_empty());
    }
}

#[test]
fn local_trust_is_shared_with_linked_worktrees() {
    let f = Fixture::new(true);
    let linked = f._temp.path().join("linked");
    git(
        &f.root,
        &["worktree", "add", "-qb", "linked", linked.to_str().unwrap()],
    );
    let out = command(env!("CARGO_BIN_EXE_tkt"))
        .current_dir(&linked)
        .env("HOOK_FIXTURE_MODE", "ok")
        .env("HOOK_FIXTURE_LOG", f.root.join(".git/hook-events.jsonl"))
        .args(["new", "linked-work", "--body", "Linked work"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    assert_eq!(f.events().len(), 1);
    assert_eq!(
        f.events()[0]["commit"],
        git(&linked, &["rev-parse", "HEAD"])
    );
}

#[test]
fn project_global_and_included_configuration_cannot_self_authorize() {
    for source in ["project", "global", "include"] {
        let f = Fixture::new(false);
        let trust_file = f._temp.path().join("untrusted-config");
        std::fs::write(&trust_file, "[tkt]\nlifecycleHooks=true\n").unwrap();
        if source == "include" {
            git(
                &f.root,
                &[
                    "config",
                    "--local",
                    "include.path",
                    trust_file.to_str().unwrap(),
                ],
            );
        }
        if source == "project" {
            let path = f.root.join(".tickets/config.toml");
            let mut text = std::fs::read_to_string(&path).unwrap();
            text += "[tkt]\nlifecycleHooks=true\n";
            std::fs::write(path, text).unwrap();
        }
        let mut child = command(env!("CARGO_BIN_EXE_tkt"));
        child
            .current_dir(&f.root)
            .env("HOOK_FIXTURE_MODE", "ok")
            .args(["new", "work", "--body", "Real work"]);
        if source == "global" {
            child.env("GIT_CONFIG_GLOBAL", &trust_file);
        }
        let out = child.output().unwrap();
        assert!(out.status.success(), "{out:?}");
        assert!(f.events().is_empty());
    }
}
#[test]
fn failed_new_and_close_commits_emit_no_event() {
    for event in ["new", "close"] {
        let f = Fixture::new(true);
        f.configure(false, false);
        f.new_ticket();
        f.configure(true, false);
        let path = f.root.join(".tickets/config.toml");
        let mut config = std::fs::read_to_string(&path).unwrap();
        config += "[commit]\nhooks=\"run\"\n";
        std::fs::write(path, config).unwrap();
        git(&f.root, &["add", "."]);
        git(&f.root, &["commit", "-qm", "enable hooks"]);
        hook(&f.root.join(".git/hooks/pre-commit"), "#!/bin/sh\nexit 1\n");
        let args = if event == "new" {
            vec!["new", "second", "--body", "Second work"]
        } else {
            vec!["close", "01", "--resolution", "Implemented and verified"]
        };
        let out = f.cli(&args, "ok");
        assert!(!out.status.success(), "{out:?}");
        assert!(f.events().is_empty());
    }
}

#[test]
fn unrelated_head_advance_after_push_never_becomes_the_event_revision() {
    let f = Fixture::new(true);
    f.new_ticket();
    let remote = f._temp.path().join("remote.git");
    git(&f.root, &["init", "--bare", remote.to_str().unwrap()]);
    git(
        &f.root,
        &["remote", "add", "origin", remote.to_str().unwrap()],
    );
    git(&f.root, &["push", "-qu", "origin", "main"]);
    f.configure(true, true);
    git(&f.root, &["add", "."]);
    git(&f.root, &["commit", "-qm", "push configuration"]);
    git(&f.root, &["push", "-q"]);
    let bin = f._temp.path().join("git-probe");
    std::fs::create_dir(&bin).unwrap();
    let git_name = if cfg!(windows) { "git.exe" } else { "git" };
    let real_git = std::env::split_paths(&std::env::var_os("PATH").unwrap())
        .map(|path| path.join(git_name))
        .find(|path| path.is_file())
        .unwrap();
    let compiled = command("rustc")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/support/git_cas_probe.rs"))
        .arg("-o")
        .arg(bin.join(git_name))
        .output()
        .unwrap();
    assert!(compiled.status.success(), "{compiled:?}");
    let race = f._temp.path().join("race.txt");
    std::fs::write(&race, "unused\nunused\nunused\nafter_push\n").unwrap();
    let path = std::env::join_paths(
        std::iter::once(bin).chain(std::env::split_paths(&std::env::var_os("PATH").unwrap())),
    )
    .unwrap();
    let out = command(env!("CARGO_BIN_EXE_tkt"))
        .current_dir(&f.root)
        .env("HOOK_FIXTURE_MODE", "ok")
        .env("PATH", path)
        .env("TKT_TEST_REAL_GIT", real_git)
        .env("TKT_TEST_GIT_RACE", &race)
        .args(["claim", "01"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    assert!(!race.exists());
    assert_ne!(
        git(&f.root, &["rev-parse", "HEAD"]),
        git(&f.root, &["rev-parse", "refs/remotes/origin/main"])
    );
    assert_eq!(
        git(&f.root, &["log", "-1", "--format=%s"]),
        "concurrent mutation"
    );
    assert_eq!(
        f.events().len(),
        1,
        "unrelated HEAD must never be emitted: {:?}",
        f.events()
    );
}

#[test]
fn unrelated_head_advance_before_capture_never_becomes_the_event_revision() {
    let f = Fixture::new(true);
    f.new_ticket();
    let remote = f._temp.path().join("remote.git");
    git(&f.root, &["init", "--bare", remote.to_str().unwrap()]);
    git(
        &f.root,
        &["remote", "add", "origin", remote.to_str().unwrap()],
    );
    git(&f.root, &["push", "-qu", "origin", "main"]);
    f.configure(true, true);
    git(&f.root, &["add", "."]);
    git(&f.root, &["commit", "-qm", "push configuration"]);
    git(&f.root, &["push", "-q"]);
    let bin = f._temp.path().join("git-probe");
    std::fs::create_dir(&bin).unwrap();
    let git_name = if cfg!(windows) { "git.exe" } else { "git" };
    let real_git = std::env::split_paths(&std::env::var_os("PATH").unwrap())
        .map(|path| path.join(git_name))
        .find(|path| path.is_file())
        .unwrap();
    let compiled = command("rustc")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/support/git_cas_probe.rs"))
        .arg("-o")
        .arg(bin.join(git_name))
        .output()
        .unwrap();
    assert!(compiled.status.success(), "{compiled:?}");
    let race = f._temp.path().join("race.txt");
    std::fs::write(&race, "unused\nunused\nunused\nafter_commit\n").unwrap();
    let path = std::env::join_paths(
        std::iter::once(bin).chain(std::env::split_paths(&std::env::var_os("PATH").unwrap())),
    )
    .unwrap();
    let out = command(env!("CARGO_BIN_EXE_tkt"))
        .current_dir(&f.root)
        .env("HOOK_FIXTURE_MODE", "ok")
        .env("PATH", path)
        .env("TKT_TEST_REAL_GIT", real_git)
        .env("TKT_TEST_GIT_RACE", &race)
        .args(["claim", "01"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    assert!(!race.exists());
    assert_eq!(
        git(&f.root, &["rev-parse", "HEAD"]),
        git(&f.root, &["rev-parse", "refs/remotes/origin/main"])
    );
    assert_eq!(
        git(&f.root, &["log", "-1", "--format=%s"]),
        "concurrent mutation"
    );
    assert_eq!(
        f.events().len(),
        1,
        "unrelated HEAD must never be emitted: {:?}",
        f.events()
    );
}

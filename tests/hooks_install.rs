use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn isolated_command(program: &str) -> Command {
    let mut command = Command::new(program);
    command
        .env(
            "GIT_CONFIG_GLOBAL",
            if cfg!(windows) { "NUL" } else { "/dev/null" },
        )
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env_remove("GIT_CONFIG_COUNT")
        .env_remove("GIT_CONFIG_PARAMETERS")
        .env("DO_NOT_TRACK", "1");
    command
}

fn git(root: &Path, args: &[&str]) -> String {
    let out = isolated_command("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .env("DO_NOT_TRACK", "1")
        .output()
        .unwrap();
    assert!(out.status.success(), "{args:?}: {out:?}");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}
fn fixture() -> (tempfile::TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("repo");
    std::fs::create_dir(&root).unwrap();
    git(&root, &["init", "-q", "-b", "main"]);
    git(&root, &["config", "user.name", "Hook fixture"]);
    git(&root, &["config", "user.email", "fixture@example.invalid"]);
    std::fs::create_dir(root.join(".tickets")).unwrap();
    std::fs::write(
        root.join(".tickets/01-seed.md"),
        "---\nid: \"01\"\ntitle: \"Seed\"\nstatus: open\nblocked_by: []\n---\n\nSeed work.\n",
    )
    .unwrap();
    git(&root, &["add", ".tickets"]);
    git(&root, &["-c", "core.hooksPath=", "commit", "-qm", "seed"]);
    (temp, root)
}
fn cli(root: &Path, args: &[&str]) -> Output {
    isolated_command(env!("CARGO_BIN_EXE_tkt"))
        .args(args)
        .current_dir(root)
        .env("DO_NOT_TRACK", "1")
        .env("TKT_NO_USER_CONFIG", "1")
        .env("TKT_UPDATE_CHECK", "0")
        .output()
        .unwrap()
}
fn succeeds(root: &Path, args: &[&str]) -> serde_json::Value {
    let mut full = vec!["-q", "-o", "json"];
    full.extend_from_slice(args);
    let out = cli(root, &full);
    assert!(out.status.success(), "{args:?}: {out:?}");
    serde_json::from_slice(&out.stdout).unwrap()
}

#[test]
fn explicit_advisory_install_preview_ownership_and_remove() {
    let (_temp, root) = fixture();
    let hook = root.join(".git/hooks/pre-commit");
    let head = git(&root, &["rev-parse", "HEAD"]);
    let config = std::fs::read(root.join(".git/config")).unwrap();
    assert_eq!(succeeds(&root, &["hooks", "check"])["state"], "absent");
    assert_eq!(
        succeeds(&root, &["--dry-run", "hooks", "install"])["state"],
        "would-install"
    );
    assert!(!hook.exists());
    assert_eq!(succeeds(&root, &["hooks", "install"])["installed"], true);
    let bytes = std::fs::read(&hook).unwrap();
    assert_eq!(bytes, include_bytes!("../assets/pre-commit.sh"));
    assert!(!bytes.contains(&b'\r'));
    assert_eq!(succeeds(&root, &["hooks", "install"])["state"], "installed");
    assert_eq!(succeeds(&root, &["hooks", "check"])["advisory"], true);
    succeeds(&root, &["--dry-run", "hooks", "remove"]);
    assert_eq!(std::fs::read(&hook).unwrap(), bytes);
    succeeds(&root, &["hooks", "remove"]);
    succeeds(&root, &["hooks", "remove"]);
    assert!(!hook.exists());
    assert_eq!(git(&root, &["rev-parse", "HEAD"]), head);
    assert_eq!(std::fs::read(root.join(".git/config")).unwrap(), config);
    assert!(git(&root, &["status", "--porcelain"]).is_empty());
    assert!(!std::fs::read_dir(root.join(".git/hooks"))
        .unwrap()
        .any(|e| e
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".tkt-")));
}

#[test]
fn foreign_modified_and_custom_hooks_are_preserved() {
    let (_temp, root) = fixture();
    let hook = root.join(".git/hooks/pre-commit");
    for bytes in [
        b"#!/bin/sh\nexit 17\n".as_slice(),
        &[
            include_bytes!("../assets/pre-commit.sh").as_slice(),
            b"# user edit\n",
        ]
        .concat(),
    ] {
        std::fs::write(&hook, bytes).unwrap();
        assert_eq!(succeeds(&root, &["hooks", "check"])["state"], "foreign");
        for args in [["hooks", "install"], ["hooks", "remove"]] {
            let out = cli(&root, &args);
            assert_eq!(out.status.code(), Some(1), "{out:?}");
            assert_eq!(std::fs::read(&hook).unwrap(), bytes);
        }
    }
    git(&root, &["config", "core.hooksPath", "custom-hooks"]);
    let before = std::fs::read(root.join(".git/config")).unwrap();
    assert_eq!(
        succeeds(&root, &["hooks", "check"])["state"],
        "custom-hooks-path"
    );
    assert_eq!(cli(&root, &["hooks", "install"]).status.code(), Some(1));
    assert_eq!(std::fs::read(root.join(".git/config")).unwrap(), before);
    assert!(!root.join("custom-hooks").exists());
}

#[test]
fn shared_worktree_install_and_advisory_git_commit() {
    let (temp, root) = fixture();
    let worktree = temp.path().join("linked");
    git(
        &root,
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            "linked",
            worktree.to_str().unwrap(),
        ],
    );
    succeeds(&worktree, &["hooks", "install"]);
    assert!(root.join(".git/hooks/pre-commit").exists());
    assert_eq!(
        succeeds(&root, &["hooks", "check"])["shared_worktrees"],
        true
    );
    std::fs::write(
        root.join(".tickets/02-invalid.md"),
        "---\nid: \"02\"\ntitle: \"Broken\"\nstatus: unsupported\nblocked_by: []\n---\n",
    )
    .unwrap();
    git(&root, &["add", ".tickets/02-invalid.md"]);
    let binary_dir = Path::new(env!("CARGO_BIN_EXE_tkt")).parent().unwrap();
    let paths = std::iter::once(binary_dir.to_path_buf())
        .chain(std::env::split_paths(&std::env::var_os("PATH").unwrap()))
        .collect::<Vec<_>>();
    let out = isolated_command("git")
        .arg("-C")
        .arg(&root)
        .args(["commit", "-qm", "advisory permits commit"])
        .env("PATH", std::env::join_paths(paths).unwrap())
        .env("DO_NOT_TRACK", "1")
        .env("TKT_NO_USER_CONFIG", "1")
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    let text = String::from_utf8_lossy(&out.stderr);
    assert!(
        text.contains("advisory") && text.contains("unparseable"),
        "{out:?}"
    );
    succeeds(&worktree, &["hooks", "remove"]);
    assert!(!root.join(".git/hooks/pre-commit").exists());
}

#[test]
fn linked_hook_file_is_never_read_or_replaced() {
    let (temp, root) = fixture();
    let foreign = temp.path().join("foreign-hook");
    std::fs::write(&foreign, "foreign bytes").unwrap();
    let hook = root.join(".git/hooks/pre-commit");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&foreign, &hook).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_file(&foreign, &hook).unwrap();
    assert_eq!(cli(&root, &["hooks", "install"]).status.code(), Some(1));
    assert_eq!(cli(&root, &["hooks", "remove"]).status.code(), Some(1));
    assert_eq!(std::fs::read(&foreign).unwrap(), b"foreign bytes");
    assert!(std::fs::symlink_metadata(&hook)
        .unwrap()
        .file_type()
        .is_symlink());
}
#[test]
#[cfg(unix)]
fn owned_unix_hook_mode_is_inspected_and_preserved() {
    use std::os::unix::fs::PermissionsExt;
    let (_temp, root) = fixture();
    succeeds(&root, &["hooks", "install"]);
    let hook = root.join(".git/hooks/pre-commit");
    std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o644)).unwrap();
    assert_eq!(
        succeeds(&root, &["hooks", "check"])["state"],
        "managed-not-executable"
    );
    assert_eq!(
        cli(&root, &["--dry-run", "hooks", "install"]).status.code(),
        Some(1)
    );
    assert_eq!(
        std::fs::metadata(&hook).unwrap().permissions().mode() & 0o111,
        0
    );
    assert_eq!(cli(&root, &["hooks", "install"]).status.code(), Some(1));
    assert_eq!(
        std::fs::metadata(&hook).unwrap().permissions().mode() & 0o111,
        0
    );
    std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(succeeds(&root, &["hooks", "check"])["installed"], true);
    assert_ne!(
        std::fs::metadata(&hook).unwrap().permissions().mode() & 0o111,
        0
    );
}

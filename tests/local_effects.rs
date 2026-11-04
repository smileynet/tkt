use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn cli(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tkt"))
        .args(args)
        .current_dir(root)
        .env("DO_NOT_TRACK", "1")
        .env("TKT_NO_USER_CONFIG", "1")
        .env("TKT_UPDATE_CHECK", "0")
        .env("TKT_DEBUG", "0")
        .env("XDG_CONFIG_HOME", root.join("private-config"))
        .env("XDG_STATE_HOME", root.join("private-state"))
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env(
            "GIT_CONFIG_GLOBAL",
            if cfg!(windows) { "NUL" } else { "/dev/null" },
        )
        .env("GIT_CONFIG_COUNT", "0")
        .env_remove("GIT_CONFIG_PARAMETERS")
        .output()
        .unwrap()
}

fn repo() -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    let out = Command::new("git")
        .args(["init", "-q"])
        .arg(temp.path())
        .env("DO_NOT_TRACK", "1")
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    temp
}

fn success(root: &Path, args: &[&str]) {
    let out = cli(root, args);
    assert!(out.status.success(), "{args:?}: {out:?}");
}

fn tree(root: &Path) -> BTreeMap<PathBuf, Option<Vec<u8>>> {
    fn walk(root: &Path, dir: &Path, result: &mut BTreeMap<PathBuf, Option<Vec<u8>>>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            let value = if path.is_dir() {
                None
            } else {
                Some(std::fs::read(&path).unwrap())
            };
            let directory = value.is_none();
            result.insert(path.strip_prefix(root).unwrap().to_path_buf(), value);
            if directory {
                walk(root, &path, result);
            }
        }
    }
    let mut result = BTreeMap::new();
    walk(root, root, &mut result);
    result
}

#[test]
fn local_cli_previews_preserve_full_tree_and_apply_controls_work() {
    let temp = repo();
    let root = temp.path();
    let before = tree(root);
    success(root, &["--dry-run", "init", "--all"]);
    assert_eq!(tree(root), before);
    success(root, &["init", "--all"]);
    success(root, &["context", "backend"]);
    success(root, &["config", "--set", "debug=false"]);
    for args in [
        vec!["--dry-run", "init", "--all"],
        vec!["--dry-run", "init", "--write", "nested/NOTES.md"],
        vec!["--dry-run", "context", "frontend"],
        vec!["--dry-run", "context", "--clear"],
        vec!["--dry-run", "config", "--set", "debug=true"],
        vec!["--dry-run", "config", "--unset", "debug"],
        vec![
            "--dry-run",
            "config",
            "--project",
            "--set",
            "push.enabled=false",
        ],
        vec![
            "--dry-run",
            "config",
            "--project",
            "--unset",
            "push.enabled",
        ],
    ] {
        let before = tree(root);
        success(root, &args);
        assert_eq!(tree(root), before, "{args:?} changed persistent state");
    }
    success(root, &["context", "frontend"]);
    assert_eq!(
        std::fs::read_to_string(root.join(".tickets/.context")).unwrap(),
        "+frontend\n"
    );
    success(root, &["context", "--clear"]);
    success(root, &["context", "--clear"]);
    assert!(!root.join(".tickets/.context").exists());
    success(
        root,
        &["config", "--project", "--set", "push.enabled=false"],
    );
    let config = std::fs::read_to_string(root.join(".tickets/config.toml")).unwrap();
    assert!(config.contains("enabled = false"));
    assert!(config.contains("require_validation_criteria"));
    std::fs::write(root.join("AGENTS.md"), "Owner prose\n").unwrap();
    success(root, &["init", "--agent-only", "--write", "AGENTS.md"]);
    let merged = std::fs::read_to_string(root.join("AGENTS.md")).unwrap();
    assert!(merged.starts_with("Owner prose\n"));
    success(root, &["init", "--agent-only", "--write", "AGENTS.md"]);
    assert_eq!(
        std::fs::read_to_string(root.join("AGENTS.md")).unwrap(),
        merged
    );
}

#[test]
fn unreadable_text_is_preserved_instead_of_becoming_empty_configuration_or_ignore() {
    let temp = repo();
    let root = temp.path();
    success(root, &["init"]);
    let ignore = root.join(".tickets/.gitignore");
    std::fs::write(&ignore, [0xff, 0xfe]).unwrap();
    let before = tree(root);
    for args in [
        vec!["-o", "json", "init"],
        vec!["-o", "json", "--dry-run", "init"],
    ] {
        let out = cli(root, &args);
        assert_eq!(out.status.code(), Some(2), "{out:?}");
        assert!(String::from_utf8_lossy(&out.stderr).contains("io"));
        assert_eq!(tree(root), before);
    }
    let config = root.join("private-config/tkt/config.toml");
    std::fs::create_dir_all(config.parent().unwrap()).unwrap();
    std::fs::write(&config, [0xff, 0xfe]).unwrap();
    let before = tree(root);
    for args in [
        vec!["-o", "json", "config", "--set", "debug=true"],
        vec!["-o", "json", "--dry-run", "config", "--set", "debug=true"],
    ] {
        let out = cli(root, &args);
        assert_eq!(out.status.code(), Some(2), "{out:?}");
        assert!(String::from_utf8_lossy(&out.stderr).contains("io"));
        assert_eq!(tree(root), before);
    }
}

#[test]
fn linked_write_targets_are_preserved_and_context_clear_unlinks_without_following() {
    let temp = repo();
    let root = temp.path();
    success(root, &["init"]);
    let foreign = root.join("foreign.txt");
    std::fs::write(&foreign, "Foreign bytes\n").unwrap();
    let link = root.join("AGENTS.md");
    #[cfg(windows)]
    std::os::windows::fs::symlink_file(&foreign, &link).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(&foreign, &link).unwrap();
    let out = cli(
        root,
        &["-o", "json", "init", "--agent-only", "--write", "AGENTS.md"],
    );
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    assert!(std::fs::symlink_metadata(&link)
        .unwrap()
        .file_type()
        .is_symlink());
    assert_eq!(
        std::fs::read_to_string(&foreign).unwrap(),
        "Foreign bytes\n"
    );
    let context = root.join(".tickets/.context");
    #[cfg(windows)]
    std::os::windows::fs::symlink_file(&foreign, &context).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(&foreign, &context).unwrap();
    let out = cli(root, &["context", "backend"]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    success(root, &["context", "--clear"]);
    assert!(std::fs::symlink_metadata(&context).is_err());
    assert_eq!(
        std::fs::read_to_string(&foreign).unwrap(),
        "Foreign bytes\n"
    );
}

#[cfg(windows)]
#[test]
fn locked_context_atomic_replace_preserves_original_and_cleans_staging() {
    use std::os::windows::fs::OpenOptionsExt;
    let temp = repo();
    let root = temp.path();
    success(root, &["init"]);
    success(root, &["context", "backend"]);
    let before = tree(root);
    let lock = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(1)
        .open(root.join(".tickets/.context"))
        .unwrap();
    let out = cli(root, &["-o", "json", "context", "frontend"]);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    assert!(String::from_utf8_lossy(&out.stderr).contains("io"));
    drop(lock);
    assert_eq!(tree(root), before);
}

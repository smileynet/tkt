use std::path::PathBuf;
use std::process::{Command, Output};

use tempfile::TempDir;

struct Fixture {
    _temp: TempDir,
    repo: PathBuf,
    user_config: PathBuf,
}

fn fixture() -> Fixture {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");
    let user_config = temp.path().join("user-config");
    std::fs::create_dir_all(repo.join(".tickets")).unwrap();
    let init = Command::new("git")
        .args(["init", "-q"])
        .current_dir(&repo)
        .env("DO_NOT_TRACK", "1")
        .output()
        .unwrap();
    assert!(init.status.success());
    Fixture {
        _temp: temp,
        repo,
        user_config,
    }
}

fn cli(fixture: &Fixture, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tkt"))
        .args(args)
        .current_dir(&fixture.repo)
        .env("DO_NOT_TRACK", "1")
        .env("TKT_NO_USER_CONFIG", "0")
        .env("XDG_CONFIG_HOME", &fixture.user_config)
        .env("TKT_UPDATE_CHECK", "0")
        .env_remove("TKT_DEBUG")
        .env_remove("TKT_DEBUG_FORMAT")
        .output()
        .unwrap()
}

#[test]
fn project_set_and_unset_write_typed_values_in_sections() {
    let fixture = fixture();
    let set_bool = cli(
        &fixture,
        &["config", "--set", "push.enabled=false", "--project"],
    );
    assert!(set_bool.status.success(), "{set_bool:?}");
    let path = fixture.repo.join(".tickets/config.toml");
    let contents = std::fs::read_to_string(&path).unwrap();
    assert!(contents.contains("[push]\nenabled = false\n"), "{contents}");
    assert!(!contents.contains("enabled = \"false\""), "{contents}");
    assert!(!fixture.user_config.join("tkt/config.toml").exists());

    let set_label = cli(
        &fixture,
        &[
            "config",
            "--project",
            "--set",
            "labels.triage.single_select=true",
        ],
    );
    assert!(set_label.status.success(), "{set_label:?}");
    let set_values = cli(
        &fixture,
        &[
            "config",
            "--project",
            "--set",
            r#"labels.triage.values=["ready","in progress"]"#,
        ],
    );
    assert!(set_values.status.success(), "{set_values:?}");
    let contents = std::fs::read_to_string(&path).unwrap();
    assert!(
        contents.contains("[labels.triage]\nsingle_select = true\n"),
        "{contents}"
    );
    assert!(
        contents.contains("values = [\"ready\",\"in progress\"]"),
        "{contents}"
    );
    let show = cli(&fixture, &["config", "--show"]);
    assert!(show.status.success(), "{show:?}");
    assert!(
        String::from_utf8_lossy(&show.stdout)
            .contains("labels.triage.values = \"ready,in progress\" (project)"),
        "{}",
        String::from_utf8_lossy(&show.stdout)
    );

    let unset = cli(
        &fixture,
        &["config", "--unset", "push.enabled", "--project"],
    );
    assert!(unset.status.success(), "{unset:?}");
    let contents = std::fs::read_to_string(path).unwrap();
    assert!(!contents.contains("enabled = false"), "{contents}");
    assert!(contents.contains("single_select = true"), "{contents}");
}

#[test]
fn invalid_project_value_does_not_modify_existing_file() {
    let fixture = fixture();
    let path = fixture.repo.join(".tickets/config.toml");
    std::fs::write(&path, "[push]\nenabled = true\n").unwrap();
    let before = std::fs::read(&path).unwrap();
    let output = cli(
        &fixture,
        &["config", "--set", "push.enabled=treu", "--project"],
    );
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("push.enabled"), "{stderr}");
    assert!(stderr.contains("treu"), "{stderr}");
    assert_eq!(std::fs::read(path).unwrap(), before);
}

#[test]
fn default_set_scope_remains_user_and_help_explains_both_scopes() {
    let fixture = fixture();
    let dry_run = cli(
        &fixture,
        &[
            "--dry-run",
            "config",
            "--project",
            "--set",
            "push.enabled=false",
        ],
    );
    assert!(dry_run.status.success(), "{dry_run:?}");
    assert!(!fixture.repo.join(".tickets/config.toml").exists());

    let output = cli(&fixture, &["config", "--set", "debug.format=human"]);
    assert!(output.status.success(), "{output:?}");
    assert!(!fixture.repo.join(".tickets/config.toml").exists());
    let user_file = std::fs::read_to_string(fixture.user_config.join("tkt/config.toml")).unwrap();
    assert!(user_file.contains("format = \"human\""), "{user_file}");

    let help = cli(&fixture, &["config", "--help"]);
    assert!(help.status.success(), "{help:?}");
    let help = String::from_utf8_lossy(&help.stdout);
    assert!(help.contains("--project"), "{help}");
    assert!(help.contains("user config"), "{help}");
    assert!(help.contains(".tickets/config.toml"), "{help}");
    assert!(help.contains("not committed automatically"), "{help}");

    let invalid_scope = cli(&fixture, &["config", "--project", "--get", "debug"]);
    assert_eq!(invalid_scope.status.code(), Some(1), "{invalid_scope:?}");
}

#[test]
fn detached_head_project_write_warns_without_committing() {
    let fixture = fixture();
    std::fs::write(fixture.repo.join("README.md"), "fixture\n").unwrap();
    for (key, value) in [
        ("user.name", "Fixture"),
        ("user.email", "fixture@example.invalid"),
    ] {
        let configured = Command::new("git")
            .args(["config", key, value])
            .current_dir(&fixture.repo)
            .env("DO_NOT_TRACK", "1")
            .output()
            .unwrap();
        assert!(configured.status.success());
    }
    let commit = Command::new("git")
        .args(["add", "README.md"])
        .current_dir(&fixture.repo)
        .env("DO_NOT_TRACK", "1")
        .output()
        .unwrap();
    assert!(commit.status.success());
    let commit = Command::new("git")
        .args(["commit", "-qm", "fixture"])
        .current_dir(&fixture.repo)
        .env("DO_NOT_TRACK", "1")
        .output()
        .unwrap();
    assert!(commit.status.success());
    let detached = Command::new("git")
        .args(["checkout", "--detach", "-q", "HEAD"])
        .current_dir(&fixture.repo)
        .env("DO_NOT_TRACK", "1")
        .output()
        .unwrap();
    assert!(detached.status.success());

    let head = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&fixture.repo)
        .output()
        .unwrap();
    let head_before = String::from_utf8(head.stdout).unwrap();
    let output = cli(
        &fixture,
        &["config", "--project", "--set", "push.enabled=false"],
    );
    assert!(output.status.success(), "{output:?}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("detached HEAD"), "{stderr}");
    assert!(stderr.contains("not committed automatically"), "{stderr}");
    let head_after = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&fixture.repo)
        .output()
        .unwrap();
    assert_eq!(String::from_utf8(head_after.stdout).unwrap(), head_before);
}

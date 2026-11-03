use std::path::PathBuf;
use std::process::{Command, Output};

use tempfile::TempDir;

struct Fixture {
    _temp: TempDir,
    repo: PathBuf,
    user_config: PathBuf,
}

fn fixture(project_config: &str) -> Fixture {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");
    let user_config = temp.path().join("user-config");
    std::fs::create_dir(&repo).unwrap();
    std::fs::create_dir_all(repo.join(".tickets")).unwrap();
    std::fs::write(repo.join(".tickets/config.toml"), project_config).unwrap();
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
    cli_with_debug_overrides(fixture, args, None, None)
}

fn cli_with_debug_overrides(
    fixture: &Fixture,
    args: &[&str],
    debug: Option<&str>,
    debug_format: Option<&str>,
) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_tkt"));
    command
        .args(args)
        .current_dir(&fixture.repo)
        .env("DO_NOT_TRACK", "1")
        .env("TKT_NO_USER_CONFIG", "0")
        .env("XDG_CONFIG_HOME", &fixture.user_config)
        .env("TKT_UPDATE_CHECK", "0")
        .env_remove("TKT_DEBUG")
        .env_remove("TKT_DEBUG_FORMAT");
    if let Some(value) = debug {
        command.env("TKT_DEBUG", value);
    }
    if let Some(value) = debug_format {
        command.env("TKT_DEBUG_FORMAT", value);
    }
    command.output().unwrap()
}

fn assert_config_error(output: &Output, key: &str, value: &str, source: &str) {
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(text.contains(key), "{text}");
    assert!(text.contains(value), "{text}");
    assert!(text.contains(source), "{text}");
    assert!(
        text.contains("invalid configuration") || text.contains("invalid user configuration"),
        "{text}"
    );
}

#[test]
fn typoed_push_policy_cannot_allow_ticket_creation() {
    let fixture = fixture("[push]\nenabled = treu\n");
    let output = cli(&fixture, &["new", "would-have-been-created"]);
    assert_config_error(&output, "push.enabled", "treu", "project");
    assert!(!fixture
        .repo
        .join(".tickets/01-would-have-been-created.md")
        .exists());
}

#[test]
fn typoed_evidence_policy_cannot_weaken_close_gate() {
    let fixture = fixture(
        "[close]\nrequire_resolution = true\nrequire_checked_acs = true\nrequire_validation_evidence = treu\n",
    );
    let ticket = fixture.repo.join(".tickets/01-ready.md");
    std::fs::write(
        &ticket,
        "---\nid: \"01\"\ntitle: \"Ready\"\nstatus: open\nblocked_by: []\nvalidation_criteria:\n  - \"checked\"\n---\n\n## Acceptance criteria\n\n- [x] done\n",
    )
    .unwrap();
    let output = cli(
        &fixture,
        &["close", "01", "--resolution", "done", "--check-all"],
    );
    assert_config_error(
        &output,
        "close.require_validation_evidence",
        "treu",
        "project",
    );
    assert!(std::fs::read_to_string(ticket)
        .unwrap()
        .contains("status: open"));
}

#[test]
fn invalid_priority_and_environment_filters_fail_with_context() {
    let fixture = fixture("[new]\ndefault_priority = urgentish\n[ready]\ndefault_env = corpX\n");
    let output = cli(&fixture, &["new", "bad-priority"]);
    assert_config_error(&output, "new.default_priority", "urgentish", "project");
    assert!(!fixture.repo.join(".tickets/01-bad-priority.md").exists());

    let output = cli(&fixture, &["ready"]);
    assert_config_error(&output, "ready.default_env", "corpX", "project");
}

#[test]
fn invalid_audit_cutoff_fails_with_source_context() {
    let fixture = fixture("[audit]\nresolution_baseline_id = someday\n");
    let output = cli(&fixture, &["audit"]);
    assert_config_error(
        &output,
        "audit.resolution_baseline_id",
        "someday",
        "project",
    );
}

#[test]
fn unknown_keys_remain_warnings_and_project_values_override_user_values() {
    let fixture =
        fixture("[push]\nenabled = true\n[new]\ndefault_priority = urgent\n[mystery]\nfoo = bar\n");
    std::fs::create_dir_all(fixture.user_config.join("tkt")).unwrap();
    std::fs::write(
        fixture.user_config.join("tkt/config.toml"),
        "[push]\nenabled = treu\n[new]\ndefault_priority = low\n",
    )
    .unwrap();
    let output = cli(&fixture, &["config", "--show"]);
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stdout.contains("push.enabled = \"true\" (project)"));
    assert!(stdout.contains("new.default_priority = \"urgent\" (project)"));
    assert!(stderr.contains("warning: unknown config key \"mystery.foo\""));
}

#[test]
fn invalid_debug_config_is_reported_but_user_config_can_inspect_and_repair_it() {
    let fixture = fixture("");
    std::fs::create_dir_all(fixture.user_config.join("tkt")).unwrap();
    let user_config = fixture.user_config.join("tkt/config.toml");
    std::fs::write(&user_config, "debug = treu\ndebug.format = huma\n").unwrap();

    let output = cli(&fixture, &["ready"]);
    assert_config_error(&output, "debug", "treu", "user config");
    let text = String::from_utf8_lossy(&output.stderr);
    assert!(text.contains("debug.format"), "{text}");
    assert!(text.contains("huma"), "{text}");

    for args in [
        ["config", "--get", "debug.format"],
        ["config", "--list", ""],
    ] {
        let args = args
            .into_iter()
            .filter(|arg| !arg.is_empty())
            .collect::<Vec<_>>();
        let output = cli(&fixture, &args);
        assert!(output.status.success(), "{output:?}");
    }

    let set = cli(&fixture, &["config", "--set", "debug=false"]);
    assert!(set.status.success(), "{set:?}");
    let unset = cli(&fixture, &["config", "--unset", "debug.format"]);
    assert!(unset.status.success(), "{unset:?}");
    let repaired = cli(&fixture, &["ready"]);
    assert_eq!(repaired.status.code(), Some(0), "{repaired:?}");
}

#[test]
fn explicit_debug_environment_modes_override_stored_debug_settings() {
    let fixture = fixture("");
    std::fs::create_dir_all(fixture.user_config.join("tkt")).unwrap();
    std::fs::write(
        fixture.user_config.join("tkt/config.toml"),
        "debug = treu\ndebug.format = huma\n",
    )
    .unwrap();

    let debug_override = cli_with_debug_overrides(&fixture, &["ready"], Some("json"), None);
    assert_eq!(debug_override.status.code(), Some(0), "{debug_override:?}");
    let format_override = cli_with_debug_overrides(&fixture, &["ready"], None, Some("json"));
    assert_eq!(
        format_override.status.code(),
        Some(1),
        "{format_override:?}"
    );
    let stderr = String::from_utf8_lossy(&format_override.stderr);
    assert!(stderr.contains("debug"), "{stderr}");
    assert!(!stderr.contains("debug.format"), "{stderr}");
}

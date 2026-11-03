use std::path::Path;
use std::process::{Command, Output};

use tempfile::TempDir;

fn repo(config: Option<&str>) -> (TempDir, std::path::PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().to_path_buf();
    let tickets = root.join(".tickets");
    std::fs::create_dir(&tickets).unwrap();
    if let Some(config) = config {
        std::fs::write(tickets.join("config.toml"), config).unwrap();
    }
    let init = Command::new("git")
        .args(["init", "-q"])
        .current_dir(&root)
        .env("DO_NOT_TRACK", "1")
        .output()
        .unwrap();
    assert!(init.status.success());
    (temp, root)
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

#[test]
fn config_show_exposes_label_namespace_settings() {
    let (_temp, root) = repo(Some(
        "[labels.triage]\nsingle_select = true\nvalues = [\"needs-info\", \"ready-for-agent\"]\n",
    ));
    let output = cli(&root, &["config", "--show"]);
    assert!(output.status.success(), "{output:?}");
    let output = String::from_utf8(output.stdout).unwrap();
    assert!(output.contains("labels.triage.single_select = \"true\" (project)"));
    assert!(output.contains("labels.triage.values = \"needs-info,ready-for-agent\" (project)"));
}

#[test]
fn config_without_label_namespaces_remains_valid() {
    let (_temp, root) = repo(Some("[push]\nenabled = false\n"));
    let output = cli(&root, &["config", "--show"]);
    assert!(output.status.success(), "{output:?}");
    let output = String::from_utf8(output.stdout).unwrap();
    assert!(output.contains("push.enabled = \"false\" (project)"));
    assert!(!output.contains("labels."));
}

#[test]
fn malformed_and_unknown_label_config_returns_a_clear_error() {
    for (config, detail) in [
        ("[labels.triage]\nsingle_select = maybe\n", "single_select"),
        ("[labels.triage]\nvalues = [\"ready\", 4]\n", "values"),
        ("[labels.triage]\nvalues = ['ready']\n", "values"),
        ("[labels.triage]\nvalues = [\"ready\",]\n", "values"),
        ("[labels.triage]\nunknown = true\n", "unknown"),
        ("[labels.triage.extra]\nvalues = [\"ready\"]\n", "malformed"),
    ] {
        let (_temp, root) = repo(Some(config));
        let output = cli(&root, &["-o", "json", "config", "--show"]);
        assert_eq!(output.status.code(), Some(1), "{output:?}");
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(text.contains("label namespace"), "{text}");
        assert!(text.contains("labels.triage"), "{text}");
        assert!(text.contains(detail), "{text}");

        let doctor = cli(&root, &["doctor"]);
        assert_eq!(doctor.status.code(), Some(1), "{doctor:?}");
        let doctor = format!(
            "{}{}",
            String::from_utf8_lossy(&doctor.stdout),
            String::from_utf8_lossy(&doctor.stderr)
        );
        assert!(doctor.contains("1 validation errors"), "{doctor}");
        let validate = cli(&root, &["validate", "--brief"]);
        assert_eq!(validate.status.code(), Some(1), "{validate:?}");
        let validate = format!(
            "{}{}",
            String::from_utf8_lossy(&validate.stdout),
            String::from_utf8_lossy(&validate.stderr)
        );
        assert!(
            validate.contains("invalid .tickets/config.toml label namespace configuration"),
            "{validate}"
        );
    }
}

#[test]
fn mutation_rejects_invalid_label_config_without_changing_ticket_or_git_state() {
    let (_temp, root) = repo(Some("[labels.triage]\nsingle_select = maybe\n"));
    let ticket_path = root.join(".tickets/01-example.md");
    std::fs::write(
        &ticket_path,
        "---\nid: \"01\"\ntitle: \"Example\"\nstatus: open\nblocked_by: []\n---\n",
    )
    .unwrap();
    for (key, value) in [
        ("user.email", "test@example.invalid"),
        ("user.name", "Test"),
    ] {
        let output = Command::new("git")
            .args(["config", key, value])
            .current_dir(&root)
            .env("DO_NOT_TRACK", "1")
            .output()
            .unwrap();
        assert!(output.status.success());
    }
    let add = Command::new("git")
        .args(["add", "."])
        .current_dir(&root)
        .env("DO_NOT_TRACK", "1")
        .output()
        .unwrap();
    assert!(add.status.success());
    let commit = Command::new("git")
        .args(["commit", "-qm", "seed"])
        .current_dir(&root)
        .env("DO_NOT_TRACK", "1")
        .output()
        .unwrap();
    assert!(commit.status.success());

    let before_ticket = std::fs::read(&ticket_path).unwrap();
    let before_head = git(&root, &["rev-parse", "HEAD"]);
    let before_status = git(&root, &["status", "--porcelain"]);
    let output = cli(&root, &["edit", "01", "--title", "Changed"]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let error = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(error.contains("label namespace configuration"), "{error}");
    assert_eq!(std::fs::read(ticket_path).unwrap(), before_ticket);
    assert_eq!(git(&root, &["rev-parse", "HEAD"]), before_head);
    assert_eq!(git(&root, &["status", "--porcelain"]), before_status);
}

use std::path::Path;
use std::process::{Command, Output};

use serde_json::Value;
use tempfile::TempDir;

fn repo() -> (TempDir, std::path::PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().to_path_buf();
    let git = Command::new("git")
        .args(["init", "-q"])
        .current_dir(&root)
        .env("DO_NOT_TRACK", "1")
        .output()
        .unwrap();
    assert!(git.status.success(), "{git:?}");
    std::fs::create_dir(root.join(".tickets")).unwrap();
    (temp, root)
}

fn write_ticket(root: &Path, status: &str, body: &str) {
    std::fs::write(
        root.join(".tickets/01-fixture.md"),
        format!(
            "---\nid: \"01\"\ntitle: \"Fixture\"\nstatus: {status}\nblocked_by: []\n---\n\n{body}"
        ),
    )
    .unwrap();
}

fn audit(root: &Path, extra: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tkt"))
        .arg("audit")
        .args(extra)
        .current_dir(root)
        .env("DO_NOT_TRACK", "1")
        .env("TKT_NO_USER_CONFIG", "1")
        .env("TKT_UPDATE_CHECK", "0")
        .env_remove("TKT_DEBUG")
        .output()
        .unwrap()
}

fn report(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "invalid audit JSON: {error}; stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

#[test]
fn clean_audit_retains_pass_and_reports_zero_counts() {
    let (_temp, root) = repo();
    write_ticket(
        &root,
        "open",
        "# Real work\n\n## What to build\n\nA substantive outcome.\n",
    );
    let output = audit(&root, &[]);
    assert!(output.status.success(), "{output:?}");
    let report = report(&output);
    assert_eq!(report["status"], "pass");
    assert_eq!(report["outcome"], "pass");
    assert_eq!(
        report["counts"],
        serde_json::json!({"errors": 0, "warnings": 0, "info": 0})
    );
    assert_eq!(report["findings"].as_array().unwrap().len(), 0);
    let brief = audit(&root, &["--brief"]);
    assert!(brief.status.success(), "{brief:?}");
    assert!(String::from_utf8_lossy(&brief.stdout).contains("pass (0 finding(s))"));
}

#[test]
fn warning_only_audit_is_explicit_without_changing_compatibility() {
    let (_temp, root) = repo();
    write_ticket(
        &root,
        "done",
        "# Incomplete closure\n\n## Acceptance criteria\n\n- [ ] Unchecked work\n",
    );
    let output = audit(&root, &[]);
    assert!(output.status.success(), "{output:?}");
    let report = report(&output);
    assert_eq!(report["status"], "pass");
    assert_eq!(report["outcome"], "pass_with_warnings");
    assert_eq!(report["counts"]["errors"], 0);
    assert!(report["counts"]["warnings"].as_u64().unwrap() >= 2);
    assert_eq!(report["counts"]["info"], 0);
    assert!(report["findings"].as_array().unwrap().len() >= 2);

    let brief = audit(&root, &["--brief"]);
    assert!(brief.status.success(), "{brief:?}");
    let stdout = String::from_utf8_lossy(&brief.stdout);
    assert!(stdout.contains("pass with warnings ("), "{stdout}");
}

#[test]
fn strict_warnings_keep_failure_status_and_exit_code() {
    let (_temp, root) = repo();
    write_ticket(
        &root,
        "done",
        "# Incomplete closure\n\n## Acceptance criteria\n\n- [ ] Unchecked work\n",
    );
    let output = audit(&root, &["--strict"]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let report = report(&output);
    assert_eq!(report["status"], "fail");
    assert_eq!(report["outcome"], "fail");
    assert_eq!(report["counts"]["errors"], 0);
    assert!(report["counts"]["warnings"].as_u64().unwrap() >= 2);
}

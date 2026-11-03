use std::path::Path;
use std::process::Command;

fn cli(dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_tkt"))
        .current_dir(dir)
        .args(args)
        .env("DO_NOT_TRACK", "1")
        .env("TKT_NO_USER_CONFIG", "1")
        .env("TKT_UPDATE_CHECK", "0")
        .env_remove("TKT_DEBUG")
        .output()
        .unwrap()
}

fn repo(config: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let init = Command::new("git")
        .args(["init", "-q"])
        .env("DO_NOT_TRACK", "1")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(init.status.success());
    let tickets = dir.path().join(".tickets");
    std::fs::create_dir(&tickets).unwrap();
    std::fs::write(tickets.join("config.toml"), config).unwrap();
    dir
}

fn ticket(tags: &[&str]) -> String {
    let tags = tags
        .iter()
        .map(|tag| format!("\"{tag}\""))
        .collect::<Vec<_>>()
        .join(", ");
    format!("---\nid: \"01\"\ntitle: Ticket\nstatus: open\nblocked_by: []\ntags: [{tags}]\n---\n")
}

fn findings(dir: &Path) -> serde_json::Value {
    let output = cli(dir, &["validate"]);
    assert!(
        output.status.success() || output.status.code() == Some(1),
        "{output:?}"
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn has_rule(report: &serde_json::Value, rule: &str) -> bool {
    report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|finding| finding["rule"] == rule)
}

fn write_ticket(dir: &Path, tags: &[&str]) {
    std::fs::write(dir.join(".tickets/01-ticket.md"), ticket(tags)).unwrap();
}

#[test]
fn closed_namespace_rejects_undeclared_values() {
    let dir = repo("[labels.triage]\nsingle_select = true\nvalues = [\"ready\", \"blocked\"]\n");
    write_ticket(dir.path(), &["triage:unknown"]);

    let report = findings(dir.path());
    assert!(has_rule(&report, "label-namespace-value"), "{report}");
}

#[test]
fn single_select_namespace_rejects_multiple_values() {
    let dir = repo("[labels.triage]\nsingle_select = true\nvalues = [\"ready\", \"blocked\"]\n");
    write_ticket(dir.path(), &["triage:ready", "triage:blocked"]);

    let report = findings(dir.path());
    assert!(
        has_rule(&report, "label-namespace-single-select"),
        "{report}"
    );
}

#[test]
fn undeclared_tags_and_multi_select_axes_remain_free() {
    let dir = repo(
        "[labels.triage]\nsingle_select = true\nvalues = [\"ready\", \"blocked\"]\n[labels.stream]\nsingle_select = false\nvalues = [\"quality\", \"config\"]\n",
    );
    write_ticket(
        dir.path(),
        &[
            "triage:ready",
            "stream:quality",
            "stream:config",
            "workstream:planning",
            "custom-label",
        ],
    );

    let report = findings(dir.path());
    assert_eq!(report["status"], "pass", "{report}");
    assert!(!has_rule(&report, "label-namespace-value"), "{report}");
    assert!(
        !has_rule(&report, "label-namespace-single-select"),
        "{report}"
    );
}

#[test]
fn empty_closed_set_rejects_any_namespaced_value() {
    let dir = repo("[labels.triage]\nsingle_select = true\nvalues = []\n");
    write_ticket(dir.path(), &["triage:ready"]);

    let report = findings(dir.path());
    assert!(has_rule(&report, "label-namespace-value"), "{report}");
}

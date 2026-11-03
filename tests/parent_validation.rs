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

fn repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let init = Command::new("git")
        .args(["init", "-q"])
        .env("DO_NOT_TRACK", "1")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(init.status.success());
    std::fs::create_dir(dir.path().join(".tickets")).unwrap();
    dir
}

fn ticket(id: &str, parent: Option<&str>, blocked_by: &[&str]) -> String {
    let parent = parent.map_or_else(String::new, |parent| format!("parent: \"{parent}\"\n"));
    let blocked_by = blocked_by
        .iter()
        .map(|id| format!("\"{id}\""))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "---\nid: \"{id}\"\ntitle: Ticket {id}\nstatus: open\n{parent}blocked_by: [{blocked_by}]\n---\n"
    )
}

fn findings(output: &std::process::Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap()
}

fn has_rule(report: &serde_json::Value, rule: &str) -> bool {
    report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|finding| finding["rule"] == rule)
}

#[test]
fn validate_flags_dangling_parent() {
    let dir = repo();
    std::fs::write(
        dir.path().join(".tickets/01-child.md"),
        ticket("01", Some("99"), &[]),
    )
    .unwrap();

    let report = findings(&cli(dir.path(), &["validate"]));
    assert!(has_rule(&report, "dangling-parent"), "{report}");
}

#[test]
fn validate_flags_self_parent() {
    let dir = repo();
    std::fs::write(
        dir.path().join(".tickets/01-self.md"),
        ticket("01", Some("01"), &[]),
    )
    .unwrap();

    let report = findings(&cli(dir.path(), &["validate"]));
    assert!(has_rule(&report, "self-parent"), "{report}");
}

#[test]
fn validate_flags_parent_chain_cycle() {
    let dir = repo();
    std::fs::write(
        dir.path().join(".tickets/01-a.md"),
        ticket("01", Some("02"), &[]),
    )
    .unwrap();
    std::fs::write(
        dir.path().join(".tickets/02-b.md"),
        ticket("02", Some("01"), &[]),
    )
    .unwrap();

    let report = findings(&cli(dir.path(), &["validate"]));
    assert!(has_rule(&report, "parent-cycle"), "{report}");
}

#[test]
fn parent_and_blocked_by_edges_are_validated_independently() {
    let dir = repo();
    std::fs::write(
        dir.path().join(".tickets/01-a.md"),
        ticket("01", Some("02"), &[]),
    )
    .unwrap();
    std::fs::write(
        dir.path().join(".tickets/02-b.md"),
        ticket("02", None, &["01"]),
    )
    .unwrap();

    let report = findings(&cli(dir.path(), &["validate"]));
    assert!(!has_rule(&report, "parent-cycle"), "{report}");
    assert!(!has_rule(&report, "cycle"), "{report}");
}

#[test]
fn blocked_by_cycle_detection_remains_independent_of_parent_chains() {
    let dir = repo();
    std::fs::write(
        dir.path().join(".tickets/01-a.md"),
        ticket("01", Some("02"), &["02"]),
    )
    .unwrap();
    std::fs::write(
        dir.path().join(".tickets/02-b.md"),
        ticket("02", None, &["01"]),
    )
    .unwrap();

    let report = findings(&cli(dir.path(), &["validate"]));
    assert!(has_rule(&report, "cycle"), "{report}");
    assert!(!has_rule(&report, "parent-cycle"), "{report}");
}

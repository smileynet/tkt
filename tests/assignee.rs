use serde_json::Value;
use std::path::Path;
use std::process::{Command, Output};

fn run(dir: &Path, args: &[&str], agent: Option<&str>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_tkt"));
    command
        .current_dir(dir)
        .args(args)
        .env("DO_NOT_TRACK", "1")
        .env("TKT_DEBUG", "0")
        .env_remove("TKT_DEBUG_FORMAT")
        .env("TKT_NO_USER_CONFIG", "1")
        .env_remove("TKT_AGENT")
        .env_remove("CREW_ENV");
    if let Some(agent) = agent {
        command.env("TKT_AGENT", agent);
    }
    command.output().unwrap()
}

fn ok(dir: &Path, args: &[&str], agent: Option<&str>) -> String {
    let output = run(dir, args, agent);
    assert!(
        output.status.success(),
        "{:?}: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

fn repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for args in [
        vec!["init", "-q"],
        vec!["config", "user.name", "test"],
        vec!["config", "user.email", "test@test"],
    ] {
        assert!(Command::new("git")
            .current_dir(dir.path())
            .args(args)
            .status()
            .unwrap()
            .success());
    }
    std::fs::create_dir(dir.path().join(".tickets")).unwrap();
    std::fs::write(
        dir.path().join(".tickets/config.toml"),
        "[push]\nenabled = false\n",
    )
    .unwrap();
    assert!(Command::new("git")
        .current_dir(dir.path())
        .args(["add", "."])
        .status()
        .unwrap()
        .success());
    assert!(Command::new("git")
        .current_dir(dir.path())
        .args(["commit", "-qm", "seed"])
        .status()
        .unwrap()
        .success());
    dir
}

fn ticket(dir: &Path, id: &str, status: &str, assignee: Option<&str>) {
    let assignee = assignee
        .map(|agent| format!("assignee: \"{agent}\"\n"))
        .unwrap_or_default();
    std::fs::write(
        dir.join(format!(".tickets/{id}-test.md")),
        format!(
            "---\nid: \"{id}\"\ntitle: \"Ticket {id}\"\nstatus: {status}\nblocked_by: []\n{assignee}---\n\nBody.\n"
        ),
    )
    .unwrap();
}

fn rows(output: &[u8]) -> Vec<Value> {
    String::from_utf8_lossy(output)
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

#[test]
fn claim_assigns_identity_and_rejects_other_owner_unless_forced() {
    let dir = repo();
    let path = dir.path();
    ticket(path, "01", "open", None);
    ticket(path, "02", "open", Some("codex"));

    let failed = run(path, &["--agent", "claude", "claim", "02"], None);
    assert_eq!(failed.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&failed.stderr).contains("assigned to codex"));
    let unchanged = std::fs::read_to_string(path.join(".tickets/02-test.md")).unwrap();
    assert!(unchanged.contains("status: open"));
    assert!(unchanged.contains("assignee: \"codex\""));

    ok(path, &["--agent", "codex", "claim", "01"], None);
    let claimed = std::fs::read_to_string(path.join(".tickets/01-test.md")).unwrap();
    assert!(claimed.contains("status: in_progress"));
    assert!(claimed.contains("assignee: \"codex\""));

    ok(path, &["--agent", "claude", "claim", "02", "--force"], None);
    let reassigned = std::fs::read_to_string(path.join(".tickets/02-test.md")).unwrap();
    assert!(reassigned.contains("status: in_progress"));
    assert!(reassigned.contains("assignee: \"claude\""));
}

#[test]
fn ready_and_query_expose_assignee_and_ready_filters_other_owners() {
    let dir = repo();
    let path = dir.path();
    ticket(path, "01", "open", None);
    ticket(path, "02", "open", Some("codex"));
    ticket(path, "03", "in_progress", Some("codex"));

    let anonymous_ready = rows(&run(path, &["ready", "--json"], None).stdout);
    assert_eq!(anonymous_ready.len(), 1);
    assert_eq!(anonymous_ready[0]["id"], "01");
    assert_eq!(anonymous_ready[0]["assignee"], Value::Null);

    let codex_ready = rows(&run(path, &["ready", "--json"], Some("codex")).stdout);
    assert_eq!(codex_ready.len(), 2);
    assert_eq!(codex_ready[1]["assignee"], "codex");

    let query = rows(&run(path, &["query"], None).stdout);
    assert_eq!(query[0]["assignee"], Value::Null);
    assert_eq!(query[1]["assignee"], "codex");
    assert_eq!(query[2]["assignee"], "codex");

    let manifest: Value = serde_json::from_str(&ok(path, &["capabilities"], None)).unwrap();
    assert!(manifest["commands"]["ready"]["output"]["description"]
        .as_str()
        .unwrap()
        .contains("assigned to another agent"));
    assert!(manifest["commands"]["query"]["output"]["description"]
        .as_str()
        .unwrap()
        .contains("assignee"));
}

#[test]
fn malformed_assignee_is_rejected_by_ticket_parser() {
    let dir = repo();
    let path = dir.path();
    ticket(path, "01", "open", Some("Invalid_ID"));

    let output = run(path, &["query"], None);
    assert!(rows(&output.stdout).is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("invalid agent id"));
}

#[test]
fn missing_identity_cannot_take_another_agents_assigned_ticket() {
    let dir = repo();
    let path = dir.path();
    ticket(path, "01", "open", Some("codex"));

    let ready = rows(&run(path, &["ready", "--json"], None).stdout);
    assert!(ready.is_empty());
    let failed = run(path, &["claim", "01"], None);
    assert_eq!(failed.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&failed.stderr).contains("assigned to codex"));

    ok(path, &["claim", "01", "--force"], None);
    let claimed = std::fs::read_to_string(path.join(".tickets/01-test.md")).unwrap();
    assert!(claimed.contains("status: in_progress"));
    assert!(claimed.contains("assignee: \"codex\""));
}

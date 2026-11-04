use std::path::Path;
use std::process::{Command, Output};

fn repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "-q"]);
    std::fs::create_dir(dir.path().join(".tickets")).unwrap();
    dir
}

fn git(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(["-C", root.to_str().unwrap()])
        .args(args)
        .env("DO_NOT_TRACK", "1")
        .env("TKT_DEBUG", "0")
        .env_remove("TKT_DEBUG_FORMAT")
        .output()
        .unwrap();
    assert!(output.status.success(), "git {args:?}: {output:?}");
}

fn cli(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tkt"))
        .args(args)
        .current_dir(root)
        .env("DO_NOT_TRACK", "1")
        .env("TKT_DEBUG", "0")
        .env_remove("TKT_DEBUG_FORMAT")
        .env("TKT_NO_USER_CONFIG", "1")
        .env("TKT_UPDATE_CHECK", "0")
        .env_remove("TKT_CONTEXT")
        .output()
        .unwrap()
}

fn write_ticket(root: &Path, id: &str, status: &str, dependencies: &[&str]) {
    let filename = format!("{id}-ticket.md");
    let blocked_by = dependencies
        .iter()
        .map(|id| format!("\"{id}\""))
        .collect::<Vec<_>>()
        .join(", ");
    std::fs::write(
        root.join(".tickets").join(filename),
        format!(
            "---\nid: \"{id}\"\ntitle: Ticket {id}\nstatus: {status}\nblocked_by: [{blocked_by}]\n---\n"
        ),
    )
    .unwrap();
}

fn rows(output: &Output) -> Vec<serde_json::Value> {
    assert!(output.status.success(), "{output:?}");
    String::from_utf8(output.stdout.clone())
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

#[test]
fn query_and_ready_distinguish_historical_edges_from_pending_blockers() {
    let dir = repo();
    write_ticket(dir.path(), "01", "done", &[]);
    write_ticket(dir.path(), "02", "open", &["01", "03", "04", "99"]);
    write_ticket(dir.path(), "03", "in_progress", &[]);
    write_ticket(dir.path(), "04", "cancelled", &[]);
    write_ticket(dir.path(), "05", "open", &["01"]);
    write_ticket(dir.path(), "06", "open", &["99"]);

    let query = rows(&cli(dir.path(), &["query", "-o", "json"]));
    let row_02 = query.iter().find(|row| row["id"] == "02").unwrap();
    assert_eq!(
        row_02["blocked_by"],
        serde_json::json!(["01", "03", "04", "99"])
    );
    assert_eq!(
        row_02["pending_blockers"],
        serde_json::json!(["03", "04", "99"])
    );
    assert_eq!(
        query.iter().find(|row| row["id"] == "05").unwrap()["pending_blockers"],
        serde_json::json!([])
    );
    assert_eq!(
        query.iter().find(|row| row["id"] == "06").unwrap()["pending_blockers"],
        serde_json::json!(["99"])
    );

    let ready = rows(&cli(dir.path(), &["ready", "--json"]));
    assert_eq!(ready.len(), 1);
    assert_eq!(ready[0]["id"], "05");
    assert_eq!(ready[0]["blocked_by"], serde_json::json!(["01"]));
    assert_eq!(ready[0]["pending_blockers"], serde_json::json!([]));

    let blocked = cli(dir.path(), &["blocked"]);
    assert!(blocked.status.success(), "{blocked:?}");
    let human = String::from_utf8(blocked.stdout).unwrap();
    assert!(!human.contains("blocked by: 01"), "{human}");
    assert!(
        human.contains("blocked by: 03 Ticket 03 (in_progress)"),
        "{human}"
    );
    assert!(
        human.contains("blocked by: 04 Ticket 04 (cancelled)"),
        "{human}"
    );
    assert!(human.contains("blocked by: 99 (not found)"), "{human}");
}

use std::path::Path;
use std::process::{Command, Output};

fn cli(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tkt"))
        .args(args)
        .current_dir(root)
        .env("DO_NOT_TRACK", "1")
        .env("TKT_NO_USER_CONFIG", "1")
        .env("TKT_UPDATE_CHECK", "0")
        .env("TKT_ASCII", "1")
        .env_remove("TKT_DEBUG")
        .output()
        .unwrap()
}

#[test]
fn skipped_tickets_have_structured_or_ascii_diagnostics() {
    let dir = tempfile::tempdir().unwrap();
    let init = Command::new("git")
        .args(["init", "-q"])
        .current_dir(dir.path())
        .env("DO_NOT_TRACK", "1")
        .output()
        .unwrap();
    assert!(init.status.success());
    std::fs::create_dir(dir.path().join(".tickets")).unwrap();
    let empty = cli(dir.path(), &["query", "-o", "json"]);
    assert!(empty.status.success());
    assert!(empty.stdout.is_empty());
    assert!(empty.stderr.is_empty());
    std::fs::write(
        dir.path().join(".tickets/01-invalid.md"),
        "---\ntitle: Missing ID\nstatus: open\n---\n",
    )
    .unwrap();
    let skipped = cli(dir.path(), &["query", "-o", "json"]);
    assert!(skipped.status.success());
    assert!(skipped.stdout.is_empty());
    let warning: serde_json::Value = serde_json::from_slice(&skipped.stderr).unwrap();
    assert_eq!(warning["warning"]["kind"], "unparseable_ticket");
    assert!(warning["warning"]["path"]
        .as_str()
        .unwrap()
        .ends_with("01-invalid.md"));
    assert!(warning["warning"]["message"]
        .as_str()
        .unwrap()
        .contains("missing required field: id"));
    let human = cli(dir.path(), &["query"]);
    assert!(human.status.success());
    let stderr = String::from_utf8(human.stderr).unwrap();
    assert!(stderr.starts_with("[warn] skipping"), "{stderr}");
    assert!(stderr.is_ascii(), "{stderr}");
    std::fs::write(
        dir.path().join(".tickets/02-valid.md"),
        "---\nid: \"02\"\ntitle: Valid\nstatus: open\nblocked_by: []\n---\n",
    )
    .unwrap();
    let mixed = cli(dir.path(), &["query", "-o", "json"]);
    assert!(mixed.status.success());
    let row: serde_json::Value = serde_json::from_slice(&mixed.stdout).unwrap();
    assert_eq!(row["id"], "02");
    assert!(serde_json::from_slice::<serde_json::Value>(&mixed.stderr).is_ok());
}

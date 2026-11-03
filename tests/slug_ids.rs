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

fn ticket(id: &str) -> String {
    format!("---\nid: \"{id}\"\ntitle: Ticket {id}\nstatus: open\nblocked_by: []\n---\n")
}

fn findings(output: &std::process::Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn slug_id_canonical() {
    let dir = repo();
    std::fs::write(dir.path().join(".tickets/cr-axcq.md"), ticket("cr-axcq")).unwrap();

    let output = cli(dir.path(), &["validate"]);
    let report = findings(&output);
    assert!(
        !report["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding["rule"] == "id-filename-mismatch"),
        "{report}"
    );
}

#[test]
fn numeric_id_mismatch_still_fails_validation() {
    let dir = repo();
    std::fs::write(dir.path().join(".tickets/07-foo.md"), ticket("04")).unwrap();

    let output = cli(dir.path(), &["validate"]);
    let report = findings(&output);
    assert_eq!(report["status"], "fail");
    assert!(report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|finding| finding["rule"] == "id-filename-mismatch"));
}

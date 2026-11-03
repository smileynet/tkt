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

fn missing_id_ticket() -> &'static str {
    "---\ntitle: A hand-authored ticket\nstatus: open\nblocked_by: []\ncustom: keep me\n---\n\nTicket body.\n"
}

#[test]
fn backfill_missing_id() {
    let dir = repo();
    let path = dir.path().join(".tickets/007-hand-authored.md");
    std::fs::write(&path, missing_id_ticket()).unwrap();

    let lint = cli(dir.path(), &["lint"]);
    assert!(
        lint.status.success(),
        "{}",
        String::from_utf8_lossy(&lint.stderr)
    );
    let repaired = std::fs::read_to_string(&path).unwrap();
    assert!(
        repaired.starts_with("---\nid: \"007\"\ntitle:"),
        "{repaired}"
    );
    assert!(repaired.contains("custom: keep me"));
    assert!(repaired.ends_with("\n\nTicket body.\n"));

    let canonical = cli(dir.path(), &["lint", "--check"]);
    assert!(
        canonical.status.success(),
        "{}",
        String::from_utf8_lossy(&canonical.stderr)
    );
    let validate = cli(dir.path(), &["validate"]);
    assert!(
        validate.status.success(),
        "{}",
        String::from_utf8_lossy(&validate.stderr)
    );
    assert!(!String::from_utf8_lossy(&validate.stdout).contains("missing required field: id"));
}

#[test]
fn check_and_dry_run_report_backfill_without_writing() {
    for args in [&["lint", "--check"][..], &["lint", "--dry-run"][..]] {
        let dir = repo();
        let path = dir.path().join(".tickets/08-no-id.md");
        let original = missing_id_ticket();
        std::fs::write(&path, original).unwrap();

        let output = cli(dir.path(), args);
        assert!(output.status.success() || args.contains(&"--check"));
        let message = String::from_utf8_lossy(&output.stderr);
        assert!(message.contains("would change"), "{message}");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
    }
}

#[test]
fn existing_id_is_preserved() {
    let dir = repo();
    let path = dir.path().join(".tickets/09-existing.md");
    let original = "---\nid: \"custom-id\"\ntitle: Existing\nstatus: open\nblocked_by: []\n---\n";
    std::fs::write(&path, original).unwrap();

    let output = cli(dir.path(), &["lint"]);
    assert!(output.status.success());
    assert!(std::fs::read_to_string(path)
        .unwrap()
        .contains("id: \"custom-id\""));
}

#[test]
fn malformed_filename_and_invalid_ticket_are_not_repaired() {
    let dir = repo();
    let malformed = dir.path().join(".tickets/not-a-ticket.md");
    let invalid = dir.path().join(".tickets/10-invalid-status.md");
    let invalid_content = missing_id_ticket().replace("status: open", "status: unknown");
    std::fs::write(&malformed, missing_id_ticket()).unwrap();
    std::fs::write(&invalid, &invalid_content).unwrap();

    let output = cli(dir.path(), &["lint"]);
    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("skip"), "{stderr}");
    assert_eq!(
        std::fs::read_to_string(malformed).unwrap(),
        missing_id_ticket()
    );
    assert_eq!(std::fs::read_to_string(invalid).unwrap(), invalid_content);
}

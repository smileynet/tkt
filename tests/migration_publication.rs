use std::path::Path;
use std::process::Command;
fn cli(root: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_tkt"))
        .current_dir(root)
        .args(args)
        .env("DO_NOT_TRACK", "1")
        .env("TKT_NO_USER_CONFIG", "1")
        .env("TKT_UPDATE_CHECK", "0")
        .env_remove("TKT_DEBUG")
        .output()
        .unwrap()
}
fn fixture() -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    assert!(Command::new("git")
        .args(["init", "-q"])
        .arg(temp.path())
        .env("DO_NOT_TRACK", "1")
        .output()
        .unwrap()
        .status
        .success());
    std::fs::create_dir(temp.path().join(".tickets")).unwrap();
    std::fs::write(
        temp.path().join(".tickets/auth.md"),
        "---\nstatus: open\ndeps: []\n---\n\n# Auth\n\nKeep this original specification.\n",
    )
    .unwrap();
    temp
}
#[test]
fn preview_then_confirm_preserves_complete_backup_and_conversion() {
    let temp = fixture();
    let root = temp.path();
    let source = root.join(".tickets/auth.md");
    let original = std::fs::read(&source).unwrap();
    let preview = cli(root, &["migrate", "--from", "tk", "--dry-run"]);
    assert!(preview.status.success(), "{preview:?}");
    assert_eq!(std::fs::read(&source).unwrap(), original);
    assert!(!root.join(".tickets.bak").exists());
    assert!(!root.join(".tickets/01-auth.md").exists());
    let applied = cli(root, &["migrate", "--from", "tk", "--confirm"]);
    assert!(applied.status.success(), "{applied:?}");
    assert!(!source.exists());
    assert_eq!(
        std::fs::read(root.join(".tickets.bak/auth.md")).unwrap(),
        original
    );
    let converted = std::fs::read_to_string(root.join(".tickets/01-auth.md")).unwrap();
    assert!(
        converted.contains("id: \"01\"") && converted.contains("Keep this original specification."),
        "{converted}"
    );
}
#[test]
fn preview_and_apply_both_preserve_a_conflicting_backup() {
    let temp = fixture();
    let root = temp.path();
    let source = root.join(".tickets/auth.md");
    let original = std::fs::read(&source).unwrap();
    let backup = root.join(".tickets.bak/auth.md");
    std::fs::create_dir(backup.parent().unwrap()).unwrap();
    std::fs::write(&backup, "older human backup").unwrap();
    for mode in ["--dry-run", "--confirm"] {
        let out = cli(root, &["-q", "-o", "json", "migrate", "--from", "tk", mode]);
        assert_eq!(out.status.code(), Some(1), "{out:?}");
        let stderr = String::from_utf8(out.stderr).unwrap();
        let error: serde_json::Value =
            serde_json::from_str(stderr.lines().last().unwrap()).unwrap();
        assert_eq!(error["error"]["kind"], "conflict");
        assert_eq!(std::fs::read(&source).unwrap(), original);
        assert_eq!(
            std::fs::read_to_string(&backup).unwrap(),
            "older human backup"
        );
        assert!(!root.join(".tickets/01-auth.md").exists());
    }
}

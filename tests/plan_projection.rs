use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn cli(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tkt"))
        .args(args)
        .current_dir(root)
        .env("DO_NOT_TRACK", "1")
        .env("TKT_DEBUG", "0")
        .env_remove("TKT_DEBUG_FORMAT")
        .env("TKT_NO_USER_CONFIG", "1")
        .env("TKT_UPDATE_CHECK", "0")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env(
            "GIT_CONFIG_GLOBAL",
            if cfg!(windows) { "NUL" } else { "/dev/null" },
        )
        .output()
        .unwrap()
}
fn fixture() -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    let out = Command::new("git")
        .args(["init", "-q"])
        .arg(temp.path())
        .env("DO_NOT_TRACK", "1")
        .env("TKT_DEBUG", "0")
        .env_remove("TKT_DEBUG_FORMAT")
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    std::fs::create_dir(temp.path().join(".tickets")).unwrap();
    for (id, status, priority) in [
        ("01", "done", "medium"),
        ("02", "backlog", "urgent"),
        ("03", "cancelled", "low"),
        ("04", "open", "high"),
        ("05", "in_progress", "medium"),
        ("cr-axcq", "open", "low"),
    ] {
        let title = if id == "04" {
            "Pipe | <tag> [link]"
        } else {
            id
        };
        std::fs::write(temp.path().join(format!(".tickets/{id}-case.md")), format!("---\nid: \"{id}\"\ntitle: \"{title}\"\nstatus: {status}\npriority: {priority}\nblocked_by: []\ntags: [\"one\"]\n---\n\n## Cancellation\n\n- Reason: obsolete\n\nRetired only for the cancelled fixture.\n")).unwrap();
    }
    temp
}
fn json(root: &Path, args: &[&str]) -> serde_json::Value {
    let mut full = vec!["-q", "-o", "json"];
    full.extend_from_slice(args);
    let out = cli(root, &full);
    assert!(out.status.success(), "{args:?}: {out:?}");
    serde_json::from_slice(&out.stdout).unwrap()
}
fn output_path(root: &Path) -> PathBuf {
    root.join("projection.md")
}

#[test]
fn whole_corpus_projection_is_deterministic_and_table_compatible() {
    let temp = fixture();
    let root = temp.path();
    assert!(cli(root, &["context", "unrelated"]).status.success());
    let first = cli(root, &["sync-plan", "--generate"]);
    assert!(first.status.success(), "{first:?}");
    let second = cli(root, &["sync-plan", "--generate"]);
    assert_eq!(first.stdout, second.stdout);
    assert!(!root.join("docs/plan.md").exists());
    let text = String::from_utf8(first.stdout).unwrap();
    let ids: Vec<_> = text
        .lines()
        .filter(|line| line.starts_with("| ") && !line.starts_with("| ID "))
        .map(|line| line.split('|').nth(1).unwrap().trim())
        .collect();
    assert_eq!(ids, ["02", "04", "01", "05", "03", "cr-axcq"]);
    assert!(
        text.contains("Pipe &#124; &lt;tag&gt; &#91;link&#93;"),
        "{text}"
    );
    assert!(text.contains("cancelled (obsolete)"));
    let report = json(root, &["sync-plan", "--generate"]);
    assert_eq!(report["tickets"], 6);
    assert_eq!(report["projection"], text);
    let path = output_path(root);
    assert_eq!(
        json(root, &["sync-plan", "--generate", path.to_str().unwrap()])["changed"],
        true
    );
    let (before, after) = (
        std::fs::read(&path).unwrap(),
        json(root, &["sync-plan", "--generate", path.to_str().unwrap()]),
    );
    assert_eq!(after["changed"], false);
    assert_eq!(std::fs::read(&path).unwrap(), before);
    let check = cli(
        root,
        &["sync-plan", "--check", "--brief", path.to_str().unwrap()],
    );
    assert!(check.status.success(), "{check:?}");
    assert!(
        String::from_utf8_lossy(&check.stdout).contains("0 finding"),
        "{check:?}"
    );
}

#[test]
fn ownership_markers_preserve_narrative_and_dry_run_has_no_writes() {
    let temp = fixture();
    let root = temp.path();
    let path = output_path(root);
    let prefix = "Human roadmap\r\nKeep this exact wording.\r\n";
    let suffix = "\r\nNext phase stays human-owned.\r\n";
    let old = format!("{prefix}<!-- tkt:projection:begin -->\r\nold generated table\r\n<!-- tkt:projection:end -->{suffix}");
    std::fs::write(&path, &old).unwrap();
    let preview = json(
        root,
        &[
            "--dry-run",
            "sync-plan",
            "--generate",
            path.to_str().unwrap(),
        ],
    );
    assert_eq!(preview["dry_run"], true);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), old);
    json(root, &["sync-plan", "--generate", path.to_str().unwrap()]);
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(
        text.starts_with(prefix) && text.ends_with(suffix),
        "{text:?}"
    );
    let absent = root.join("absent.md");
    json(
        root,
        &[
            "--dry-run",
            "sync-plan",
            "--generate",
            absent.to_str().unwrap(),
        ],
    );
    assert!(!absent.exists());
    for foreign in [
        "Human prose without an owned block",
        "<!-- tkt:projection:begin -->\nmissing end",
        "<!-- tkt:projection:end -->\n<!-- tkt:projection:begin -->",
        "```md\n<!-- tkt:projection:begin -->\n<!-- tkt:projection:end -->\n```",
        "<!-- tkt:projection:begin -->\n<!-- tkt:projection:begin -->\n<!-- tkt:projection:end -->",
    ] {
        std::fs::write(&path, foreign).unwrap();
        let out = cli(root, &["sync-plan", "--generate", path.to_str().unwrap()]);
        assert_eq!(out.status.code(), Some(1), "{out:?}");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), foreign);
    }
}

#[test]
fn incomplete_or_duplicate_corpus_and_ticket_output_are_rejected() {
    let temp = fixture();
    let root = temp.path();
    let path = output_path(root);
    let target = root.join(".tickets/generated.md");
    assert_eq!(
        cli(root, &["sync-plan", "--generate", target.to_str().unwrap()])
            .status
            .code(),
        Some(1)
    );
    assert!(!target.exists());
    std::fs::write(root.join(".tickets/bad.md"), "not a ticket").unwrap();
    let out = cli(root, &["sync-plan", "--generate", path.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    assert!(!path.exists());
    std::fs::remove_file(root.join(".tickets/bad.md")).unwrap();
    let source = std::fs::read(root.join(".tickets/04-case.md")).unwrap();
    std::fs::write(root.join(".tickets/04-duplicate.md"), source).unwrap();
    let out = cli(root, &["sync-plan", "--generate", path.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    assert!(!path.exists());
    assert!(!cli(root, &["sync-plan", "--generate", "--fix"])
        .status
        .success());
    assert!(!cli(root, &["sync-plan", "--generate", "--check"])
        .status
        .success());
}

#[test]
#[cfg(windows)]
fn locked_projection_preserves_existing_bytes_on_atomic_write_failure() {
    use std::os::windows::fs::OpenOptionsExt;
    let temp = fixture();
    let root = temp.path();
    let path = output_path(root);
    json(root, &["sync-plan", "--generate", path.to_str().unwrap()]);
    let before = std::fs::read(&path).unwrap();
    let held = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(1)
        .open(&path)
        .unwrap();
    let ticket = root.join(".tickets/04-case.md");
    std::fs::write(
        &ticket,
        std::fs::read_to_string(&ticket)
            .unwrap()
            .replace("Pipe | <tag> [link]", "Changed title"),
    )
    .unwrap();
    let out = cli(root, &["sync-plan", "--generate", path.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    assert_eq!(std::fs::read(&path).unwrap(), before);
    drop(held);
}
#[test]
fn linked_output_and_database_alias_preserve_foreign_content() {
    let temp = fixture();
    let root = temp.path();
    let foreign = root.join("foreign.md");
    std::fs::write(&foreign, "Human text remains untouched").unwrap();
    let output = root.join("linked.md");
    let alias = root.join("database-alias");
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&foreign, &output).unwrap();
        std::os::unix::fs::symlink(root.join(".tickets"), &alias).unwrap();
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_file(&foreign, &output).unwrap();
        std::os::windows::fs::symlink_dir(root.join(".tickets"), &alias).unwrap();
    }
    assert_eq!(
        cli(root, &["sync-plan", "--generate", output.to_str().unwrap()])
            .status
            .code(),
        Some(1)
    );
    assert!(std::fs::symlink_metadata(&output)
        .unwrap()
        .file_type()
        .is_symlink());
    assert_eq!(
        std::fs::read_to_string(&foreign).unwrap(),
        "Human text remains untouched"
    );
    let path = alias.join("generated.md");
    assert_eq!(
        cli(root, &["sync-plan", "--generate", path.to_str().unwrap()])
            .status
            .code(),
        Some(1)
    );
    assert!(!path.exists());
}

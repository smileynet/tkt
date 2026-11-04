use std::path::Path;
use std::process::{Command, Output};

use tempfile::TempDir;

fn repo() -> (TempDir, std::path::PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().to_path_buf();
    git(&root, &["init", "-q"]);
    git(&root, &["config", "user.email", "test@example.invalid"]);
    git(&root, &["config", "user.name", "Test"]);
    let tickets = root.join(".tickets");
    std::fs::create_dir(&tickets).unwrap();
    std::fs::write(tickets.join("config.toml"), "[push]\nenabled = false\n").unwrap();
    for (id, title, tags) in [
        ("01", "Ink", "tags: [ink]\n"),
        ("02", "Other", "tags: [other]\n"),
        ("03", "Untagged", ""),
    ] {
        std::fs::write(
            tickets.join(format!("{id}-ticket.md")),
            format!(
                "---\nid: \"{id}\"\ntitle: \"{title}\"\nstatus: open\nblocked_by: []\n{tags}---\n"
            ),
        )
        .unwrap();
    }
    git(&root, &["add", "."]);
    git(&root, &["commit", "-qm", "seed"]);
    (temp, root)
}

fn git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(["-C", root.to_str().unwrap()])
        .args(args)
        .env("DO_NOT_TRACK", "1")
        .env("TKT_DEBUG", "0")
        .env_remove("TKT_DEBUG_FORMAT")
        .output()
        .unwrap();
    assert!(output.status.success(), "git {args:?}: {output:?}");
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

fn query(root: &Path, args: &[&str], context: Option<&str>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_tkt"));
    command
        .args(args)
        .current_dir(root)
        .env("DO_NOT_TRACK", "1")
        .env("TKT_DEBUG", "0")
        .env_remove("TKT_DEBUG_FORMAT")
        .env("TKT_NO_USER_CONFIG", "1")
        .env_remove("TKT_UPDATE_CHECK")
        .env_remove("TKT_CONTEXT");
    if let Some(context) = context {
        command.env("TKT_CONTEXT", context);
    }
    command.output().unwrap()
}

fn ids(output: &Output) -> Vec<String> {
    assert!(output.status.success(), "{output:?}");
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| {
            serde_json::from_str::<serde_json::Value>(line).unwrap()["id"]
                .as_str()
                .unwrap()
                .to_string()
        })
        .collect()
}

#[test]
fn query_context_is_visible_bypassable_and_keeps_json_stdout_clean() {
    let (_temp, root) = repo();

    let scoped = query(&root, &["query"], Some("ink"));
    assert_eq!(ids(&scoped), ["01", "03"]);
    let notice = String::from_utf8_lossy(&scoped.stderr);
    assert!(
        notice.contains("context"),
        "missing context notice: {notice}"
    );
    assert!(!String::from_utf8_lossy(&scoped.stdout).contains("context"));

    let all = query(&root, &["query", "--all-context"], Some("ink"));
    assert_eq!(ids(&all), ["01", "02", "03"]);
    assert!(!String::from_utf8_lossy(&all.stderr).contains("context"));

    let json_scoped = query(&root, &["-o", "json", "query"], Some("ink"));
    assert_eq!(ids(&json_scoped), ["01", "03"]);
    let warning: serde_json::Value = serde_json::from_slice(&json_scoped.stderr).unwrap();
    assert_eq!(warning["warning"]["kind"], "context_filter");
    assert!(warning["warning"]["message"]
        .as_str()
        .unwrap()
        .contains("--all-context"));

    let unscoped = query(&root, &["query"], None);
    assert_eq!(ids(&unscoped), ["01", "02", "03"]);
    assert!(!String::from_utf8_lossy(&unscoped.stderr).contains("context"));
}

#[test]
fn query_help_explains_context_filter_and_bypass() {
    let output = Command::new(env!("CARGO_BIN_EXE_tkt"))
        .args(["query", "--help"])
        .env("DO_NOT_TRACK", "1")
        .env("TKT_DEBUG", "0")
        .env_remove("TKT_DEBUG_FORMAT")
        .output()
        .unwrap();
    assert!(output.status.success());
    let help = String::from_utf8_lossy(&output.stdout);
    assert!(
        help.contains("context"),
        "missing context explanation: {help}"
    );
    assert!(
        help.contains("--all-context"),
        "missing bypass flag: {help}"
    );
}

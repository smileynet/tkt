use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

fn repo() -> (TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().to_path_buf();
    git(&root, &["init", "-q"]);
    git(&root, &["config", "user.email", "test@example.invalid"]);
    git(&root, &["config", "user.name", "Test"]);
    let tickets = root.join(".tickets");
    std::fs::create_dir(&tickets).unwrap();
    std::fs::write(
        tickets.join("config.toml"),
        "[push]\nenabled = false\n[close]\nrequire_resolution = false\nrequire_checked_acs = false\nallow_force = true\n[validate]\nmissing_resolution_before_id = 100\n",
    )
    .unwrap();
    for id in ["01", "02", "03"] {
        write_ticket(&tickets, id, "open", "");
    }
    git(&root, &["add", "."]);
    git(&root, &["commit", "-qm", "seed"]);
    (temp, root)
}

fn write_ticket(tickets: &Path, id: &str, status: &str, extra: &str) {
    std::fs::write(
        tickets.join(format!("{id}-ticket.md")),
        format!(
            "---\nid: \"{id}\"\ntitle: \"Ticket {id}\"\nstatus: {status}\nblocked_by: []\n{extra}---\n\n# Ticket {id}\n"
        ),
    )
    .unwrap();
}

fn git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(["-C", root.to_str().unwrap()])
        .args(args)
        .env("DO_NOT_TRACK", "1")
        .output()
        .unwrap();
    assert!(output.status.success(), "git {args:?}: {output:?}");
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

fn cli(root: &Path, data_home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tkt"))
        .args(["-q", "-o", "json"])
        .args(args)
        .current_dir(root)
        .env("DO_NOT_TRACK", "1")
        .env("TKT_NO_USER_CONFIG", "1")
        .env("APPDATA", data_home)
        .env("XDG_CONFIG_HOME", data_home)
        .env("XDG_DATA_HOME", data_home)
        .env("XDG_STATE_HOME", data_home.join("state"))
        .env("TKT_DEBUG", "0")
        .env_remove("TKT_DEBUG_FORMAT")
        .env_remove("TKT_UPDATE_CHECK")
        .env_remove("TKT_CONTEXT")
        .env_remove("TKT_AGENT")
        .output()
        .unwrap()
}

fn assert_success_object(name: &str, output: &Output) {
    assert!(
        output.status.success(),
        "{name}: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<_> = stdout.lines().filter(|line| !line.is_empty()).collect();
    assert_eq!(
        lines.len(),
        1,
        "{name} must emit one JSON object: {stdout:?}"
    );
    let json: serde_json::Value = serde_json::from_str(lines[0])
        .unwrap_or_else(|error| panic!("{name} emitted non-JSON success: {error}: {stdout:?}"));
    assert_eq!(json["ok"], true, "{name}: {json}");
    assert!(json.get("result").is_some(), "{name}: {json}");
}

#[test]
fn write_commands_keep_one_json_success_object_in_quiet_mode() {
    let commands = [
        "new",
        "batch",
        "claim",
        "edit",
        "close",
        "cancel",
        "reopen",
        "renumber",
        "context",
        "config",
        "init",
        "lint",
        "validate-fix",
        "sync-plan-fix",
        "migrate",
        "rebase",
    ];

    for name in commands {
        let (temp, root) = repo();
        let data_home = temp.path().join("private-data");
        let output = match name {
            "new" => cli(&root, &data_home, &["new", "fresh", "--title", "Fresh"]),
            "batch" => cli(&root, &data_home, &["batch", "one:One", "two:Two"]),
            "claim" => cli(&root, &data_home, &["claim", "01"]),
            "edit" => cli(&root, &data_home, &["edit", "01", "--title", "Changed"]),
            "close" => cli(&root, &data_home, &["close", "01", "--force"]),
            "cancel" => cli(
                &root,
                &data_home,
                &["cancel", "01", "--reason", "obsolete", "--note", "Retired"],
            ),
            "reopen" => {
                assert_success_object(
                    "cancel setup",
                    &cli(
                        &root,
                        &data_home,
                        &["cancel", "01", "--reason", "obsolete", "--note", "Retired"],
                    ),
                );
                cli(&root, &data_home, &["reopen", "01"])
            }
            "renumber" => cli(&root, &data_home, &["renumber", "01", "07"]),
            "context" => cli(&root, &data_home, &["context", "+work"]),
            "config" => cli(
                &root,
                &data_home,
                &[
                    "config",
                    "--project",
                    "--set",
                    "priority.warn_unknown=false",
                ],
            ),
            "init" => {
                std::fs::remove_dir_all(root.join(".tickets")).unwrap();
                cli(&root, &data_home, &["init"])
            }
            "lint" => {
                std::fs::write(
                    root.join(".tickets/04-noncanonical.md"),
                    "---\nid: 04\ntitle: Noncanonical\nstatus: open\nblocked_by: []\n---\n",
                )
                .unwrap();
                git(&root, &["add", ".tickets/04-noncanonical.md"]);
                git(&root, &["commit", "-qm", "add noncanonical ticket"]);
                cli(&root, &data_home, &["lint"])
            }
            "validate-fix" => {
                std::fs::write(
                    root.join(".tickets/04-legacy.md"),
                    "---\nid: \"04\"\ntitle: \"Legacy\"\nstatus: closed\nblocked_by: []\n---\n",
                )
                .unwrap();
                git(&root, &["add", ".tickets/04-legacy.md"]);
                git(&root, &["commit", "-qm", "add legacy ticket"]);
                cli(&root, &data_home, &["validate", "--fix"])
            }
            "sync-plan-fix" => {
                std::fs::create_dir(root.join("docs")).unwrap();
                std::fs::write(
                    root.join("docs/plan.md"),
                    "# Plan\n\n| ID | Work | Status |\n|---:|---|---|\n",
                )
                .unwrap();
                git(&root, &["add", "docs/plan.md"]);
                git(&root, &["commit", "-qm", "add empty plan"]);
                cli(&root, &data_home, &["sync-plan", "--fix"])
            }
            "migrate" => {
                for entry in std::fs::read_dir(root.join(".tickets")).unwrap() {
                    let entry = entry.unwrap();
                    if entry
                        .path()
                        .extension()
                        .is_some_and(|extension| extension == "md")
                    {
                        std::fs::remove_file(entry.path()).unwrap();
                    }
                }
                std::fs::write(
                    root.join(".tickets/legacy.md"),
                    "---\nstatus: open\ndeps: []\n---\n\n# Legacy\n\nMigrate me.\n",
                )
                .unwrap();
                cli(&root, &data_home, &["migrate", "--from", "tk", "--confirm"])
            }
            "rebase" => {
                let remote = temp.path().join("remote.git");
                std::fs::create_dir(&remote).unwrap();
                git(&remote, &["init", "--bare", "-q", "-b", "main"]);
                git(
                    &root,
                    &["remote", "add", "origin", remote.to_str().unwrap()],
                );
                git(&root, &["push", "-q", "-u", "origin", "HEAD:main"]);
                std::fs::write(
                    root.join(".tickets/01-collision.md"),
                    "---\nid: \"01\"\ntitle: \"Collision\"\nstatus: open\nblocked_by: []\n---\n",
                )
                .unwrap();
                git(&root, &["add", ".tickets/01-collision.md"]);
                git(&root, &["commit", "-qm", "add collision"]);
                cli(&root, &data_home, &["rebase"])
            }
            _ => unreachable!(),
        };
        assert_success_object(name, &output);
    }
}

#[test]
fn quiet_and_json_help_explains_the_combination() {
    let output = Command::new(env!("CARGO_BIN_EXE_tkt"))
        .arg("--help")
        .env("DO_NOT_TRACK", "1")
        .output()
        .unwrap();
    assert!(output.status.success());
    let help = String::from_utf8_lossy(&output.stdout);
    assert!(help.contains("-o json still emits structured success output"));
    assert!(help.contains("JSON is preserved with --quiet"));
}

#[test]
fn quiet_json_previews_and_writer_noops_remain_structured_without_writing() {
    let (temp, root) = repo();
    let data_home = temp.path().join("private-data");
    let tickets = root.join(".tickets");

    for args in [
        vec!["--dry-run", "new", "preview", "--title", "Preview"],
        vec!["--dry-run", "batch", "one:One", "two:Two"],
        vec!["--dry-run", "claim", "01"],
        vec!["--dry-run", "edit", "01", "--title", "Preview"],
        vec!["--dry-run", "close", "01", "--force"],
        vec![
            "--dry-run",
            "cancel",
            "01",
            "--reason",
            "obsolete",
            "--note",
            "Preview",
        ],
        vec!["--dry-run", "renumber", "01", "07"],
        vec!["--dry-run", "context", "+preview"],
        vec![
            "--dry-run",
            "config",
            "--project",
            "--set",
            "priority.warn_unknown=false",
        ],
        vec!["--dry-run", "init"],
    ] {
        let before = git(&root, &["status", "--porcelain"]);
        let output = cli(&root, &data_home, &args);
        assert_success_object(args[1], &output);
        assert_eq!(git(&root, &["status", "--porcelain"]), before, "{args:?}");
    }

    let lint_before = std::fs::read(tickets.join("01-ticket.md")).unwrap();
    assert_success_object("lint no-op", &cli(&root, &data_home, &["lint"]));
    assert_eq!(
        std::fs::read(tickets.join("01-ticket.md")).unwrap(),
        lint_before
    );

    std::fs::write(
        tickets.join("04-noncanonical.md"),
        "---\nid: 04\ntitle: Noncanonical\nstatus: open\nblocked_by: []\n---\n",
    )
    .unwrap();
    let lint_before = std::fs::read(tickets.join("04-noncanonical.md")).unwrap();
    assert_success_object(
        "lint preview",
        &cli(&root, &data_home, &["--dry-run", "lint"]),
    );
    assert_eq!(
        std::fs::read(tickets.join("04-noncanonical.md")).unwrap(),
        lint_before
    );

    std::fs::create_dir(root.join("docs")).unwrap();
    std::fs::write(
        root.join("docs/plan.md"),
        "# Plan\n\n| ID | Work | Status |\n|---:|---|---|\n| 01 | Ticket | done |\n",
    )
    .unwrap();
    let plan_before = std::fs::read(root.join("docs/plan.md")).unwrap();
    assert_success_object(
        "sync-plan preview",
        &cli(&root, &data_home, &["sync-plan", "--fix", "--dry-run"]),
    );
    assert_eq!(
        std::fs::read(root.join("docs/plan.md")).unwrap(),
        plan_before
    );

    assert_success_object(
        "validate preview",
        &cli(&root, &data_home, &["validate", "--fix", "--dry-run"]),
    );
    assert_success_object("rebase no-op", &cli(&root, &data_home, &["rebase"]));

    for entry in std::fs::read_dir(&tickets).unwrap() {
        let entry = entry.unwrap();
        if entry
            .path()
            .extension()
            .is_some_and(|extension| extension == "md")
        {
            std::fs::remove_file(entry.path()).unwrap();
        }
    }
    assert_success_object(
        "migrate empty no-op",
        &cli(&root, &data_home, &["--dry-run", "migrate", "--from", "tk"]),
    );
}

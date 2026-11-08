use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;
use tempfile::TempDir;

fn git(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(["-C", root.to_str().unwrap()])
        .args(args)
        .env("DO_NOT_TRACK", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn cli(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tkt"))
        .args(args)
        .current_dir(root)
        .env("DO_NOT_TRACK", "1")
        .env("TKT_NO_USER_CONFIG", "1")
        .env("TKT_ASCII", "1")
        .env_remove("TKT_DEBUG")
        .env_remove("TKT_DEBUG_FORMAT")
        .env_remove("TKT_UPDATE_CHECK")
        .env_remove("TKT_CONTEXT")
        .env_remove("TKT_AGENT")
        .output()
        .unwrap()
}

fn success_json(output: &Output) -> Value {
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        stdout.lines().filter(|line| !line.is_empty()).count(),
        1,
        "expected one JSON object: {stdout:?}"
    );
    let json: Value = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(json["ok"], true, "{json}");
    json
}

fn local_repo() -> (TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().to_path_buf();
    git(&root, &["init", "-q"]);
    git(&root, &["config", "user.email", "test@example.invalid"]);
    git(&root, &["config", "user.name", "Test"]);
    std::fs::create_dir(root.join(".tickets")).unwrap();
    std::fs::write(
        root.join(".tickets/config.toml"),
        "[push]\nenabled = false\n",
    )
    .unwrap();
    git(&root, &["add", "."]);
    git(&root, &["commit", "-qm", "seed"]);
    (temp, root)
}

fn assert_created(json: &Value, id: &str, slug: &str) {
    let path = format!(".tickets/{id}-{slug}.md");
    assert_eq!(json["id"], id, "{json}");
    assert_eq!(json["path"], path, "{json}");
}

fn assert_batch_created(json: &Value, expected: &[(&str, &str)]) {
    assert!(
        json["result"].as_str().unwrap().contains("tickets"),
        "{json}"
    );
    let tickets = json["tickets"].as_array().unwrap();
    assert_eq!(tickets.len(), expected.len(), "{json}");
    for (ticket, (id, slug)) in tickets.iter().zip(expected) {
        assert_created(ticket, id, slug);
    }
}

#[test]
fn creation_json_exposes_ids_and_paths_in_normal_quiet_and_dry_run_modes() {
    let (_temp, root) = local_repo();

    let created = success_json(&cli(
        &root,
        &["-o", "json", "new", "alpha", "--title", "Alpha"],
    ));
    assert_created(&created, "01", "alpha");
    assert!(root.join(created["path"].as_str().unwrap()).exists());

    let quiet = success_json(&cli(
        &root,
        &["-q", "-o", "json", "new", "beta", "--title", "Beta"],
    ));
    assert_created(&quiet, "02", "beta");

    let before = cli(&root, &["--dry-run", "new", "unused"]);
    assert!(before.status.success());
    let preview = success_json(&cli(
        &root,
        &[
            "-q",
            "-o",
            "json",
            "--dry-run",
            "new",
            "gamma",
            "--title",
            "Gamma",
        ],
    ));
    assert_created(&preview, "03", "gamma");
    assert!(preview["result"].as_str().unwrap().contains("would create"));
    assert!(!root.join(".tickets/03-gamma.md").exists());

    let batch = success_json(&cli(
        &root,
        &["-o", "json", "batch", "three:Three", "four:Four"],
    ));
    assert_batch_created(&batch, &[("03", "three"), ("04", "four")]);
    for ticket in batch["tickets"].as_array().unwrap() {
        assert!(root.join(ticket["path"].as_str().unwrap()).exists());
    }

    let quiet_batch = success_json(&cli(
        &root,
        &["-q", "-o", "json", "batch", "five:Five", "six:Six"],
    ));
    assert_batch_created(&quiet_batch, &[("05", "five"), ("06", "six")]);

    let batch_preview = success_json(&cli(
        &root,
        &[
            "-q",
            "-o",
            "json",
            "--dry-run",
            "batch",
            "seven:Seven",
            "eight:Eight",
        ],
    ));
    assert_batch_created(&batch_preview, &[("07", "seven"), ("08", "eight")]);
    assert!(!root.join(".tickets/07-seven.md").exists());
    assert!(!root.join(".tickets/08-eight.md").exists());

    let human = cli(&root, &["new", "human", "--title", "Human"]);
    assert!(human.status.success(), "{human:?}");
    let human = String::from_utf8_lossy(&human.stdout);
    assert!(human.contains("created 07 human (local only)"), "{human}");
    assert!(!human.contains("\"id\""), "{human}");
}

#[test]
fn edit_ac_help_describes_repeatable_one_based_indices() {
    let (_temp, root) = local_repo();
    let output = cli(&root, &["edit", "--help"]);
    assert!(output.status.success(), "{output:?}");
    let help = String::from_utf8_lossy(&output.stdout);
    assert!(
        help.contains(
            "Check acceptance criteria by 1-based index; repeat or comma-separate indices"
        ),
        "{help}"
    );
}

fn remote_pair() -> (TempDir, PathBuf, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let remote = temp.path().join("remote.git");
    let clone_a = temp.path().join("clone-a");
    let clone_b = temp.path().join("clone-b");
    git(
        temp.path(),
        &[
            "init",
            "--bare",
            "-q",
            "-b",
            "main",
            remote.to_str().unwrap(),
        ],
    );
    git(
        temp.path(),
        &[
            "clone",
            "-q",
            remote.to_str().unwrap(),
            clone_a.to_str().unwrap(),
        ],
    );
    git(&clone_a, &["config", "user.email", "a@example.invalid"]);
    git(&clone_a, &["config", "user.name", "A"]);
    std::fs::create_dir(clone_a.join(".tickets")).unwrap();
    std::fs::write(
        clone_a.join(".tickets/01-seed.md"),
        "---\nid: \"01\"\ntitle: \"Seed\"\nstatus: open\nblocked_by: []\n---\n\n# Seed\n",
    )
    .unwrap();
    git(&clone_a, &["add", "."]);
    git(&clone_a, &["commit", "-qm", "seed"]);
    git(&clone_a, &["push", "-q", "origin", "HEAD:main"]);
    git(
        temp.path(),
        &[
            "clone",
            "-q",
            remote.to_str().unwrap(),
            clone_b.to_str().unwrap(),
        ],
    );
    git(&clone_b, &["config", "user.email", "b@example.invalid"]);
    git(&clone_b, &["config", "user.name", "B"]);
    (temp, clone_a, clone_b)
}

#[test]
fn new_json_reports_the_final_id_and_path_after_allocation_retry() {
    let (_temp, clone_a, clone_b) = remote_pair();
    let first = cli(&clone_a, &["new", "alpha", "--title", "Alpha"]);
    assert!(first.status.success(), "{first:?}");

    let retried = success_json(&cli(
        &clone_b,
        &["-o", "json", "new", "beta", "--title", "Beta"],
    ));
    assert_created(&retried, "03", "beta");
    assert!(retried["result"].as_str().unwrap().contains("renumbered"));
    assert!(clone_b.join(".tickets/03-beta.md").exists());
    assert!(!clone_b.join(".tickets/02-beta.md").exists());
}

#[test]
fn batch_json_reports_final_ids_and_paths_after_allocation_retry() {
    let (_temp, clone_a, clone_b) = remote_pair();
    let first = cli(&clone_a, &["batch", "alpha:Alpha", "beta:Beta"]);
    assert!(first.status.success(), "{first:?}");

    let retried = success_json(&cli(
        &clone_b,
        &["-o", "json", "batch", "gamma:Gamma", "delta:Delta"],
    ));
    assert_batch_created(&retried, &[("04", "gamma"), ("05", "delta")]);
    for slug in ["gamma", "delta"] {
        assert!(!clone_b.join(format!(".tickets/02-{slug}.md")).exists());
        assert!(!clone_b.join(format!(".tickets/03-{slug}.md")).exists());
    }
    assert!(clone_b.join(".tickets/04-gamma.md").exists());
    assert!(clone_b.join(".tickets/05-delta.md").exists());
}

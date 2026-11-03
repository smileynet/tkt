use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};

fn cli(root: &Path, args: &[&str], input: Option<&str>) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tkt"))
        .args(args)
        .current_dir(root)
        .env("DO_NOT_TRACK", "1")
        .env("TKT_NO_USER_CONFIG", "1")
        .env("TKT_UPDATE_CHECK", "0")
        .env_remove("TKT_DEBUG")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    if let Some(input) = input {
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
    }
    child.wait_with_output().unwrap()
}

fn repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for args in [
        vec!["init", "-q"],
        vec!["config", "user.name", "Fixture"],
        vec!["config", "user.email", "fixture@example.invalid"],
    ] {
        assert!(Command::new("git")
            .args(args)
            .current_dir(dir.path())
            .env("DO_NOT_TRACK", "1")
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
    dir
}

const BODY: &str = "# Filled task\n\n## What to build\n\nDeliver a concrete bounded change from its cited source.\n\n## Acceptance criteria\n\n- [ ] Public behavior is verified.\n";

#[test]
fn new_body_template_prompts_and_remains_detectable() {
    let dir = repo();
    let created = cli(dir.path(), &["new", "prompt"], None);
    assert!(created.status.success(), "{created:?}");
    let text = std::fs::read_to_string(dir.path().join(".tickets/01-prompt.md")).unwrap();
    assert!(!text.lines().any(|line| line == "TBD"));
    assert!(text.contains("link its source"), "{text}");
    let check = cli(dir.path(), &["validate", "--strict"], None);
    assert_eq!(check.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&check.stdout).contains("stub-body"));
    let batch = cli(dir.path(), &["batch", "second", "third"], None);
    assert!(batch.status.success(), "{batch:?}");
    let batch_text = std::fs::read_to_string(dir.path().join(".tickets/02-second.md")).unwrap();
    assert!(batch_text.contains("link its source"));
}

#[test]
fn new_body_from_file_stdin_and_named_templates() {
    let dir = repo();
    let file = dir.path().join("body with spaces.md");
    std::fs::write(&file, BODY).unwrap();
    let source = format!("@{}", file.display());
    let created = cli(dir.path(), &["new", "file", "--body", &source], None);
    assert!(created.status.success(), "{created:?}");
    assert!(
        std::fs::read_to_string(dir.path().join(".tickets/01-file.md"))
            .unwrap()
            .ends_with(BODY)
    );
    let created = cli(dir.path(), &["new", "stdin", "--body", "-"], Some(BODY));
    assert!(created.status.success(), "{created:?}");
    assert!(
        std::fs::read_to_string(dir.path().join(".tickets/02-stdin.md"))
            .unwrap()
            .ends_with(BODY)
    );
    std::fs::create_dir(dir.path().join(".tickets/templates")).unwrap();
    std::fs::write(dir.path().join(".tickets/templates/brief.md"), BODY).unwrap();
    let created = cli(dir.path(), &["new", "named", "--template", "brief"], None);
    assert!(created.status.success(), "{created:?}");
    let batch = cli(
        dir.path(),
        &["batch", "batch-one", "batch-two", "--template", "brief"],
        None,
    );
    assert!(batch.status.success(), "{batch:?}");
    for path in ["03-named.md", "04-batch-one.md", "05-batch-two.md"] {
        assert!(
            std::fs::read_to_string(dir.path().join(".tickets").join(path))
                .unwrap()
                .ends_with(BODY)
        );
    }
}

#[test]
fn missing_or_unsafe_template_and_dry_run_do_not_create_tickets() {
    let dir = repo();
    for args in [
        vec!["new", "missing", "--template", "absent"],
        vec!["batch", "missing", "--template", "../escape"],
    ] {
        let output = cli(dir.path(), &args, None);
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("template"));
    }
    let dry = cli(
        dir.path(),
        &["new", "dry", "--body", BODY, "--dry-run"],
        None,
    );
    assert!(dry.status.success(), "{dry:?}");
    assert_eq!(
        std::fs::read_dir(dir.path().join(".tickets"))
            .unwrap()
            .count(),
        1
    );
}

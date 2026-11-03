use std::path::Path;
use std::process::Command;

fn repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "-q"]);
    git(dir.path(), &["config", "user.name", "Test"]);
    git(
        dir.path(),
        &["config", "user.email", "test@example.invalid"],
    );
    std::fs::create_dir(dir.path().join(".tickets")).unwrap();
    dir
}

fn git(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(["-C", root.to_str().unwrap()])
        .args(args)
        .env("DO_NOT_TRACK", "1")
        .output()
        .unwrap();
    assert!(output.status.success(), "git {args:?}: {output:?}");
}

fn query(root: &Path) -> Vec<serde_json::Value> {
    let output = Command::new(env!("CARGO_BIN_EXE_tkt"))
        .args(["query", "-o", "json"])
        .current_dir(root)
        .env("DO_NOT_TRACK", "1")
        .env("TKT_NO_USER_CONFIG", "1")
        .env("TKT_UPDATE_CHECK", "0")
        .env_remove("TKT_CONTEXT")
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

#[test]
fn query_includes_typed_fields_and_preserves_unknown_raw_frontmatter() {
    let dir = repo();
    std::fs::write(
        dir.path().join(".tickets/01-seed.md"),
        "---\nid: \"01\"\ntitle: Seed\nstatus: done\nblocked_by: []\n---\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join(".tickets/02-feature.md"),
        "---\nid: \"02\"\ntitle: Feature\nstatus: open\nblocked_by: [\"01\"]\nenv: either\npriority: urgent\nspec: specs/feature.md\nparent: \"01\"\nassignee: codex\nvalidation_criteria:\n  - tests pass\n  - output is stable\ntags: [\"stream:cli\", \"kind:bug\"]\nrequires: [\"gpu\", \"network\"]\nagents: [\"claude\", \"codex\"]\ncustom: \"preserve me\"\npending_blockers: \"stored user data\"\nfrontmatter: \"a stored key named frontmatter\"\n---\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join(".tickets/03-empty-optionals.md"),
        "---\nid: \"03\"\ntitle: Empty optionals\nstatus: open\nblocked_by: []\nspec:\nparent:\npriority:\n---\n",
    )
    .unwrap();

    let rows = query(dir.path());
    let row = rows.iter().find(|row| row["id"] == "02").unwrap();
    assert_eq!(row["id"], "02");
    assert_eq!(row["title"], "Feature");
    assert_eq!(row["status"], "open");
    assert_eq!(row["blocked_by"], serde_json::json!(["01"]));
    assert_eq!(row["pending_blockers"], serde_json::json!([]));
    assert_eq!(row["env"], "either");
    assert_eq!(row["priority"], "urgent");
    assert_eq!(row["spec"], "specs/feature.md");
    assert_eq!(row["parent"], "01");
    assert_eq!(row["assignee"], "codex");
    assert_eq!(
        row["validation_criteria"],
        serde_json::json!(["tests pass", "output is stable"])
    );
    assert_eq!(row["tags"], serde_json::json!(["stream:cli", "kind:bug"]));
    assert_eq!(row["requires"], serde_json::json!(["gpu", "network"]));
    assert_eq!(row["agents"], serde_json::json!(["claude", "codex"]));
    assert_eq!(row["frontmatter"]["custom"], "\"preserve me\"");
    assert_eq!(
        row["frontmatter"]["pending_blockers"],
        "\"stored user data\""
    );
    assert_eq!(
        row["frontmatter"]["frontmatter"],
        "\"a stored key named frontmatter\""
    );
    let empty_optionals = rows.iter().find(|row| row["id"] == "03").unwrap();
    assert_eq!(empty_optionals["spec"], "");
    assert_eq!(empty_optionals["parent"], "");
    assert_eq!(empty_optionals["frontmatter"]["priority"], "");
}

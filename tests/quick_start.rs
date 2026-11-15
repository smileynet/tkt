use std::path::Path;
use std::process::{Command, Output};

fn cli(root: &Path, args: &[&str]) -> Output {
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

fn ok(root: &Path, args: &[&str]) -> String {
    let output = cli(root, args);
    assert!(output.status.success(), "{args:?}: {output:?}");
    String::from_utf8(output.stdout).unwrap()
}

fn repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for args in [
        vec!["init", "-q"],
        vec!["config", "user.name", "Quick Start"],
        vec!["config", "user.email", "quick-start@example.invalid"],
    ] {
        assert!(Command::new("git")
            .args(args)
            .current_dir(dir.path())
            .env("DO_NOT_TRACK", "1")
            .status()
            .unwrap()
            .success());
    }
    dir
}

const BODY: &str = "# Implement authentication\n\n## Intent\n\nProtect the API with JWT access and refresh tokens.\n\n## Acceptance criteria\n\n- [ ] Users can authenticate and refresh their session.\n";
const VALIDATION: &str = "Authentication and session refresh pass integration tests";

#[test]
fn documented_quick_start_closes_a_valid_ticket_in_a_fresh_repository() {
    let dir = repo();
    ok(dir.path(), &["init"]);
    ok(
        dir.path(),
        &[
            "new",
            "auth",
            "--title",
            "Implement authentication",
            "--body",
            BODY,
            "--validation",
            VALIDATION,
        ],
    );
    ok(
        dir.path(),
        &["new", "api", "--title", "Build API", "--blocked-by", "01"],
    );
    assert!(ok(dir.path(), &["ready"]).contains("Implement authentication"));
    ok(
        dir.path(),
        &[
            "close",
            "01",
            "--check-all",
            "--evidence",
            "Authentication and refresh integration tests pass",
            "--resolution",
            "JWT access and refresh tokens shipped",
        ],
    );
    let ready = ok(dir.path(), &["ready"]);
    assert!(ready.contains("Build API"), "{ready}");
    assert!(!ready.contains("Implement authentication"), "{ready}");
    let ticket = std::fs::read_to_string(dir.path().join(".tickets/01-auth.md")).unwrap();
    assert!(ticket.contains("- [x] Users can authenticate"), "{ticket}");
    assert!(
        ticket.contains("1. ✓ Authentication and session refresh pass integration tests — \"Authentication and refresh integration tests pass\""),
        "{ticket}"
    );
    assert!(
        ticket.contains("JWT access and refresh tokens shipped"),
        "{ticket}"
    );

    let readme =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("README.md")).unwrap();
    for documented in [
        "tkt init",
        "--validation",
        "--check-all",
        "--evidence",
        "--resolution",
    ] {
        assert!(
            readme.contains(documented),
            "README quick start omits {documented}"
        );
    }
}

#[test]
fn capabilities_scripting_recipe_selects_at_most_one_ready_ticket() {
    let manifest: serde_json::Value = serde_json::from_str(&ok(
        Path::new(env!("CARGO_MANIFEST_DIR")),
        &["capabilities"],
    ))
    .unwrap();
    let recipe = manifest["workflows"]["scripting"].as_str().unwrap();
    assert!(recipe.contains("head -n 1"), "{recipe}");
    assert!(recipe.contains("test -n \"$id\""), "{recipe}");
    assert!(!recipe.contains("xargs tkt claim"), "{recipe}");

    let dir = repo();
    ok(dir.path(), &["init"]);
    ok(dir.path(), &["new", "first", "--priority", "high"]);
    ok(dir.path(), &["new", "second", "--priority", "low"]);
    let ready = ok(dir.path(), &["ready", "--json"]);
    let selected = ready
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter_map(|ticket| ticket["id"].as_str().map(str::to_owned))
        .next()
        .unwrap();
    assert_eq!(selected, "01");
    ok(dir.path(), &["claim", &selected]);
    let second = std::fs::read_to_string(dir.path().join(".tickets/02-second.md")).unwrap();
    assert!(second.contains("status: open"), "{second}");
}

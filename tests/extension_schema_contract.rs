use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn command(program: &str) -> Command {
    let mut command = Command::new(program);
    command
        .env("DO_NOT_TRACK", "1")
        .env("TKT_NO_USER_CONFIG", "1")
        .env("TKT_DEBUG", "0")
        .env("TKT_UPDATE_CHECK", "0")
        .env(
            "GIT_CONFIG_GLOBAL",
            if cfg!(windows) { "NUL" } else { "/dev/null" },
        )
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_COUNT", "0")
        .env_remove("GIT_CONFIG_PARAMETERS")
        .env_remove("TKT_AGENT")
        .env_remove("TKT_CONTEXT");
    command
}
fn git(root: &Path, args: &[&str]) -> String {
    let out = command("git")
        .current_dir(root)
        .args(args)
        .output()
        .unwrap();
    assert!(out.status.success(), "{args:?}: {out:?}");
    String::from_utf8(out.stdout).unwrap().trim().into()
}
fn cli(root: &Path, args: &[&str]) -> Output {
    command(env!("CARGO_BIN_EXE_tkt"))
        .current_dir(root)
        .env("XDG_CONFIG_HOME", root.parent().unwrap().join("config"))
        .args(args)
        .output()
        .unwrap()
}
fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn walk(root: &Path, path: &Path, all: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in std::fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(root, &path, all)
            } else {
                all.insert(
                    path.strip_prefix(root).unwrap().into(),
                    std::fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut all = BTreeMap::new();
    walk(root, root, &mut all);
    all
}
#[test]
fn proposed_scalar_representations_round_trip_through_current_cli_without_enforcement() {
    for (ticket, schema, key) in [
        (
            include_str!("fixtures/extension-schema-v1/before.md"),
            include_str!("fixtures/extension-schema-v1/schema.toml"),
            "x_team",
        ),
        (
            include_str!("fixtures/extension-schema-v1/after.md"),
            include_str!("fixtures/extension-schema-v1/schema-after.toml"),
            "x_group",
        ),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("repo");
        std::fs::create_dir(&root).unwrap();
        git(&root, &["init", "-q", "-b", "main"]);
        git(&root, &["config", "user.name", "Schema fixture"]);
        git(&root, &["config", "user.email", "fixture@example.invalid"]);
        git(&root, &["config", "core.autocrlf", "false"]);
        git(&root, &["config", "maintenance.auto", "false"]);
        git(&root, &["config", "gc.auto", "0"]);
        std::fs::create_dir(root.join(".tickets")).unwrap();
        std::fs::write(root.join(".tickets/config.toml"), "[push]\nenabled=false\n").unwrap();
        std::fs::write(root.join(".tickets/schema.toml"), schema).unwrap();
        let path = root.join(".tickets/01-extension-fixture.md");
        std::fs::write(&path, ticket).unwrap();
        git(&root, &["add", "."]);
        git(&root, &["commit", "-qm", "schema fixture"]);
        let before = snapshot(&root);
        let out = cli(&root, &["query", "--all-context", "-o", "json"]);
        assert!(out.status.success(), "{out:?}");
        let query: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(query["frontmatter"][key], "\"platform\"");
        assert_eq!(query["frontmatter"]["legacy_plain"], "backend");
        assert_eq!(query["frontmatter"]["legacy_list"], "[\"keep\", \"raw\"]");
        assert_eq!(query["frontmatter"]["legacy_block"], "arbitrary: unchanged");
        let raw = query["frontmatter"]["x_summary"].as_str().unwrap();
        let decoded: String = serde_json::from_str(raw).unwrap();
        assert_eq!(decoded, "Quote \" and Unicode Ω; line\nnext");
        let out = cli(
            &root,
            &["--dry-run", "edit", "01", "--title", "Updated fixture"],
        );
        assert!(out.status.success(), "{out:?}");
        assert_eq!(snapshot(&root), before);
        let out = cli(&root, &["edit", "01", "--title", "Updated fixture"]);
        assert!(out.status.success(), "{out:?}");
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            ticket.replace("title: \"Extension fixture\"", "title: \"Updated fixture\"")
        );
        assert_eq!(
            std::fs::read_to_string(root.join(".tickets/schema.toml")).unwrap(),
            schema
        );
    }
}

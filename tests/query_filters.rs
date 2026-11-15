use std::process::{Command, Output};

fn run(root: &std::path::Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tkt"))
        .arg("query")
        .args(args)
        .current_dir(root)
        .env("DO_NOT_TRACK", "1")
        .env("TKT_DEBUG", "0")
        .env_remove("TKT_DEBUG_FORMAT")
        .env("TKT_NO_USER_CONFIG", "1")
        .env("TKT_UPDATE_CHECK", "0")
        .env("TKT_CONTEXT", "cli")
        .output()
        .unwrap()
}

fn fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let output = Command::new("git")
        .args(["init", "-q"])
        .current_dir(dir.path())
        .env("DO_NOT_TRACK", "1")
        .output()
        .unwrap();
    assert!(output.status.success());
    std::fs::create_dir(dir.path().join(".tickets")).unwrap();
    for (id, title, status, priority, tags, body) in [
        (
            "01",
            "Alpha",
            "open",
            "high",
            "cli, bug",
            "Mixed NEEDLE body",
        ),
        ("02", "Needle title", "done", "high", "cli", "other"),
        ("03", "Other", "open", "low", "cli, bug", "needle"),
        ("04", "Needle outside", "open", "high", "bug", "other"),
    ] {
        std::fs::write(
            dir.path().join(format!(".tickets/{id}-test.md")),
            format!("---\nid: \"{id}\"\ntitle: {title}\nstatus: {status}\npriority: {priority}\nblocked_by: []\ntags: [{tags}]\n---\n{body}\n"),
        )
        .unwrap();
    }
    dir
}

#[test]
fn combines_literal_match_tags_status_priority_and_projection() {
    let dir = fixture();
    let output = run(
        dir.path(),
        &[
            "--match",
            "nEeDlE",
            "--tag",
            "cli",
            "--tag",
            "bug",
            "--status",
            "open",
            "--priority",
            "high",
            "--fields",
            "id,status,title,spec",
            "-o",
            "json",
        ],
    );
    assert!(output.status.success(), "{output:?}");
    let rows: Vec<serde_json::Value> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(
        rows,
        vec![serde_json::json!({"id":"01", "status":"open", "title":"Alpha", "spec":null})]
    );
    let literal = run(dir.path(), &["--match", "n.*e"]);
    assert!(literal.status.success());
    assert!(literal.stdout.is_empty());
}

#[test]
fn title_body_context_and_text_projection_are_composable() {
    let dir = fixture();
    let output = run(
        dir.path(),
        &["--match", "NEEDLE", "--fields", "id,title", "-o", "text"],
    );
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "\"01\"\t\"Alpha\"\n\"02\"\t\"Needle title\"\n\"03\"\t\"Other\"\n"
    );
    let output = run(
        dir.path(),
        &[
            "--all-context",
            "--tag",
            "bug",
            "--priority",
            "high",
            "--fields",
            "id",
            "-o",
            "json",
            "-q",
        ],
    );
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "{\"id\":\"01\"}\n{\"id\":\"04\"}\n"
    );
}

#[test]
fn rejects_invalid_projection_before_emitting_rows() {
    let dir = fixture();
    for fields in ["unknown", "id,", "id,id", " "] {
        let output = run(dir.path(), &["--fields", fields, "-o", "json"]);
        assert!(!output.status.success(), "{fields}: {output:?}");
        assert!(output.stdout.is_empty());
        let stderr = String::from_utf8(output.stderr).unwrap();
        let error: serde_json::Value =
            serde_json::from_str(stderr.lines().last().unwrap()).unwrap();
        assert_eq!(error["error"]["kind"], "validation");
    }
}

#[test]
fn projected_text_escapes_strings_and_retains_compound_values() {
    let dir = fixture();
    std::fs::write(
        dir.path().join(".tickets/05-escaped.md"),
        "---\nid: \"05\"\ntitle: \"Line\\nTab\\tQuote\\\"\"\nstatus: open\nblocked_by: []\ntags: [cli]\ncustom: retained\n---\n",
    )
    .unwrap();
    for format in ["text", "json"] {
        let output = run(
            dir.path(),
            &[
                "--match",
                "Quote",
                "--fields",
                "title,tags,frontmatter",
                "-o",
                format,
            ],
        );
        assert!(output.status.success(), "{output:?}");
        let text = String::from_utf8(output.stdout).unwrap();
        assert_eq!(text.lines().count(), 1);
        if format == "text" {
            let values: Vec<serde_json::Value> = text
                .trim_end()
                .split('\t')
                .map(|value| serde_json::from_str(value).unwrap())
                .collect();
            assert_eq!(
                values,
                vec![
                    serde_json::json!("Line\nTab\tQuote\""),
                    serde_json::json!(["cli"]),
                    serde_json::json!({"custom":"retained"})
                ]
            );
        } else {
            let row: serde_json::Value = serde_json::from_str(&text).unwrap();
            assert_eq!(row["title"], "Line\nTab\tQuote\"");
            assert_eq!(row["tags"], serde_json::json!(["cli"]));
            assert_eq!(row["frontmatter"], serde_json::json!({"custom":"retained"}));
        }
    }
}

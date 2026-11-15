use std::path::Path;
use std::process::{Command, Output};

use serde_json::Value;
use tempfile::TempDir;

fn cli(root: &Path, args: &[&str], agent: Option<&str>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_tkt"));
    command
        .current_dir(root)
        .args(args)
        .env("DO_NOT_TRACK", "1")
        .env("TKT_NO_USER_CONFIG", "1")
        .env("TKT_UPDATE_CHECK", "0")
        .env_remove("TKT_DEBUG")
        .env_remove("TKT_AGENT")
        .env_remove("CREW_ENV")
        .env_remove("TKT_CONTEXT");
    if let Some(agent) = agent {
        command.env("TKT_AGENT", agent);
    }
    command.output().unwrap()
}

fn ok(root: &Path, args: &[&str], agent: Option<&str>) -> String {
    let output = cli(root, args, agent);
    assert!(output.status.success(), "{args:?}: {output:?}");
    String::from_utf8(output.stdout).unwrap()
}

fn repo() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let initialized = Command::new("git")
        .args(["init", "-q"])
        .current_dir(dir.path())
        .env("DO_NOT_TRACK", "1")
        .output()
        .unwrap();
    assert!(initialized.status.success(), "{initialized:?}");
    std::fs::create_dir(dir.path().join(".tickets")).unwrap();
    std::fs::write(
        dir.path().join(".tickets/config.toml"),
        "[push]\nenabled = false\n[machine]\ncapabilities = \"gpu\"\n",
    )
    .unwrap();
    dir
}

fn ticket(root: &Path, id: &str, title: &str, fields: &str) {
    std::fs::write(
        root.join(format!(".tickets/{id}-test.md")),
        format!(
            "---\nid: \"{id}\"\ntitle: \"{title}\"\nstatus: open\nblocked_by: []\n{fields}---\n"
        ),
    )
    .unwrap();
}

fn rows(output: &[u8]) -> Vec<Value> {
    String::from_utf8_lossy(output)
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

fn ids(output: &[u8]) -> Vec<String> {
    rows(output)
        .into_iter()
        .map(|row| row["id"].as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn priority_thresholds_include_unset_as_medium_and_preserve_order_and_schema() {
    let dir = repo();
    let root = dir.path();
    for (id, title, priority) in [
        ("01", "Low first", Some("low")),
        ("02", "Urgent", Some("urgent")),
        ("03", "High", Some("high")),
        ("04", "Medium", Some("medium")),
        ("05", "Unset", None),
        ("06", "Low second", Some("low")),
    ] {
        let fields = priority
            .map(|priority| format!("priority: {priority}\n"))
            .unwrap_or_default();
        ticket(root, id, title, &fields);
    }

    let unfiltered = rows(&cli(root, &["ready", "--json"], None).stdout);
    assert_eq!(
        unfiltered
            .iter()
            .map(|row| row["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["02", "03", "04", "05", "01", "06"]
    );
    for (threshold, expected) in [
        ("urgent", vec!["02"]),
        ("high", vec!["02", "03"]),
        ("medium", vec!["02", "03", "04", "05"]),
        ("low", vec!["02", "03", "04", "05", "01", "06"]),
    ] {
        let output = cli(root, &["ready", "--priority", threshold, "--json"], None);
        assert!(output.status.success(), "{threshold}: {output:?}");
        assert_eq!(ids(&output.stdout), expected, "{threshold}");
        if threshold == "low" {
            assert_eq!(rows(&output.stdout), unfiltered);
        }
    }

    let human = ok(root, &["ready", "--priority", "high"], None);
    assert!(human.contains("Urgent"), "{human}");
    assert!(human.contains("High"), "{human}");
    assert!(!human.contains("Medium"), "{human}");
    assert!(!human.contains("Unset"), "{human}");
    assert!(!human.contains("Low first"), "{human}");

    let query = cli(root, &["query", "--priority", "medium"], None);
    assert!(query.status.success(), "{query:?}");
    assert_eq!(ids(&query.stdout), ["04"]);
}

#[test]
fn threshold_composes_with_parent_context_capability_eligibility_and_ownership() {
    let dir = repo();
    let root = dir.path();
    ticket(root, "01", "Parent container", "priority: urgent\n");
    ticket(
        root,
        "02",
        "Child high",
        "priority: high\nparent: \"01\"\ntags: [alpha]\n",
    );
    ticket(
        root,
        "03",
        "Other context",
        "priority: urgent\nparent: \"01\"\ntags: [beta]\n",
    );
    ticket(
        root,
        "04",
        "Below threshold",
        "priority: medium\nparent: \"01\"\ntags: [alpha]\n",
    );
    ticket(
        root,
        "05",
        "Root urgent",
        "priority: urgent\ntags: [alpha]\n",
    );
    ticket(
        root,
        "06",
        "Missing capability",
        "priority: high\ntags: [alpha]\nrequires: [missing]\n",
    );
    ticket(
        root,
        "07",
        "Restricted",
        "priority: high\ntags: [alpha]\nagents: [codex]\n",
    );
    ticket(
        root,
        "08",
        "Other owner",
        "priority: high\ntags: [alpha]\nassignee: \"codex\"\n",
    );
    ticket(
        root,
        "09",
        "Owned high",
        "priority: high\nparent: \"01\"\ntags: [alpha]\nassignee: \"kimi\"\n",
    );
    ticket(
        root,
        "10",
        "Excluded urgent",
        "priority: urgent\nparent: \"01\"\ntags: [alpha]\nagents: [claude]\n",
    );
    std::fs::write(
        root.join(".tickets/11-test.md"),
        "---\nid: \"11\"\ntitle: \"WIP low\"\nstatus: in_progress\nblocked_by: []\npriority: low\nparent: \"01\"\ntags: [alpha]\n---\n",
    )
    .unwrap();
    ok(root, &["context", "alpha"], None);

    let parent = cli(
        root,
        &["ready", "--parent", "01", "--priority", "high", "--json"],
        Some("kimi"),
    );
    assert!(parent.status.success(), "{parent:?}");
    assert_eq!(ids(&parent.stdout), ["02", "09"]);

    let frontier = cli(
        root,
        &["ready", "--priority", "high", "--json"],
        Some("kimi"),
    );
    assert!(frontier.status.success(), "{frontier:?}");
    assert_eq!(ids(&frontier.stdout), ["05", "02", "09"]);

    let human = ok(
        root,
        &["ready", "--parent", "01", "--priority", "high"],
        Some("kimi"),
    );
    assert!(human.contains("Child high"), "{human}");
    assert!(human.contains("Owned high"), "{human}");
    assert!(!human.contains("Below threshold"), "{human}");
    assert!(human.contains("In progress (1):"), "{human}");
    assert!(human.contains("WIP low"), "{human}");
}

#[test]
fn empty_results_and_invalid_thresholds_keep_output_contracts() {
    let dir = repo();
    let root = dir.path();
    ticket(root, "01", "Medium only", "priority: medium\n");

    let empty = cli(root, &["ready", "--priority", "urgent", "--json"], None);
    assert!(empty.status.success(), "{empty:?}");
    assert!(empty.stdout.is_empty());
    let human = ok(root, &["ready", "--priority", "urgent"], None);
    assert!(human.contains("No tickets ready."), "{human}");

    let invalid = cli(
        root,
        &["ready", "--priority", "urgentish", "-o", "json"],
        None,
    );
    assert_eq!(invalid.status.code(), Some(1), "{invalid:?}");
    assert!(invalid.stdout.is_empty());
    let stderr = String::from_utf8(invalid.stderr).unwrap();
    let error: Value = serde_json::from_str(stderr.lines().last().unwrap()).unwrap();
    assert_eq!(error["error"]["kind"], "validation");
    assert!(error["error"]["message"]
        .as_str()
        .unwrap()
        .contains("expected urgent/high/medium/low"));
}

#[test]
fn capabilities_and_guidance_explain_threshold_versus_exact_query_matching() {
    let dir = repo();
    let root = dir.path();
    let manifest: Value = serde_json::from_str(&ok(root, &["capabilities"], None)).unwrap();
    let ready = &manifest["commands"]["ready"];
    assert_eq!(
        ready["inputSchema"]["properties"]["priority"]["enum"],
        serde_json::json!(["urgent", "high", "medium", "low"])
    );
    let behavior = ready["output"]["description"].as_str().unwrap();
    assert!(behavior.contains("threshold"), "{behavior}");
    assert!(behavior.contains("unset"), "{behavior}");
    assert!(behavior.contains("query"), "{behavior}");
    assert!(manifest["commands"]["query"]["output"]["description"]
        .as_str()
        .unwrap()
        .contains("exact stored-priority"));

    ok(root, &["init", "--agent-only", "--target", "agents"], None);
    let snippet = std::fs::read_to_string(root.join("AGENTS.md")).unwrap();
    assert!(snippet.contains("ready --priority high"), "{snippet}");
    assert!(snippet.contains("threshold"), "{snippet}");

    for path in [
        "README.md",
        "skills/tkt-core/SKILL.md",
        "skills/tkt-core/references/commands.md",
        "docs/agents/tkt-contract.md",
    ] {
        let text =
            std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap();
        assert!(text.contains("ready --priority high"), "{path}");
        assert!(text.contains("threshold"), "{path}");
        assert!(text.contains("query --priority"), "{path}");
    }
}

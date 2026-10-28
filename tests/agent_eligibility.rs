use std::path::Path;
use std::process::{Command, Output};

fn run(dir: &Path, args: &[&str], agent: Option<&str>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_tkt"));
    command
        .current_dir(dir)
        .args(args)
        .env("DO_NOT_TRACK", "1")
        .env("TKT_NO_USER_CONFIG", "1")
        .env_remove("TKT_AGENT")
        .env_remove("CREW_ENV");
    if let Some(agent) = agent {
        command.env("TKT_AGENT", agent);
    }
    command.output().unwrap()
}
fn ok(dir: &Path, args: &[&str]) -> String {
    let result = run(dir, args, None);
    assert!(
        result.status.success(),
        "{:?}: {}",
        args,
        String::from_utf8_lossy(&result.stderr)
    );
    String::from_utf8(result.stdout).unwrap()
}
fn repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for args in [
        vec!["init", "-q"],
        vec!["config", "user.name", "test"],
        vec!["config", "user.email", "test@test"],
    ] {
        assert!(Command::new("git")
            .current_dir(dir.path())
            .args(args)
            .status()
            .unwrap()
            .success());
    }
    std::fs::create_dir(dir.path().join(".tickets")).unwrap();
    std::fs::write(
        dir.path().join(".tickets/config.toml"),
        "[push]\nenabled = false\n[machine]\ncapabilities = \"gpu\"\n",
    )
    .unwrap();
    assert!(Command::new("git")
        .current_dir(dir.path())
        .args(["add", "."])
        .status()
        .unwrap()
        .success());
    assert!(Command::new("git")
        .current_dir(dir.path())
        .args(["commit", "-qm", "seed"])
        .status()
        .unwrap()
        .success());
    dir
}
fn ticket(dir: &Path, id: &str, fields: &str) {
    std::fs::write(dir.join(format!(".tickets/{id}-test.md")), format!("---\nid: \"{id}\"\ntitle: \"Ticket {id}\"\nstatus: open\nblocked_by: []\n{fields}---\n")).unwrap();
}
fn ids(output: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(output)
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
fn agent_eligibility_frontier_composes_and_query_stays_scoped() {
    let dir = repo();
    let dir = dir.path();
    ticket(dir, "01", "tags: [one]\n");
    ticket(
        dir,
        "02",
        "agents: [codex]\ntags: [one]\npriority: high\nrequires: [gpu]\n",
    );
    ticket(dir, "03", "agents: ['!codex']\ntags: [one]\n");
    ticket(dir, "04", "agents: [codex]\ntags: [two]\n");
    ticket(
        dir,
        "05",
        "agents: [codex]\ntags: [one]\nrequires: [missing]\n",
    );
    std::fs::write(dir.join(".tickets/06-test.md"), "---\nid: 06\ntitle: blocked\nstatus: open\nblocked_by: [01]\nagents: [codex]\ntags: [one]\n---\n").unwrap();
    ok(dir, &["context", "one"]);
    let output = run(dir, &["ready", "--json"], None);
    assert_eq!(ids(&output.stdout), ["01"]);
    assert!(String::from_utf8_lossy(&output.stderr).contains("Hidden: 2"));
    assert_eq!(
        ids(&run(dir, &["ready", "--json"], Some("codex")).stdout),
        ["02", "01"]
    );
    assert_eq!(
        ids(&run(
            dir,
            &["--agent", "claude", "ready", "--json"],
            Some("codex")
        )
        .stdout),
        ["01", "03"]
    );
    let query = run(dir, &["query"], Some("codex"));
    assert_eq!(ids(&query.stdout), ["01", "02", "03", "05", "06"]);
    assert!(String::from_utf8_lossy(&query.stdout).contains("\"agents\":[\"!codex\"]"));
    assert!(ok(dir, &["--agent", "codex", "ready"]).contains("[agents: codex]"));
}
#[test]
fn agent_eligibility_claim_reasons_and_force() {
    let dir = repo();
    let dir = dir.path();
    for (id, fields, agent, reason) in [
        ("01", "agents: [codex]\n", None, "identify this invocation"),
        (
            "02",
            "agents: [claude]\n",
            Some("codex"),
            "not in this ticket's allow list",
        ),
        (
            "03",
            "agents: ['!codex']\n",
            Some("codex"),
            "explicitly excluded",
        ),
    ] {
        ticket(dir, id, fields);
        let output = run(dir, &["claim", id, "-o", "json"], agent);
        assert_eq!(output.status.code(), Some(1));
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(reason), "{stderr}");
        assert!(stderr.contains("gate_failed"));
        ok(dir, &["claim", id, "--force"]);
    }
    ticket(dir, "04", "agents: [codex]\n");
    ok(dir, &["--agent", "codex", "claim", "04"]);
}
#[test]
fn agent_eligibility_creation_edit_and_discovery() {
    let dir = repo();
    let dir = dir.path();
    ok(dir, &["new", "allow", "--agents", "kiro,claude"]);
    ok(
        dir,
        &[
            "batch",
            "first",
            "second",
            "--exclude-agents",
            "kiro,antigravity",
        ],
    );
    let query = ok(dir, &["query"]);
    assert!(query.contains("\"agents\":[\"claude\",\"kiro\"]"));
    assert_eq!(
        query
            .matches("\"agents\":[\"!antigravity\",\"!kiro\"]")
            .count(),
        2
    );
    ok(dir, &["edit", "01", "--agents", ""]);
    ok(dir, &["edit", "02", "--exclude-agents", ""]);
    for command in [vec!["new", "bad"], vec!["batch", "bad"], vec!["edit", "01"]] {
        let mut args = command;
        args.extend(["--agents", "codex", "--exclude-agents", "claude"]);
        assert!(!run(dir, &args, None).status.success());
    }
    for invalid in ["Codex", "a_b", "!codex", "codex,codex", "codex,"] {
        assert!(
            !run(dir, &["new", "bad", "--agents", invalid], None)
                .status
                .success(),
            "{invalid}"
        );
    }
    let output = ok(dir, &["capabilities"]);
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(value["global_flags"]["agent"]["environment"], "TKT_AGENT");
    for command in ["new", "batch", "edit"] {
        let properties = &value["commands"][command]["inputSchema"]["properties"];
        assert!(properties.get("agents").is_some());
        assert!(properties.get("exclude_agents").is_some());
    }
    assert!(value["commands"]["claim"]["inputSchema"]["properties"]
        .get("force")
        .is_some());
}

#[test]
fn agent_eligibility_identity_boundary_and_init_guidance() {
    let dir = repo();
    let dir = dir.path();
    ticket(dir, "01", "agents: [future-agent-7]\n");
    assert_eq!(
        ids(&run(dir, &["ready", "--json"], Some("future-agent-7")).stdout),
        ["01"]
    );
    let invalid = run(dir, &["ready", "-o", "json"], Some("Bad_Id"));
    assert_eq!(invalid.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&invalid.stderr);
    let error: serde_json::Value = serde_json::from_str(stderr.lines().last().unwrap()).unwrap();
    assert_eq!(error["error"]["kind"], "validation");
    // An explicit valid flag overrides even an invalid ambient identity.
    assert!(
        run(dir, &["ready", "--agent", "future-agent-7"], Some("Bad_Id"))
            .status
            .success()
    );
    assert!(!ok(dir, &["config", "--show"]).contains("TKT_AGENT"));
    std::fs::create_dir_all(dir.join(".github")).unwrap();
    for (target, file) in [
        ("agents", "AGENTS.md"),
        ("claude", "CLAUDE.md"),
        ("cursor", ".cursor/rules/tkt.mdc"),
        ("kiro", ".kiro/steering/tkt.md"),
        ("copilot", ".github/copilot-instructions.md"),
        ("opencode", "AGENTS.md"),
        ("windsurf", ".windsurf/rules/tkt.md"),
    ] {
        ok(dir, &["init", "--agent-only", "--target", target]);
        let snippet = std::fs::read_to_string(dir.join(file)).unwrap();
        assert!(snippet.contains("TKT_AGENT"), "{target}");
        assert!(snippet.contains("--exclude-agents"), "{target}");
    }
}

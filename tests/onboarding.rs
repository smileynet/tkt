use std::path::Path;
use std::process::Command;

fn cli(dir: &Path, args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_tkt"))
        .current_dir(dir)
        .args(args)
        .env("DO_NOT_TRACK", "1")
        .env("TKT_NO_USER_CONFIG", "1")
        .env("TKT_UPDATE_CHECK", "0")
        .env_remove("TKT_DEBUG")
        .output()
        .unwrap();
    assert!(output.status.success(), "{args:?}: {output:?}");
    String::from_utf8(output.stdout).unwrap()
}

fn repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let init = Command::new("git")
        .args(["init", "-q"])
        .env("DO_NOT_TRACK", "1")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(init.status.success());
    dir
}

#[test]
fn help_points_to_structured_command_discovery() {
    let help = cli(Path::new(env!("CARGO_MANIFEST_DIR")), &["--help"]);
    assert!(
        help.contains("For machine-readable command schemas: tkt capabilities -o json"),
        "{help}"
    );
}

#[test]
fn init_recommends_project_instructions_and_separate_workflow_skills() {
    let dir = repo();
    let output = cli(dir.path(), &["init"]);
    assert!(output.contains("Agent setup recommendations:"), "{output}");
    assert!(output.contains("tkt init --write"));
    assert!(output.contains("install the tkt plugin separately"));
    assert!(output.contains("CLI installation does not include workflow skills"));
    assert!(output.contains("tkt capabilities -o json"));
    assert!(dir.path().join(".tickets/config.toml").exists());
}

#[test]
fn every_generated_instruction_target_retains_the_discovery_pointer() {
    let dir = repo();
    std::fs::create_dir(dir.path().join(".github")).unwrap();
    cli(dir.path(), &["init", "--all"]);
    for path in [
        "AGENTS.md",
        "CLAUDE.md",
        ".cursor/rules/tkt.mdc",
        ".kiro/steering/tkt.md",
        ".github/copilot-instructions.md",
        ".windsurf/rules/tkt.md",
    ] {
        let instructions = std::fs::read_to_string(dir.path().join(path)).unwrap();
        assert!(instructions.contains("tkt capabilities -o json"), "{path}");
        assert!(
            instructions.contains("Ticket files own lifecycle status and closure evidence"),
            "{path}"
        );
        assert!(
            instructions.contains("docs/plan.md owns intended human sequencing"),
            "{path}"
        );
        assert!(
            instructions.contains("explicit user request selects the ticket"),
            "{path}"
        );
    }
}

#[test]
fn pure_snippet_and_quiet_writes_do_not_gain_setup_chatter() {
    let dir = repo();
    let snippet = cli(dir.path(), &["init", "--agent-only"]);
    assert!(snippet.contains("tkt capabilities -o json"));
    assert!(!snippet.contains("Agent setup recommendations:"));
    assert!(!dir.path().join(".tickets").exists());
    let quiet = cli(dir.path(), &["init", "--write", "-q"]);
    assert!(quiet.is_empty(), "{quiet}");
}

#[test]
fn distributed_workflow_guidance_has_one_authority_and_no_provider_execution() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let work = std::fs::read_to_string(root.join("skills/tkt-work/SKILL.md")).unwrap();
    for removed in [
        "CREW_TICKET_SOURCES",
        "CREW_TICKET_SYNC",
        "glab issue",
        "gh issue close",
    ] {
        assert!(
            !work.contains(removed),
            "legacy provider behavior: {removed}"
        );
    }
    assert!(work.contains("Do not independently select the first `tkt ready` row"));
    assert!(!work.contains("plan is the single source of truth for work status"));
    for default in [
        "close.require_checked_acs = true",
        "close.require_resolution = false",
        "close.require_validation_criteria = false",
        "close.require_validation_evidence = \"warn\"",
    ] {
        assert!(work.contains(default), "missing actual default {default}");
    }
    for skill in ["tkt-orient", "tkt-work"] {
        let guidance =
            std::fs::read_to_string(root.join(format!("skills/{skill}/SKILL.md"))).unwrap();
        assert!(guidance.contains("Ticket files own status/evidence"));
        assert!(guidance.contains("plan owns") || guidance.contains("Plan owns"));
    }
}

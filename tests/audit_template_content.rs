use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;
use tempfile::TempDir;

fn repo() -> (TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().to_path_buf();
    git(&root, &["init", "-q"]);
    git(&root, &["config", "user.email", "test@example.invalid"]);
    git(&root, &["config", "user.name", "Test"]);
    std::fs::create_dir(root.join(".tickets")).unwrap();
    (temp, root)
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

fn write_ticket(root: &Path, id: &str, status: &str, fields: &str, body: &str) {
    let filename = format!(".tickets/{id}-fixture.md");
    std::fs::write(
        root.join(filename),
        format!(
            "---\nid: \"{id}\"\ntitle: \"Fixture {id}\"\nstatus: {status}\nblocked_by: []\n{fields}---\n\n{body}"
        ),
    )
    .unwrap();
}

fn cli(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tkt"))
        .args(args)
        .current_dir(root)
        .env("DO_NOT_TRACK", "1")
        .env("TKT_NO_USER_CONFIG", "1")
        .env("TKT_UPDATE_CHECK", "0")
        .env_remove("TKT_DEBUG")
        .output()
        .unwrap()
}

fn findings(output: &Output) -> Vec<Value> {
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    report["findings"].as_array().unwrap().clone()
}

fn files_for(findings: &[Value], rule: &str) -> Vec<String> {
    findings
        .iter()
        .filter(|finding| finding["rule"] == rule)
        .map(|finding| finding["file"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn deep_audit_classifies_actual_template_sections_not_quoted_text() {
    let (_temp, root) = repo();

    // Reproduces #176: this old marker is quoted as historical template text inside
    // substantive prose; the ticket's actual acceptance criterion is checked.
    write_ticket(
        &root,
        "176",
        "done",
        "validation_criteria:\n  - \"body input works\"\n",
        "# Body creation\n\n## What to build\n\nCapture content live at creation.\n\n## Acceptance criteria\n\n- [x] body input works\n\n## Context\n\nThe old template was `## What to build\\n\\nTBD` and `- [ ] TBD`; this ticket documents replacing it.\n\n## Resolution (2026-11-03)\n\nChanged the creation prompts.\n",
    );

    write_ticket(
        &root,
        "01",
        "done",
        "",
        "# Legacy prompt\n\n## What to build\n\nTBD\n\n## Acceptance criteria\n\n- [ ] Real criterion\n\n## Resolution (2026-11-03)\n\nNot left blank.\n",
    );
    write_ticket(
        &root,
        "02",
        "done",
        "",
        "# Legacy AC\n\n## What to build\n\nReal intent.\n\n## Acceptance criteria\n\n- [ ] TBD\n\n## Resolution (2026-11-03)\n\nNot left blank.\n",
    );
    write_ticket(
        &root,
        "03",
        "done",
        "",
        "# Current prompts\n\n## What to build\n\nDescribe the change, why it is needed, and link its source (--spec or a #NN reference).\n\n## Acceptance criteria\n\n- [ ] Define an observable result and how to verify it.\n\n## Resolution (2026-11-03)\n\nNot left blank.\n",
    );
    write_ticket(
        &root,
        "04",
        "done",
        "",
        "# Empty intent\n\n## What to build\n\n\n## Acceptance criteria\n\n- [x] Real criterion\n\n## Resolution (2026-11-03)\n\nNot left blank.\n",
    );
    write_ticket(
        &root,
        "05",
        "done",
        "",
        "# Empty criteria\n\n## What to build\n\nReal intent.\n\n## Acceptance criteria\n\n\n## Resolution (2026-11-03)\n\nNot left blank.\n",
    );
    write_ticket(
        &root,
        "06",
        "done",
        "",
        "# Checked legacy prompt\n\n## What to build\n\nReal intent.\n\n## Acceptance criteria\n\n- [x] TBD\n\n## Resolution (2026-11-03)\n\nNot left blank.\n",
    );
    write_ticket(
        &root,
        "07",
        "done",
        "",
        "# Checked current prompt\n\n## What to build\n\nReal intent.\n\n## Acceptance criteria\n\n- [x] Define an observable result and how to verify it.\n\n## Resolution (2026-11-03)\n\nNot left blank.\n",
    );
    write_ticket(
        &root,
        "16",
        "done",
        "",
        "# Mixed placeholder\n\n## What to build\n\nReal intent.\n\n## Acceptance criteria\n\n- [x] A real criterion.\n- [ ] TBD\n\n## Resolution (2026-11-03)\n\nNot left blank.\n",
    );
    write_ticket(
        &root,
        "08",
        "done",
        "",
        "# Fenced example\n\nThe following is a historical example only:\n\n```markdown\n## What to build\n\nTBD\n\n## Acceptance criteria\n\n- [ ] TBD\n```\n\n## Resolution (2026-11-03)\n\nImplemented the example parser.\n",
    );
    write_ticket(
        &root,
        "17",
        "done",
        "",
        "# Fenced inside section\n\n## What to build\n\nImplemented real behavior.\n\n## Acceptance criteria\n\n- [x] Real criterion.\n\n```markdown\n## Notes\n- [ ] TBD\n```\n\n## Resolution (2026-11-03)\n\nImplemented behavior.\n",
    );
    write_ticket(
        &root,
        "09",
        "done",
        "",
        "# Similar heading\n\n## What to build notes\n\nThis section discusses the string `- [ ] TBD` but is not the template heading.\n\n## Resolution (2026-11-03)\n\nDiscussed the example.\n",
    );
    for (id, disposition) in [
        ("10", "Migrated to #11."),
        ("11", "Superseded by #12."),
        ("12", "Moved to #13."),
    ] {
        write_ticket(
            &root,
            id,
            "done",
            "",
            &format!(
                "# Exempt {id}\n\n## What to build\n\nTBD\n\n## Acceptance criteria\n\n- [ ] TBD\n\n## Resolution (2026-11-03)\n\n{disposition}\n"
            ),
        );
    }
    write_ticket(
        &root,
        "13",
        "open",
        "",
        "# Open prompt\n\n## What to build\n\nTBD\n\n## Acceptance criteria\n\n- [ ] TBD\n",
    );
    write_ticket(
        &root,
        "14",
        "in_progress",
        "",
        "# WIP prompt\n\n## What to build\n\nDescribe the change, why it is needed, and link its source (--spec or a #NN reference).\n\n## Acceptance criteria\n\n- [ ] Define an observable result and how to verify it.\n",
    );
    write_ticket(
        &root,
        "15",
        "open",
        "",
        "# Filled example\n\n## What to build\n\nExplain how the parser handles the literal `- [ ] TBD` marker without treating it as a checklist item.\n\n## Acceptance criteria\n\n- [ ] A real behavior is verified.\n",
    );
    write_ticket(
        &root,
        "18",
        "open",
        "",
        "# Open fenced example\n\n## What to build\n\nImplement real behavior.\n\n## Acceptance criteria\n\n- [ ] A real criterion.\n\n```markdown\n## Acceptance criteria\n- [ ] TBD\n```\n",
    );
    write_ticket(
        &root,
        "19",
        "done",
        "",
        "# Fenced-only intent example\n\n## What to build\n\n```text\nTBD\n```\n\n## Acceptance criteria\n\n- [x] Real criterion.\n\n## Resolution (2026-11-03)\n\nImplemented behavior.\n",
    );
    write_ticket(
        &root,
        "20",
        "done",
        "",
        "# Fenced-only AC example\n\n## What to build\n\nReal behavior.\n\n## Acceptance criteria\n\n```markdown\n- [ ] TBD\n```\n\n## Resolution (2026-11-03)\n\nImplemented behavior.\n",
    );
    write_ticket(
        &root,
        "21",
        "done",
        "",
        "# Placeholder with fenced example\n\n## What to build\n\nTBD\n\n```text\nexample\n```\n\n## Acceptance criteria\n\n- [x] Real criterion.\n\n## Resolution (2026-11-03)\n\nImplemented behavior.\n",
    );
    write_ticket(
        &root,
        "22",
        "done",
        "",
        "# Empty intent at EOF\n\n## What to build",
    );
    write_ticket(
        &root,
        "23",
        "done",
        "",
        "# Empty AC before heading\n\n## What to build\n\nReal behavior.\n\n## Acceptance criteria\n## Resolution (2026-11-03)\n\nImplemented behavior.\n",
    );
    write_ticket(
        &root,
        "24",
        "done",
        "",
        "# Repeated intent\n\n## What to build\n\nReal behavior.\n\n## Context\n\nMore context.\n\n## What to build\n\nTBD\n\n## Acceptance criteria\n\n- [x] Real criterion.\n\n## Resolution (2026-11-03)\n\nImplemented behavior.\n",
    );
    write_ticket(
        &root,
        "25",
        "done",
        "",
        "# Invalid fence closer\n\n## What to build\n\nReal behavior.\n\n```markdown\n## What to build\nTBD\n```garbage\n~~~\n```\n\n## Acceptance criteria\n\n- [x] Real criterion.\n\n## Resolution (2026-11-03)\n\nImplemented behavior.\n",
    );

    git(&root, &["add", "."]);
    git(&root, &["commit", "-qm", "seed audit fixtures"]);

    let audit_findings = findings(&cli(&root, &["audit", "--deep"]));
    let template_files = files_for(&audit_findings, "template-only-closure");
    for id in [
        "01", "02", "03", "04", "05", "06", "07", "16", "21", "22", "23", "24",
    ] {
        assert!(
            template_files.iter().any(|file| file.starts_with(id)),
            "actual prompt/empty section {id} should remain flagged: {template_files:?}"
        );
    }
    for id in [
        "176", "08", "09", "10", "11", "12", "13", "14", "15", "17", "19", "20", "25",
    ] {
        assert!(
            !template_files.iter().any(|file| file.starts_with(id)),
            "quoted, fenced, exempt, non-done, or nonmatching content {id} must not be flagged: {template_files:?}"
        );
    }
    assert!(
        files_for(&audit_findings, "low-evidence-closure")
            .iter()
            .any(|file| file.starts_with("176")),
        "removing the template false positive must preserve the historical evidence warning"
    );

    let validation_findings = findings(&cli(&root, &["validate"]));
    let stub_files = files_for(&validation_findings, "stub-body");
    assert!(stub_files.iter().any(|file| file.starts_with("13")));
    assert!(stub_files.iter().any(|file| file.starts_with("14")));
    assert!(!stub_files.iter().any(|file| file.starts_with("15")));
    assert!(
        !stub_files.iter().any(|file| file.starts_with("18")),
        "a fenced placeholder and heading inside an actual open section are examples, not stubs"
    );
}

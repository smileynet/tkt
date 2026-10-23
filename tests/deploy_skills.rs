//! Dual-deploy guard in tools/deploy-skills.sh.
//!
//! With the tkt Claude Code plugin installed, a root Claude Code reads (any
//! `.claude/skills`, or a project `.agents/skills`) must not get copies too, or
//! every skill lists twice (`tkt-core` and `tkt:tkt-core`). Each test runs the
//! script against throwaway roots with a fake plugins dir
//! (`CLAUDE_CODE_PLUGIN_CACHE_DIR`), never a real harness root.

use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempDir;

const MANIFEST: &str = ".installed-by-tkt.json";

fn script() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tools/deploy-skills.sh")
}

/// A temp tree with `plugins/installed_plugins.json` listing `plugin` (if any).
fn sandbox(plugin: Option<&str>) -> TempDir {
    let dir = TempDir::new().unwrap();
    set_plugin(dir.path(), plugin);
    dir
}

fn set_plugin(base: &Path, plugin: Option<&str>) {
    let plugins = base.join("plugins");
    std::fs::create_dir_all(&plugins).unwrap();
    let ids = match plugin {
        Some(id) => format!("{{\"{id}\": [{{\"scope\": \"user\"}}]}}"),
        None => "{}".to_string(),
    };
    std::fs::write(
        plugins.join("installed_plugins.json"),
        format!("{{\"version\": 2, \"plugins\": {ids}}}"),
    )
    .unwrap();
}

fn deploy(base: &Path, args: &[&str]) -> String {
    let out = Command::new("bash")
        .arg(script())
        .args(args)
        .env("CLAUDE_CODE_PLUGIN_CACHE_DIR", base.join("plugins"))
        .env("CODEX_HOME", base)
        .env("DO_NOT_TRACK", "1")
        .output()
        .unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out.status.success(), "deploy-skills.sh failed:\n{text}");
    text
}

#[test]
fn plugin_installed_skips_claude_root() {
    let base = sandbox(Some("tkt@tkt"));
    let root = base.path().join(".claude/skills");
    let out = deploy(base.path(), &["--root", root.to_str().unwrap()]);
    assert!(
        out.contains("skipped: Claude plugin tkt@tkt is installed"),
        "{out}"
    );
    assert!(
        !root.join("tkt-core").exists(),
        "no copy where Claude reads"
    );
}

#[test]
fn plugin_installed_removes_earlier_copies_but_not_foreign_dirs() {
    let base = sandbox(None);
    let root = base.path().join(".claude/skills");
    let root_arg = root.to_str().unwrap();
    deploy(base.path(), &["--root", root_arg]);
    assert!(
        root.join("tkt-core").exists(),
        "precondition: copy installed"
    );
    std::fs::create_dir_all(root.join("someone-elses")).unwrap();

    set_plugin(base.path(), Some("tkt@tkt"));
    deploy(base.path(), &["--root", root_arg]);
    assert!(!root.join("tkt-core").exists(), "our earlier copy removed");
    assert!(!root.join(MANIFEST).exists(), "our manifest removed");
    assert!(root.join("someone-elses").exists(), "foreign dir untouched");
}

#[test]
fn plugin_installed_skips_project_agents_root() {
    let base = sandbox(Some("tkt@tkt"));
    let root = base.path().join("proj/.agents/skills");
    let out = deploy(base.path(), &["--root", root.to_str().unwrap()]);
    assert!(out.contains("skipped: Claude plugin"), "{out}");
    assert!(!root.join("tkt-core").exists());
}

#[test]
fn other_roots_install_and_allow_dual_overrides() {
    let base = sandbox(Some("tkt@tkt"));
    let kiro = base.path().join(".kiro/skills");
    deploy(base.path(), &["--root", kiro.to_str().unwrap()]);
    assert!(kiro.join("tkt-core").exists(), "non-Claude root unaffected");

    let claude = base.path().join(".claude/skills");
    deploy(
        base.path(),
        &["--root", claude.to_str().unwrap(), "--allow-dual"],
    );
    assert!(
        claude.join("tkt-core").exists(),
        "--allow-dual copies anyway"
    );
}

#[test]
fn another_plugin_does_not_trigger_the_guard() {
    let base = sandbox(Some("grey-matter@grey-matter"));
    let root = base.path().join(".claude/skills");
    deploy(base.path(), &["--root", root.to_str().unwrap()]);
    assert!(root.join("tkt-core").exists(), "only the tkt plugin counts");
}

#[test]
fn codex_plugin_removes_owned_copies_and_preserves_foreign_skills() {
    let base = sandbox(None);
    let root = base.path().join(".codex/skills");
    let args = ["--root", root.to_str().unwrap()];
    deploy(base.path(), &args);
    std::fs::create_dir_all(root.join("foreign")).unwrap();
    let config = base.path().join("config.toml");
    std::fs::write(&config, "[plugins.\"tkt@tkt\"]\nenabled = true\n").unwrap();
    assert!(deploy(base.path(), &args).contains("skipped: Codex plugin"));
    assert!(!root.join("tkt-core").exists());
    assert!(root.join("foreign").exists());
    deploy(
        base.path(),
        &["--root", root.to_str().unwrap(), "--allow-dual"],
    );
    assert!(root.join("tkt-core").exists());
    std::fs::write(&config, "[plugins.\"tkt@tkt\"]\nenabled = false\n").unwrap();
    deploy(base.path(), &args);
    assert!(root.join("tkt-core").exists());
    std::fs::write(&config, "[plugins.\"tkt@tkt\"]\nenabled = true\n").unwrap();
    let shared = base.path().join(".agents/skills");
    assert!(deploy(base.path(), &["--root", shared.to_str().unwrap()])
        .contains("skipped: Codex plugin"));
}

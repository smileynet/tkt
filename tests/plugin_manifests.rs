//! Plugin manifests must carry the crate version.
//!
//! Claude Code caches a plugin by its `version`, so a manifest left behind on a
//! release means installed users never receive the update. cargo-release bumps
//! all manifests (`pre-release-replacements` in release.toml); this test fails
//! if any manifest drifts from Cargo.toml.

use std::path::Path;

fn manifest_version(rel: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let text =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let json: serde_json::Value =
        serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()));
    json["version"]
        .as_str()
        .unwrap_or_else(|| panic!("{} has no string \"version\"", path.display()))
        .to_string()
}

#[test]
fn claude_plugin_version_matches_crate() {
    assert_eq!(
        manifest_version(".claude-plugin/plugin.json"),
        env!("CARGO_PKG_VERSION")
    );
}

#[test]
fn agy_plugin_version_matches_crate() {
    assert_eq!(manifest_version("plugin.json"), env!("CARGO_PKG_VERSION"));
}

#[test]
fn codex_plugin_version_matches_crate() {
    assert_eq!(
        manifest_version(".codex-plugin/plugin.json"),
        env!("CARGO_PKG_VERSION")
    );
}

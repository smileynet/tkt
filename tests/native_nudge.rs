use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn command(program: &str) -> Command {
    let mut child = Command::new(program);
    child
        .env("DO_NOT_TRACK", "1")
        .env("TKT_NO_USER_CONFIG", "1")
        .env("TKT_UPDATE_CHECK", "0")
        .env("TKT_DEBUG", "0")
        .env(
            "GIT_CONFIG_GLOBAL",
            if cfg!(windows) { "NUL" } else { "/dev/null" },
        )
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_COUNT", "0")
        .env_remove("GIT_CONFIG_PARAMETERS")
        .env_remove("TKT_AGENT")
        .env_remove("TKT_CONTEXT");
    child
}
fn git(root: &Path, args: &[&str]) {
    let out = command("git")
        .current_dir(root)
        .args(args)
        .output()
        .unwrap();
    assert!(out.status.success(), "{args:?}: {out:?}");
}
struct Fixture {
    _temp: tempfile::TempDir,
    root: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("repo Ω with spaces");
        std::fs::create_dir(&root).unwrap();
        git(&root, &["init", "-q", "-b", "main"]);
        git(&root, &["config", "user.name", "Native fixture"]);
        git(&root, &["config", "user.email", "fixture@example.invalid"]);
        git(&root, &["config", "maintenance.auto", "false"]);
        git(&root, &["config", "gc.auto", "0"]);
        git(&root, &["config", "core.autocrlf", "false"]);
        std::fs::create_dir(root.join(".tickets")).unwrap();
        std::fs::write(root.join(".tickets/config.toml"), "[push]\nenabled=false\n").unwrap();
        git(&root, &["add", "."]);
        git(&root, &["commit", "-qm", "fixture"]);
        Self { _temp: temp, root }
    }
    fn run(&self, args: &[&str]) -> Output {
        command(env!("CARGO_BIN_EXE_tkt"))
            .current_dir(&self.root)
            .env("XDG_CONFIG_HOME", self._temp.path().join("config"))
            .args(args)
            .output()
            .unwrap()
    }
    fn native(&self, profile: &str, action: &str) -> Value {
        let out = self.run(&[
            "-q",
            "-o",
            "json",
            "init",
            "--write-nudge",
            profile,
            "--nudge-action",
            action,
        ]);
        assert!(out.status.success(), "{out:?}");
        serde_json::from_slice(&out.stdout).unwrap()
    }
    fn json(&self, name: &str) -> Value {
        serde_json::from_slice(&std::fs::read(self.root.join(name)).unwrap()).unwrap()
    }
    fn write(&self, name: &str, value: &Value) {
        let path = self.root.join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, serde_json::to_string_pretty(value).unwrap()).unwrap();
    }
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
fn ordinary_init_and_all_previews_never_install_or_change_trust() {
    let f = Fixture::new();
    let out = f.run(&["init", "--all"]);
    assert!(out.status.success(), "{out:?}");
    assert!(!f.root.join(".claude/settings.json").exists());
    assert!(!f.root.join(".codex/hooks.json").exists());
    assert!(!f
        .root
        .join(".tickets/hooks/native-write-nudge.mjs")
        .exists());
    let before = snapshot(&f.root);
    let out = f.run(&[
        "--dry-run",
        "-q",
        "-o",
        "json",
        "init",
        "--write-nudge",
        "both",
    ]);
    assert!(out.status.success(), "{out:?}");
    let value: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(value["trust_changed"], false);
    assert_eq!(value["installed"], false);
    assert_eq!(snapshot(&f.root), before);
    for args in [
        vec!["init", "--nudge-action", "check"],
        vec!["init", "--write-nudge", "both", "--all"],
    ] {
        let out = f.run(&args);
        assert_eq!(out.status.code(), Some(2), "{out:?}");
        assert_eq!(snapshot(&f.root), before);
    }
}
#[test]
fn explicit_generation_preserves_foreign_settings_and_converges() {
    let f = Fixture::new();
    let foreign =
        json!({"matcher":"Write","hooks":[{"type":"command","command":"foreign-command"}]});
    let claude = json!({"permissions":{"allow":["Read(*)"]},"theme":"dark","hooks":{"PreToolUse":[foreign.clone()],"Stop":[{"hooks":[]}]}});
    let codex = json!({"description":"foreign settings","hooks":{"PreToolUse":[foreign.clone()],"Stop":[]}});
    f.write(".claude/settings.json", &claude);
    f.write(".codex/hooks.json", &codex);
    std::fs::write(
        f.root.join(".codex/config.toml"),
        "# foreign config remains byte-identical\n",
    )
    .unwrap();
    let before_git = snapshot(&f.root.join(".git"));
    let value = f.native("both", "install");
    assert_eq!(value["installed"], true);
    assert_eq!(value["trust_changed"], false);
    let c = f.json(".claude/settings.json");
    assert_eq!(c["permissions"], claude["permissions"]);
    assert_eq!(c["hooks"]["PreToolUse"][0], foreign);
    assert_eq!(c["hooks"]["Stop"], claude["hooks"]["Stop"]);
    let handler = &c["hooks"]["PreToolUse"][1]["hooks"][0];
    assert_eq!(handler["command"], "node");
    assert_eq!(
        handler["args"][0],
        "${CLAUDE_PROJECT_DIR}/.tickets/hooks/native-write-nudge.mjs"
    );
    assert_eq!(handler["onFailure"], "continue");
    let c = f.json(".codex/hooks.json");
    assert_eq!(c["description"], codex["description"]);
    assert_eq!(c["hooks"]["PreToolUse"][0], foreign);
    let group = &c["hooks"]["PreToolUse"][1];
    assert_eq!(group["matcher"], "^apply_patch$");
    assert!(group["hooks"][0]["commandWindows"]
        .as_str()
        .unwrap()
        .contains("Join-Path"));
    assert_eq!(group["hooks"][0]["timeout"], 3);
    assert_eq!(
        std::fs::read_to_string(f.root.join(".codex/config.toml")).unwrap(),
        "# foreign config remains byte-identical\n"
    );
    assert_eq!(snapshot(&f.root.join(".git")), before_git);
    let before = snapshot(&f.root);
    assert_eq!(f.native("both", "install")["changed"], false);
    assert_eq!(snapshot(&f.root), before);
    assert_eq!(f.native("both", "check")["installed"], true);
    assert_eq!(snapshot(&f.root), before);
    let out = f.run(&[
        "--dry-run",
        "init",
        "--write-nudge",
        "both",
        "--nudge-action",
        "remove",
    ]);
    assert!(out.status.success(), "{out:?}");
    assert_eq!(snapshot(&f.root), before);
    f.native("both", "remove");
    assert_eq!(f.json(".claude/settings.json"), claude);
    assert_eq!(f.json(".codex/hooks.json"), codex);
    assert!(f
        .root
        .join(".tickets/hooks/native-write-nudge.mjs")
        .is_file());
    assert_eq!(
        f.json(".tickets/hooks/native-write-nudge.json")["profiles"],
        json!([])
    );
    let before = snapshot(&f.root);
    assert_eq!(f.native("both", "remove")["changed"], false);
    assert_eq!(snapshot(&f.root), before);
}
#[test]
fn check_is_read_only_and_profiles_can_be_installed_and_removed_independently() {
    let f = Fixture::new();
    let before = snapshot(&f.root);
    let out = f.run(&[
        "init",
        "--write-nudge",
        "both",
        "--nudge-action",
        "check",
        "-o",
        "json",
    ]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    assert_eq!(snapshot(&f.root), before);
    f.native("claude", "install");
    assert!(!f.root.join(".codex/hooks.json").exists());
    f.native("codex", "install");
    f.native("claude", "remove");
    assert_eq!(f.native("codex", "check")["installed"], true);
    let out = f.run(&["init", "--write-nudge", "claude", "--nudge-action", "check"]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
}
#[test]
fn owned_drift_duplicates_and_unowned_collisions_are_refused_without_writes() {
    for mode in ["helper", "group", "duplicate", "missing", "owner"] {
        let f = Fixture::new();
        f.native("both", "install");
        match mode {
            "helper" => std::fs::write(
                f.root.join(".tickets/hooks/native-write-nudge.mjs"),
                "// user change\n",
            )
            .unwrap(),
            "owner" => f.write(
                ".tickets/hooks/native-write-nudge.json",
                &json!({"version":2,"kind":"tkt-native-write-nudge","profiles":[]}),
            ),
            _ => {
                let mut config = f.json(".claude/settings.json");
                let groups = config["hooks"]["PreToolUse"].as_array_mut().unwrap();
                match mode {
                    "group" => groups[0]["hooks"][0]["timeout"] = json!(99),
                    "duplicate" => groups.push(groups[0].clone()),
                    _ => {
                        groups.clear();
                    }
                }
                f.write(".claude/settings.json", &config);
            }
        }
        let before = snapshot(&f.root);
        let out = f.run(&["init", "--write-nudge", "both"]);
        assert!(!out.status.success(), "{mode}: {out:?}");
        assert_eq!(snapshot(&f.root), before);
    }
    let f = Fixture::new();
    f.write(".claude/settings.json",&json!({"hooks":{"PreToolUse":[{"matcher":"Write","hooks":[{"command":"node native-write-nudge.mjs"}]}]}}));
    let before = snapshot(&f.root);
    let out = f.run(&["init", "--write-nudge", "claude"]);
    assert!(!out.status.success(), "{out:?}");
    assert_eq!(snapshot(&f.root), before);
}
#[test]
fn all_targets_are_preflighted_before_any_mutation() {
    for value in [
        "{",
        "[]",
        "{\"hooks\":[]}",
        "{\"hooks\":{\"PreToolUse\":{}}}",
    ] {
        let f = Fixture::new();
        std::fs::create_dir(f.root.join(".codex")).unwrap();
        std::fs::write(f.root.join(".codex/hooks.json"), value).unwrap();
        let before = snapshot(&f.root);
        let out = f.run(&["init", "--write-nudge", "both"]);
        assert!(!out.status.success(), "{out:?}");
        assert_eq!(snapshot(&f.root), before);
    }
    let f = Fixture::new();
    std::fs::create_dir(f.root.join(".claude")).unwrap();
    std::fs::write(
        f.root.join(".claude/settings.json"),
        vec![b' '; 1024 * 1024 + 1],
    )
    .unwrap();
    let before = snapshot(&f.root);
    let out = f.run(&["init", "--write-nudge", "claude"]);
    assert!(!out.status.success(), "{out:?}");
    assert_eq!(snapshot(&f.root), before);
}
#[test]
fn linked_parent_and_unowned_helper_are_preserved() {
    let f = Fixture::new();
    let other = f._temp.path().join("foreign");
    std::fs::create_dir(&other).unwrap();
    std::fs::write(other.join("keep"), "Foreign content").unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(&other, f.root.join(".claude")).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(&other, f.root.join(".claude")).unwrap();
    let before = snapshot(&other);
    let out = f.run(&["init", "--write-nudge", "claude"]);
    assert!(!out.status.success(), "{out:?}");
    assert_eq!(snapshot(&other), before);
    assert!(!f.root.join(".tickets/hooks").exists());
    let f = Fixture::new();
    std::fs::create_dir(f.root.join(".tickets/hooks")).unwrap();
    std::fs::write(
        f.root.join(".tickets/hooks/native-write-nudge.mjs"),
        include_str!("../assets/native-write-nudge.mjs"),
    )
    .unwrap();
    let before = snapshot(&f.root);
    let out = f.run(&["init", "--write-nudge", "claude"]);
    assert!(!out.status.success(), "{out:?}");
    assert_eq!(snapshot(&f.root), before);
}

#[test]
fn generated_launchers_reach_the_helper_from_nested_directories() {
    use std::io::Write;
    use std::process::Stdio;
    let f = Fixture::new();
    f.native("both", "install");
    let nested = f.root.join("nested directory");
    std::fs::create_dir(&nested).unwrap();
    let config = f.json(".codex/hooks.json");
    let handler = &config["hooks"]["PreToolUse"][0]["hooks"][0];
    #[cfg(windows)]
    let (launcher, flag, line) = (
        "powershell.exe",
        "-Command",
        handler["commandWindows"].as_str().unwrap(),
    );
    #[cfg(not(windows))]
    let (launcher, flag, line) = ("sh", "-c", handler["command"].as_str().unwrap());
    let mut cmd = command(launcher);
    #[cfg(windows)]
    cmd.args([
        "-NoLogo",
        "-NoProfile",
        "-NonInteractive",
        "-WindowStyle",
        "Hidden",
    ]);
    let mut child = cmd
        .arg(flag)
        .arg(line)
        .current_dir(&nested)
        .env_remove("NODE_OPTIONS")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(br#"{"hook_event_name":"PreToolUse","tool_name":"apply_patch","tool_input":{"command":"*** Begin Patch\n*** Update File: .tickets/01-work.md\n*** End Patch"}}"#).unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "{out:?}");
    let value: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(value["hookSpecificOutput"]["additionalContext"].is_string());
    let config = f.json(".claude/settings.json");
    let handler = &config["hooks"]["PreToolUse"][0]["hooks"][0];
    let args: Vec<_> = handler["args"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| {
            value
                .as_str()
                .unwrap()
                .replace("${CLAUDE_PROJECT_DIR}", f.root.to_str().unwrap())
        })
        .collect();
    let mut child = command(handler["command"].as_str().unwrap())
        .args(args)
        .current_dir(&nested)
        .env_remove("NODE_OPTIONS")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(br#"{"hook_event_name":"PreToolUse","tool_name":"Write","tool_input":{"file_path":".tickets/01-work.md"}}"#).unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "{out:?}");
    let value: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(value["hookSpecificOutput"]["additionalContext"].is_string());
}

#[test]
fn duplicate_json_keys_at_any_depth_and_in_receipts_are_preserved_by_refusal() {
    for target in [".claude/settings.json", ".codex/hooks.json"] {
        for text in [
            r#"{"theme":"first","theme":"second"}"#,
            r#"{"foreign":{"value":1,"value":2}}"#,
            r#"{"foreign":[{"value":1,"\u0076alue":2}]}"#,
        ] {
            let f = Fixture::new();
            let path = f.root.join(target);
            std::fs::create_dir(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
            let before = snapshot(&f.root);
            let out = f.run(&["init", "--write-nudge", "both"]);
            assert!(!out.status.success(), "{out:?}");
            assert!(
                String::from_utf8_lossy(&out.stderr).contains("duplicate object key"),
                "{out:?}"
            );
            assert_eq!(snapshot(&f.root), before);
        }
    }
    let f = Fixture::new();
    f.native("both", "install");
    std::fs::write(f.root.join(".tickets/hooks/native-write-nudge.json"),r#"{"kind":"tkt-native-write-nudge","version":1,"profiles":[],"profiles":["claude","codex"]}"#).unwrap();
    let before = snapshot(&f.root);
    let out = f.run(&["init", "--write-nudge", "both", "--nudge-action", "remove"]);
    assert!(!out.status.success(), "{out:?}");
    assert_eq!(snapshot(&f.root), before);
}

#[test]
fn foreign_numbers_are_exact_or_refused_without_writes() {
    for number in [
        "0.10000000000000001",
        "1.234567890123456789",
        "18446744073709551616",
        "1e-1000",
    ] {
        let f = Fixture::new();
        std::fs::create_dir(f.root.join(".claude")).unwrap();
        std::fs::write(
            f.root.join(".claude/settings.json"),
            format!("{{\"foreign\":{number}}}"),
        )
        .unwrap();
        let before = snapshot(&f.root);
        let out = f.run(&["init", "--write-nudge", "claude"]);
        assert!(!out.status.success(), "{number}: {out:?}");
        assert_eq!(snapshot(&f.root), before);
    }
    let f = Fixture::new();
    std::fs::create_dir(f.root.join(".claude")).unwrap();
    std::fs::write(
        f.root.join(".claude/settings.json"),
        r#"{"fraction":0.1000,"decimal":1.2300e2,"negative_zero":-0.0,"big":18446744073709551615}"#,
    )
    .unwrap();
    f.native("claude", "install");
    let value = f.json(".claude/settings.json");
    assert_eq!(value["fraction"], json!(0.1));
    assert_eq!(value["decimal"], json!(123.0));
    assert_eq!(value["big"].as_u64(), Some(u64::MAX));
    assert!(
        std::fs::read_to_string(f.root.join(".claude/settings.json"))
            .unwrap()
            .contains("-0.0")
    );
}

#[test]
fn near_limit_outputs_remain_checkable_and_removable_or_refuse_before_writing() {
    const LIMIT: usize = 1024 * 1024;
    for (profile, path) in [
        ("claude", ".claude/settings.json"),
        ("codex", ".codex/hooks.json"),
    ] {
        let f = Fixture::new();
        let file = f.root.join(path);
        std::fs::create_dir(file.parent().unwrap()).unwrap();
        std::fs::write(
            &file,
            format!("{{\"padding\":\"{}\"}}", "x".repeat(LIMIT - 2048)),
        )
        .unwrap();
        f.native(profile, "install");
        assert!(std::fs::metadata(&file).unwrap().len() <= LIMIT as u64);
        f.native(profile, "check");
        f.native(profile, "remove");
        assert!(std::fs::metadata(&file).unwrap().len() <= LIMIT as u64);
    }
    let f = Fixture::new();
    std::fs::create_dir(f.root.join(".codex")).unwrap();
    std::fs::write(
        f.root.join(".codex/hooks.json"),
        format!("{{\"padding\":\"{}\"}}", "x".repeat(LIMIT - 128)),
    )
    .unwrap();
    let before = snapshot(&f.root);
    let out = f.run(&["init", "--write-nudge", "both"]);
    assert!(!out.status.success(), "{out:?}");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("would exceed"),
        "{out:?}"
    );
    assert_eq!(snapshot(&f.root), before);
    let f = Fixture::new();
    std::fs::create_dir(f.root.join(".claude")).unwrap();
    std::fs::write(
        f.root.join(".claude/settings.json"),
        format!(
            "{{\"values\":[{}]}}",
            std::iter::repeat_n("0", 150000)
                .collect::<Vec<_>>()
                .join(",")
        ),
    )
    .unwrap();
    let before = snapshot(&f.root);
    let out = f.run(&["init", "--write-nudge", "claude"]);
    assert!(!out.status.success(), "{out:?}");
    assert_eq!(snapshot(&f.root), before);
}

#[test]
fn linked_ticket_store_cannot_redirect_generation_to_another_git_repository() {
    let f = Fixture::new();
    let other = f._temp.path().join("other repository");
    std::fs::create_dir(&other).unwrap();
    git(&other, &["init", "-q", "-b", "main"]);
    std::fs::create_dir(other.join(".tickets")).unwrap();
    std::fs::write(other.join(".tickets/keep"), "Foreign ticket store").unwrap();
    std::fs::remove_file(f.root.join(".tickets/config.toml")).unwrap();
    std::fs::remove_dir(f.root.join(".tickets")).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(other.join(".tickets"), f.root.join(".tickets")).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(other.join(".tickets"), f.root.join(".tickets")).unwrap();
    let before = snapshot(&other);
    let out = f.run(&["init", "--write-nudge", "both"]);
    assert!(!out.status.success(), "{out:?}");
    assert_eq!(snapshot(&other), before);
    assert!(!f.root.join(".claude").exists());
    assert!(!f.root.join(".codex").exists());
}

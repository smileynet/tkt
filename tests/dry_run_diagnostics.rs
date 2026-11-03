use std::path::Path;
use std::process::{Command, Output};

fn cli(root: &Path, state: &Path, mode: &str, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tkt"))
        .args(args)
        .current_dir(root)
        .env("DO_NOT_TRACK", "1")
        .env("TKT_NO_USER_CONFIG", "1")
        .env("TKT_UPDATE_CHECK", "0")
        .env("TKT_DEBUG", mode)
        .env_remove("TKT_DEBUG_FORMAT")
        .env("XDG_CONFIG_HOME", root.join("private-config"))
        .env("XDG_STATE_HOME", state)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env(
            "GIT_CONFIG_GLOBAL",
            if cfg!(windows) { "NUL" } else { "/dev/null" },
        )
        .output()
        .unwrap()
}

#[test]
fn dry_run_diagnostics_preserve_absent_and_existing_state_for_global_and_local_modes() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let init = Command::new("git")
        .args(["init", "-q"])
        .arg(root)
        .env("DO_NOT_TRACK", "1")
        .output()
        .unwrap();
    assert!(init.status.success());
    std::fs::create_dir(root.join(".tickets")).unwrap();
    std::fs::write(root.join(".tickets/01-seed.md"), "---\nid: \"01\"\ntitle: \"Seed\"\nstatus: done\nblocked_by: []\n---\n\n## Resolution\n\nSeeded a completed fixture with durable recorded work.\n").unwrap();
    for (index, args) in [
        vec!["--dry-run", "config", "--set", "debug=true"],
        vec!["validate", "--fix", "--dry-run"],
        vec!["rebase", "--dry-run"],
        vec!["--dry-run", "new", "example", "--title", "Example"],
        vec!["--dry-run", "hooks", "install"],
        vec!["--dry-run", "sync-plan", "--generate", "projection.md"],
    ]
    .iter()
    .enumerate()
    {
        for existing in [false, true] {
            let state = root.join(format!("state-{index}-{existing}"));
            let log = state.join("tkt/debug.log");
            if existing {
                std::fs::create_dir_all(log.parent().unwrap()).unwrap();
                std::fs::write(&log, "existing log\n").unwrap();
            }
            let out = cli(root, &state, "log", args);
            assert!(out.status.success(), "{args:?}: {out:?}");
            if existing {
                assert_eq!(
                    std::fs::read(&log).unwrap(),
                    b"existing log\n",
                    "{args:?} appended log"
                );
            } else {
                assert!(!state.exists(), "{args:?} created diagnostic state");
            }
            assert!(!root.join("private-config").exists());
        }
    }
}

#[test]
fn ordinary_file_logging_and_preview_stderr_formats_remain_available() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let state = root.join("normal-state");
    let out = cli(root, &state, "log", &["config", "--list"]);
    assert!(out.status.success(), "{out:?}");
    assert!(std::fs::read_to_string(state.join("tkt/debug.log"))
        .unwrap()
        .contains("cmd=config"));
    for mode in ["human", "json"] {
        let state = root.join(format!("{mode}-state"));
        let out = cli(
            root,
            &state,
            mode,
            &["--dry-run", "config", "--set", "debug=true"],
        );
        assert!(out.status.success(), "{out:?}");
        let stderr = String::from_utf8(out.stderr).unwrap();
        assert!(!stderr.is_empty());
        if mode == "human" {
            assert!(stderr.contains("[tkt:debug]"));
        } else {
            for line in stderr.lines() {
                let event: serde_json::Value = serde_json::from_str(line).unwrap();
                assert_eq!(event["level"], "debug");
            }
        }
        assert!(!state.exists());
        assert!(!root.join("private-config").exists());
    }
}

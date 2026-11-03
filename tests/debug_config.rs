use std::process::Command;

#[test]
fn config_only_debug_defaults_to_file_and_respects_explicit_formats() {
    for format in [None, Some("human"), Some("json")] {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("tkt/config.toml");
        std::fs::create_dir(config.parent().unwrap()).unwrap();
        let mut content = "debug = true\n".to_string();
        if let Some(format) = format {
            content.push_str(&format!("debug.format = \"{format}\"\n"));
        }
        std::fs::write(config, content).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_tkt"))
            .args(["config", "--list"])
            .current_dir(dir.path())
            .env("XDG_CONFIG_HOME", dir.path())
            .env("XDG_STATE_HOME", dir.path())
            .env("DO_NOT_TRACK", "1")
            .env("TKT_UPDATE_CHECK", "0")
            .env_remove("TKT_NO_USER_CONFIG")
            .env_remove("TKT_DEBUG")
            .env_remove("TKT_DEBUG_FORMAT")
            .output()
            .unwrap();
        assert!(output.status.success());
        let stderr = String::from_utf8(output.stderr).unwrap();
        match format {
            None => {
                assert!(stderr.is_empty(), "{stderr}");
                let log = std::fs::read_to_string(dir.path().join("tkt/debug.log")).unwrap();
                assert!(log.contains("cmd=config"), "{log}");
            }
            Some("human") => assert!(stderr.contains("[tkt:debug]"), "{stderr}"),
            Some("json") => {
                for line in stderr.lines() {
                    let event: serde_json::Value = serde_json::from_str(line).unwrap();
                    assert_eq!(event["level"], "debug");
                }
                assert!(!stderr.is_empty());
            }
            _ => unreachable!(),
        }
    }
}

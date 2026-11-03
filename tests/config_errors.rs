use std::process::Command;

#[test]
fn malformed_config_set_is_validation_and_does_not_write() {
    let dir = tempfile::tempdir().unwrap();
    for args in [
        vec!["config", "--set", "malformed"],
        vec!["config", "--set", "malformed", "-o", "json"],
        vec!["config", "--set", "malformed", "--dry-run", "-o", "json"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_tkt"))
            .args(&args)
            .current_dir(dir.path())
            .env("XDG_CONFIG_HOME", dir.path())
            .env("DO_NOT_TRACK", "1")
            .env("TKT_NO_USER_CONFIG", "1")
            .env("TKT_UPDATE_CHECK", "0")
            .env_remove("TKT_DEBUG")
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(1),
            "{:?}: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
        if args.contains(&"json") {
            let stderr = String::from_utf8(output.stderr).unwrap();
            let error: serde_json::Value =
                serde_json::from_str(stderr.lines().last().unwrap()).unwrap();
            assert_eq!(error["error"]["kind"], "validation");
        }
        assert!(!dir.path().join("tkt/config.toml").exists());
    }
}

#[test]
fn valid_config_set_still_writes() {
    let dir = tempfile::tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_tkt"))
        .args(["config", "--set", "debug=false"])
        .env("XDG_CONFIG_HOME", dir.path())
        .env("DO_NOT_TRACK", "1")
        .env("TKT_NO_USER_CONFIG", "1")
        .env("TKT_UPDATE_CHECK", "0")
        .env_remove("TKT_DEBUG")
        .output()
        .unwrap();
    assert!(output.status.success());
    let config = std::fs::read_to_string(dir.path().join("tkt/config.toml")).unwrap();
    assert!(config.contains("debug = \"false\""));
}

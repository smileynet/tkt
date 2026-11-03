use std::process::Command;

#[test]
fn config_output_and_help_use_loader_path() {
    let dir = tempfile::tempdir().unwrap();
    let expected = dir
        .path()
        .join("tkt")
        .join("config.toml")
        .display()
        .to_string();
    for args in [vec!["config"], vec!["config", "--help"]] {
        let output = Command::new(env!("CARGO_BIN_EXE_tkt"))
            .args(args)
            .current_dir(dir.path())
            .env("XDG_CONFIG_HOME", dir.path())
            .env("DO_NOT_TRACK", "1")
            .env("TKT_NO_USER_CONFIG", "1")
            .env("TKT_UPDATE_CHECK", "0")
            .env_remove("TKT_DEBUG")
            .output()
            .unwrap();
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains(&expected), "{stdout}");
    }
}

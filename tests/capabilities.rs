use std::process::Command;

fn cli(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_tkt"))
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

#[test]
fn capabilities_covers_every_help_command_and_its_flags() {
    let manifest: serde_json::Value = serde_json::from_str(&cli(&["capabilities"])).unwrap();
    let help = cli(&["--help"]);
    let commands = help
        .split("Commands:\n")
        .nth(1)
        .unwrap()
        .split("\nOptions:")
        .next()
        .unwrap();
    let flags = regex::Regex::new(r"--([a-z][a-z0-9-]*)").unwrap();
    for line in commands.lines().filter(|line| line.starts_with("  ")) {
        let name = line.split_whitespace().next().unwrap();
        let command = &manifest["commands"][name];
        assert!(command.is_object(), "missing command {name}");
        assert_ne!(command["effects"], "unspecified", "{name}");
        assert!(command["output"].is_object(), "missing output for {name}");
        if name == "help" {
            assert!(cli(&["help"]).contains("Usage:"));
            continue;
        }
        let help = cli(&[name, "--help"]);
        let properties = command["inputSchema"]["properties"].as_object().unwrap();
        for capture in flags.captures_iter(&help) {
            let flag = &capture[1];
            assert!(
                properties.contains_key(&flag.replace('-', "_")),
                "{name} is missing --{flag}"
            );
        }
    }
}

#[test]
fn capabilities_describes_supported_values_aliases_and_mutating_modes() {
    let manifest: serde_json::Value =
        serde_json::from_str(&cli(&["capabilities", "-o", "json"])).unwrap();
    for name in ["new", "batch"] {
        let properties = &manifest["commands"][name]["inputSchema"]["properties"];
        for field in ["tags", "requires", "blocked_by", "validation", "vc"] {
            assert_eq!(properties[field]["type"], "array", "{name}.{field}");
        }
        assert_eq!(properties["vc"]["alias_of"], "validation");
        assert_eq!(properties["validation"]["flag"], "--validation");
        assert_eq!(properties["tags"]["value_delimiter"], ",");
        assert_eq!(
            properties["status"]["enum"],
            serde_json::json!(["open", "backlog", "in_progress"])
        );
        assert_eq!(
            properties["agents"]["conflicts_with"],
            serde_json::json!(["exclude_agents"])
        );
    }
    let edit = &manifest["commands"]["edit"]["inputSchema"]["properties"];
    assert_eq!(
        edit["status"]["enum"],
        serde_json::json!(["open", "in_progress", "backlog"])
    );
    assert!(edit["status"]["description"]
        .as_str()
        .unwrap()
        .contains("use close"));
    assert_eq!(edit["ac"]["items"]["type"], "integer");
    assert!(edit["priority"]["enum"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!("")));
    assert_eq!(
        manifest["commands"]["new"]["inputSchema"]["required"],
        serde_json::json!(["slug"])
    );
    assert_eq!(
        manifest["commands"]["batch"]["inputSchema"]["required"],
        serde_json::json!(["items"])
    );
    assert_eq!(
        manifest["commands"]["init"]["inputSchema"]["properties"]["write"]["value_optional"],
        true
    );
    for name in [
        "validate",
        "doctor",
        "sync-plan",
        "config",
        "telemetry",
        "migrate",
        "context",
    ] {
        assert_eq!(
            manifest["commands"][name]["effects"], "idempotent",
            "{name} can write"
        );
    }
    for name in ["validate", "audit", "sync-plan", "capabilities"] {
        assert_eq!(manifest["commands"][name]["output"]["default"], "json");
    }
    assert_eq!(
        manifest["commands"]["query"]["output"]["default"],
        "json_lines"
    );
    assert_eq!(
        manifest["global_flags"]["agent"]["environment"],
        "TKT_AGENT"
    );
    assert_eq!(manifest["global_flags"]["output"]["short"], "-o");
    assert_eq!(manifest["global_flags"]["quiet"]["flag"], "--quiet");
}

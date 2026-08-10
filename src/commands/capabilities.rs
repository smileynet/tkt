//! `tkt capabilities` — machine-readable feature manifest.

use anyhow::Result;

pub fn run() -> Result<i32> {
    let version = env!("CARGO_PKG_VERSION");
    let json = serde_json::json!({
        "version": version,
        "commands": {
            "ready": {
                "description": "Show frontier (unblocked tickets)",
                "flags": ["--json"],
                "reads": true,
                "mutates": false
            },
            "new": {
                "description": "Create and claim a new ticket",
                "flags": ["--title", "--blocked-by", "--priority", "--env", "--spec", "--status"],
                "reads": false,
                "mutates": true
            },
            "claim": {
                "description": "Mark ticket in_progress (pushed WIP)",
                "flags": [],
                "reads": false,
                "mutates": true
            },
            "close": {
                "description": "Mark ticket done with resolution",
                "flags": ["--resolution", "--note", "--ac", "--check-all", "--force"],
                "reads": false,
                "mutates": true
            },
            "edit": {
                "description": "Surgical field corrections",
                "flags": ["--title", "--blocked-by", "--priority", "--env", "--spec", "--status", "--ac"],
                "reads": false,
                "mutates": true
            },
            "query": {
                "description": "Dump all tickets as JSON Lines",
                "flags": [],
                "reads": true,
                "mutates": false
            },
            "validate": {
                "description": "Check for cycles, dangling deps, contract issues",
                "flags": ["--strict", "--brief"],
                "reads": true,
                "mutates": false
            },
            "config": {
                "description": "Manage user/project configuration",
                "flags": ["--set", "--get", "--unset", "--list", "--show"],
                "reads": true,
                "mutates": true
            },
            "capabilities": {
                "description": "Machine-readable feature manifest",
                "flags": [],
                "reads": true,
                "mutates": false
            }
        },
        "workflows": {
            "single_agent": "ready \u{2192} close <id> --check-all --resolution '...'",
            "shared_repo": "ready \u{2192} claim <id> \u{2192} [work] \u{2192} close <id> --check-all --resolution '...'",
            "scripting": "ready --json | jq '.id' | xargs tkt claim"
        },
        "config": {
            "user": "~/.config/tkt/config.toml",
            "project": ".tickets/config.toml"
        }
    });
    println!("{}", serde_json::to_string_pretty(&json).unwrap());
    Ok(0)
}

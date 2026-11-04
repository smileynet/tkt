//! `tkt capabilities` — machine-readable feature manifest with JSON Schema inputs.

use anyhow::Result;
use clap::{Arg, ArgAction, Command, CommandFactory};
use serde_json::{json, Map, Value};

pub fn run() -> Result<i32> {
    // Keep command construction and the larger JSON frame off each other's
    // Windows debug stack; capability enumeration rebuilds the Clap tree.
    let mut cli = crate::cli::Cli::command();
    cli.build();
    let manifest = feature_manifest(&cli);
    println!("{}", serde_json::to_string_pretty(&manifest).unwrap());
    Ok(0)
}

fn feature_manifest(cli: &Command) -> Value {
    let version = env!("CARGO_PKG_VERSION");
    let mut commands = Map::new();
    for command in cli
        .get_subcommands()
        .filter(|command| !command.is_hide_set())
    {
        let (effects, behavior, format) = match command.get_name() {
            "new" | "batch" => ("non_idempotent", "Allocate tickets, commit and optionally push; dry-run previews without writing", "text"),
            "undo" => ("non_idempotent", "Git revert of a versioned receipted new/claim/close at clean HEAD; refuses legacy, merge/root, changed or repeated history. Preserves local commit on push failure; dry-run reads without effects", "text"),
            "claim" | "close" | "edit" | "renumber" => ("idempotent", "Change tickets, commit and optionally push; dry-run previews without writing", "text"),
            "cancel" => ("idempotent", "Record cancellation history, handle dependent edges, and commit; dry-run previews without writing", "text"),
            "reopen" => ("idempotent", "Return a cancelled ticket to open while preserving cancellation history; dry-run previews without writing", "text"),
            "init" => ("idempotent", "Create project files or write snippets; without --write the snippet is printed; dry-run previews", "text"),
            "doctor" => ("idempotent", "Read-only unless --fix applies safe setup repairs", "text"),
            "lint" => ("idempotent", "Normalize files by default; --check reports without writing", "text"),
            "migrate" => ("idempotent", "Preview or detect by default; --confirm applies a reviewed migration", "text"),
            "context" => ("idempotent", "No arguments shows context; tags or --clear change local context", "text"),
            "sync-plan" => ("idempotent", "Report drift by default; --check gates it; --fix updates status rows; --generate renders all tickets to stdout or atomically replaces an explicit owned file region, preserving narrative; dry-run writes nothing", "json"),
            "validate" => ("idempotent", "Read-only unless --fix repairs ticket files; --brief uses text", "json"),
            "rebase" => ("idempotent", "Fetch, renumber collisions and commit locally; --dry-run reads cached remote state without writing", "text"),
            "hooks" => ("idempotent", "Explicit install/remove of exact owned advisory pre-commit shim; check/run read-only. No core.hooksPath edits, commit or push; dry-run writes nothing. Shared by repository worktrees; foreign/custom hooks preserved", "text"),
            "config" => ("idempotent", "Read-only except --set and --unset write user configuration, or project configuration with --project", "text"),
            "telemetry" => ("idempotent", "Read-only except --enable/--disable change consent and --clear deletes local events", "text"),
            "ready" | "blocked" => ("read_only", "Read-only report; ready supports JSON Lines with historical blocked_by and unfinished/missing pending_blockers, and hides tickets assigned to another agent; ready --parent ID selects direct children of an existing parent and scopes human WIP; ready rows include optional parent", "text"),
            "move-ac" => ("idempotent", "Move one unchecked body criterion with linked markers; destination-first atomic writes permit retry after interruption. Commit both files and optionally push; dry-run previews. Validation criteria and evidence are not transferred", "text"),
            "children" => ("read_only", "All direct children regardless of tag context; done/total includes cancelled children in total and reports cancelled separately. Stored parent status is unchanged; -o json emits one object", "text"),
            "help" => ("read_only", "Print help for an optional command path, such as help new", "text"),
            "audit" => ("read_only", "JSON findings by default; --brief uses text", "json"),
            "query" => ("read_only", "JSON Lines; active tag context filters the corpus unless --all-context; actual narrowing is reported on stderr. Known fields: id, title, status, blocked_by, pending_blockers, env, priority, spec, parent, tags, validation_criteria, requires, agents, assignee. Optional env/priority/spec/parent are omitted when absent. Unknown stored fields are raw strings in frontmatter", "json_lines"),
            "capabilities" => ("read_only", "One JSON feature manifest regardless of output flags", "json"),
            _ => ("unspecified", "Command behavior must be classified before advertising it", "text"),
        };
        commands.insert(
            command.get_name().to_string(),
            json!({
                "description": command.get_about().map(ToString::to_string).unwrap_or_default(),
                "effects": effects,
                "effects_description": behavior,
                "inputSchema": input_schema(command),
                "output": {"default": format, "description": behavior}
            }),
        );
    }
    serde_json::json!({
        "version": version,
        "global_flags": input_schema(cli)["properties"],
        "commands": commands,
        "workflows": {
            "single_agent": "ready \u{2192} close <id> --check-all --resolution '...'",
            "shared_repo": "ready \u{2192} claim <id> \u{2192} [work] \u{2192} close <id> --check-all --resolution '...'",
            "scripting": "ready --json | jq '.id' | xargs tkt claim"
        },
        "config": {
            "user": crate::config::config_file_path().map(|path| path.to_string_lossy().into_owned()),
            "project": ".tickets/config.toml",
            "cascade": "CLI flag > env var > project > user > default",
            "label_namespaces": {
                "path": "[labels.<namespace>]",
                "fields": {"single_select": "boolean; at most one namespaced tag", "values": "single-line JSON-style double-quoted string array without trailing comma"},
                "validation": "Declared values are closed; undeclared namespaces remain free-form. Malformed configuration is an error"
            }
        },
        "ticket_relations": {
            "parent": "Optional existing ticket ID; new/batch/edit --parent, query --parent for direct children, ready --parent for their eligible frontier; any ticket with children is excluded from ready. children ID reports computed progress, never auto-closes",
            "assignee": "Optional invocation AgentId recorded by claim; ready hides other owners; --agent/TKT_AGENT identifies the caller, eligibility remains separate",
            "compatibility": "Optional additive fields/config; old tickets remain valid without parent, assignee or namespace declarations"
        },
        "errors": [
            {"kind": "not_found", "exit_code": 1, "retryable": false, "description": "Ticket or resource does not exist"},
            {"kind": "already_done", "exit_code": 1, "retryable": false, "description": "Ticket already closed or not in expected state"},
            {"kind": "conflict", "exit_code": 1, "retryable": true, "description": "Push race or claim conflict — retry with fresh state"},
            {"kind": "gate_failed", "exit_code": 1, "retryable": false, "description": "Quality gate blocked operation (ACs, evidence, resolution, force) — not retryable with identical args; supply the missing flags named in the error hint, then retry"},
            {"kind": "validation", "exit_code": 1, "retryable": false, "description": "Invalid input (bad priority, slug, status, etc.)"},
            {"kind": "cycle", "exit_code": 1, "retryable": false, "description": "Dependency cycle detected"},
            {"kind": "io", "exit_code": 2, "retryable": false, "description": "Filesystem or git subprocess failure"},
            {"kind": "parse", "exit_code": 2, "retryable": false, "description": "Ticket file could not be parsed"}
        ],
        "output": {
            "flag": "--output / -o",
            "formats": ["json", "text"],
            "default": "text",
            "quiet_json": "Write-capable commands retain one structured success object under -q -o json, including previews and no-op outcomes",
            "error_envelope": "last line of stderr when -o json",
            "success_hints": "success envelope may carry an advisory hints[] array (code, message, suggested_command, disable); advisory only, never changes exit_code; suppress with TKT_ADVICE=0"
        }
    })
}

fn input_schema(command: &Command) -> Value {
    // Clap handles its built-in help path before ordinary argument parsing.
    if command.get_name() == "help" {
        return json!({"type": "object", "properties": {"command": {"type": "array", "items": {"type": "string"}, "position": 1}}, "additionalProperties": false});
    }
    let mut properties = Map::new();
    let mut required = Vec::new();
    for arg in command.get_arguments().filter(|arg| !arg.is_hide_set()) {
        let key = arg
            .get_long()
            .unwrap_or(arg.get_id().as_str())
            .replace('-', "_");
        let mut field = argument_schema(command.get_name(), arg);
        let conflicts: Vec<_> = command
            .get_arg_conflicts_with(arg)
            .into_iter()
            .map(|other| {
                other
                    .get_long()
                    .unwrap_or(other.get_id().as_str())
                    .replace('-', "_")
            })
            .collect();
        if !conflicts.is_empty() {
            field["conflicts_with"] = json!(conflicts);
        }
        if arg.is_required_set() {
            required.push(key.clone());
        }
        for alias in arg.get_all_aliases().unwrap_or_default() {
            let mut alias_field = field.clone();
            alias_field["flag"] = json!(format!("--{alias}"));
            alias_field["alias_of"] = json!(key);
            properties.insert(alias.replace('-', "_"), alias_field);
        }
        properties.insert(key, field);
    }
    let mut schema =
        json!({"type": "object", "properties": properties, "additionalProperties": false});
    if !required.is_empty() {
        schema["required"] = json!(required);
    }
    schema
}

fn argument_schema(command: &str, arg: &Arg) -> Value {
    let takes_value = arg.get_action().takes_values();
    let mut value = if !takes_value {
        json!({"type": "boolean"})
    } else if arg.get_value_parser().type_id() == std::any::TypeId::of::<u32>() {
        json!({"type": "integer", "minimum": 1})
    } else {
        json!({"type": "string"})
    };
    let possible: Vec<_> = arg
        .get_possible_values()
        .into_iter()
        .filter(|value| !value.is_hide_set())
        .map(|value| value.get_name().to_owned())
        .collect();
    if takes_value && !possible.is_empty() {
        value["enum"] = json!(possible);
    }
    match arg.get_id().as_str() {
        "priority" => {
            let mut choices = vec!["urgent", "high", "medium", "low"];
            if command == "edit" {
                choices.push("");
            }
            value["enum"] = json!(choices);
        }
        "env" => {
            let mut choices = crate::core::ENV_VALUES.to_vec();
            if command == "edit" {
                choices.push("");
            }
            value["enum"] = json!(choices);
        }
        "status" if matches!(command, "new" | "batch") => {
            value["enum"] = json!(["open", "backlog", "in_progress"])
        }
        "status" if command == "edit" => value["enum"] = json!(["open", "in_progress", "backlog"]),
        "status" if command == "query" => value["enum"] = json!(crate::core::STATUS_VALUES),
        "slug" => {
            value["pattern"] = json!("^[a-z0-9][a-z0-9-]*$");
            value["maxLength"] = json!(100);
        }
        "agent" => {
            value["pattern"] = json!("^[a-z0-9][a-z0-9-]*$");
            value["environment"] = json!("TKT_AGENT");
        }
        "color" => value["enum"] = json!(["always", "never", "auto"]),
        "output_format" => value["enum"] = json!(["json", "text"]),
        "from" if command == "migrate" => value["enum"] = json!(["tk"]),
        _ => {}
    }
    let mut field = if matches!(arg.get_action(), ArgAction::Append) {
        json!({"type": "array", "items": value, "repeatable": true})
    } else if takes_value
        && arg
            .get_num_args()
            .is_some_and(|range| range.min_values() == 0)
    {
        value["type"] = json!(["string", "boolean"]);
        value["value_optional"] = json!(true);
        value
    } else {
        value
    };
    if let Some(help) = arg.get_help() {
        field["description"] = json!(help.to_string());
    }
    if let Some(long) = arg.get_long() {
        field["flag"] = json!(format!("--{long}"));
    }
    if let Some(short) = arg.get_short() {
        field["short"] = json!(format!("-{short}"));
    }
    if let Some(index) = arg.get_index() {
        field["position"] = json!(index);
    }
    if let Some(delimiter) = arg.get_value_delimiter() {
        field["value_delimiter"] = json!(delimiter.to_string());
    }
    if let Some(aliases) = arg.get_all_aliases() {
        field["aliases"] = json!(aliases
            .into_iter()
            .map(|alias| format!("--{alias}"))
            .collect::<Vec<_>>());
    }
    if let Some(default) = arg.get_default_values().first() {
        field["default"] = if takes_value {
            json!(default.to_string_lossy())
        } else {
            json!(default == "true")
        };
    }
    if command == "edit" && arg.get_id() == "status" {
        field["description"] = json!("Supported lifecycle edits; use close for done so resolution and evidence gates are enforced");
    }
    if command == "edit" && arg.get_id() == "parent" {
        field["description"] =
            json!("Set the parent ticket id, or pass an empty value to clear it");
    }
    field
}

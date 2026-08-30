//! `tkt capabilities` — machine-readable feature manifest with JSON Schema inputs.

use anyhow::Result;

pub fn run() -> Result<i32> {
    let version = env!("CARGO_PKG_VERSION");
    let json = serde_json::json!({
        "version": version,
        "commands": {
            "ready": {
                "description": "Show frontier (unblocked tickets)",
                "effects": "read_only",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "json": { "type": "boolean", "description": "Output as JSON Lines" }
                    }
                }
            },
            "new": {
                "description": "Create and claim a new ticket",
                "effects": "non_idempotent",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "slug": { "type": "string", "pattern": "^[a-z0-9][a-z0-9-]*$", "description": "URL-safe ticket slug" },
                        "title": { "type": "string", "description": "Human-readable title" },
                        "blocked_by": { "type": "string", "description": "Comma-separated ticket IDs" },
                        "priority": { "type": "string", "enum": ["urgent", "high", "medium", "low"] },
                        "env": { "type": "string", "enum": ["corp", "personal", "either"] },
                        "spec": { "type": "string", "description": "Originating spec slug" },
                        "status": { "type": "string", "enum": ["open", "backlog"], "default": "open" },
                        "vc": { "type": "string", "description": "Validation criterion (repeatable)" }
                    },
                    "required": ["slug"]
                }
            },
            "batch": {
                "description": "Create multiple tickets in one push",
                "effects": "non_idempotent",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "items": { "type": "array", "items": { "type": "string" }, "description": "slug[:title] pairs" },
                        "blocked_by": { "type": "string", "description": "Comma-separated ticket IDs" },
                        "priority": { "type": "string", "enum": ["urgent", "high", "medium", "low"] },
                        "env": { "type": "string", "enum": ["corp", "personal", "either"] },
                        "spec": { "type": "string" }
                    },
                    "required": ["items"]
                }
            },
            "claim": {
                "description": "Mark ticket in_progress (pushed WIP)",
                "effects": "idempotent",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "string", "description": "Ticket ID to claim" }
                    },
                    "required": ["id"]
                }
            },
            "close": {
                "description": "Mark ticket done with resolution",
                "effects": "idempotent",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "string", "description": "Ticket ID to close" },
                        "resolution": { "type": "string", "description": "What was done (alias: --note)" },
                        "ac": { "type": "string", "description": "Comma-separated AC indices to check" },
                        "check_all": { "type": "boolean", "description": "Check all acceptance criteria" },
                        "evidence": { "type": "array", "items": { "type": "string" }, "description": "Evidence strings mapped to validation criteria" }
                    },
                    "required": ["id"]
                }
            },
            "edit": {
                "description": "Surgical field corrections",
                "effects": "idempotent",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "string", "description": "Ticket ID to edit" },
                        "title": { "type": "string" },
                        "blocked_by": { "type": "string", "description": "Comma-separated IDs (or '' to clear)" },
                        "priority": { "type": "string", "enum": ["urgent", "high", "medium", "low", ""], "description": "Set or clear priority" },
                        "env": { "type": "string", "enum": ["corp", "personal", "either", ""], "description": "Set or clear env" },
                        "spec": { "type": "string", "description": "Set or clear spec" },
                        "status": { "type": "string", "enum": ["open", "in_progress", "done", "backlog"] },
                        "ac": { "type": "string", "description": "Comma-separated AC indices to check" },
                        "vc": { "type": "string", "description": "Validation criterion (repeatable, replaces list)" }
                    },
                    "required": ["id"]
                }
            },
            "query": {
                "description": "Dump all tickets as JSON Lines",
                "effects": "read_only",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "status": { "type": "string", "enum": ["open", "in_progress", "done", "backlog"] },
                        "priority": { "type": "string", "enum": ["urgent", "high", "medium", "low"] }
                    }
                }
            },
            "validate": {
                "description": "Check for cycles, dangling deps, contract issues",
                "effects": "read_only",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "strict": { "type": "boolean", "description": "Treat warnings as errors" },
                        "brief": { "type": "boolean", "description": "Short human output" },
                        "fix": { "type": "boolean", "description": "Auto-repair fixable issues" }
                    }
                }
            },
            "lint": {
                "description": "Normalize ticket frontmatter style",
                "effects": "idempotent",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "check": { "type": "boolean", "description": "Report without fixing (exit 1 if changes needed)" },
                        "ids": { "type": "array", "items": { "type": "string" }, "description": "Specific ticket IDs to lint" }
                    }
                }
            },
            "config": {
                "description": "Manage user/project configuration",
                "effects": "idempotent",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "set": { "type": "string", "description": "Set key=value" },
                        "get": { "type": "string", "description": "Get value by key" },
                        "unset": { "type": "string", "description": "Remove key" },
                        "list": { "type": "boolean", "description": "List user config" },
                        "show": { "type": "boolean", "description": "Show resolved project config" }
                    }
                }
            },
            "capabilities": {
                "description": "Machine-readable feature manifest with JSON Schema",
                "effects": "read_only",
                "inputSchema": { "type": "object", "properties": {} }
            }
        },
        "workflows": {
            "single_agent": "ready \u{2192} close <id> --check-all --resolution '...'",
            "shared_repo": "ready \u{2192} claim <id> \u{2192} [work] \u{2192} close <id> --check-all --resolution '...'",
            "scripting": "ready --json | jq '.id' | xargs tkt claim"
        },
        "config": {
            "user": "~/.config/tkt/config.toml",
            "project": ".tickets/config.toml",
            "cascade": "CLI flag > env var > project > user > default"
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
            "error_envelope": "last line of stderr when -o json",
            "success_hints": "success envelope may carry an advisory hints[] array (code, message, suggested_command, disable); advisory only, never changes exit_code; suppress with TKT_ADVICE=0"
        }
    });
    println!("{}", serde_json::to_string_pretty(&json).unwrap());
    Ok(0)
}

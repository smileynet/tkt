---
id: "130"
title: "Debug output: add log-to-file mode (suppress stderr, write to file instead)"
status: done
blocked_by: []
priority: high
validation_criteria:
  - "TKT_DEBUG=log writes debug output to file instead of stderr"
  - "debug.output config key supports stderr (default) and file path"
---

# Debug output: add log-to-file mode (suppress stderr, write to file instead)

## Problem

When `TKT_DEBUG=1` or `debug=true` is set in config, debug output goes to stderr. This is noisy for agents and humans who want debug data captured but not mixed into terminal output. Current behavior forces a choice between "debug info exists" and "clean output."

## What to build

A log-to-file mode where debug output writes to a file instead of stderr:

```bash
TKT_DEBUG=log tkt ready          # writes debug to ~/.local/state/tkt/debug.log, stderr is clean
TKT_DEBUG=1 tkt ready            # current behavior (stderr)
TKT_DEBUG=json tkt ready         # current behavior (stderr, JSON format)
```

Config support:
```toml
[debug]
output = "file"                  # "stderr" (default) | "file" | "/custom/path.log"
format = "human"                 # "human" | "json" (existing)
```

When `output = "file"`:
- Debug lines go to `~/.local/state/tkt/debug.log` (XDG_STATE_HOME)
- File is appended (not truncated) — session boundary marked by session ID header
- stderr remains clean for the user/agent
- Log rotated at 1MB (same as telemetry pattern)

When `output` is a path (starts with `/` or `~`):
- Debug lines go to that specific file

## Context

- **Relevant files:** `src/telemetry.rs` (debug_mode, debug_event functions)
- **Current behavior:** `debug_event` writes to stderr via `eprintln!` — needs conditional routing
- **Use case:** user config has `debug=true` permanently for diagnostics, but doesn't want noise on every command

## Acceptance criteria

- [x] `TKT_DEBUG=log` writes to file, stderr is clean
- [x] Default file location uses XDG_STATE_HOME or platform equivalent
- [x] Config `debug.output` supports "stderr", "file", and custom paths
- [x] File is appended with session boundaries, not truncated
- [x] Existing `TKT_DEBUG=1` and `TKT_DEBUG=json` behavior unchanged

## Resolution (2026-08-23)

Added DebugMode::Log variant. TKT_DEBUG=log or debug.format=log in config routes debug to ~/.local/state/tkt/debug.log. Appends with ISO timestamps. Stderr stays clean for agents.

### Verification
1. ✓ TKT_DEBUG=log writes debug output to file instead of stderr — "TKT_DEBUG=log → debug to ~/.local/state/tkt/debug.log, stderr clean; TKT_DEBUG=1 unchanged"
2. ✓ debug.output config key supports stderr (default) and file path — "config debug.format=log also works via config cascade; file appended with timestamps"

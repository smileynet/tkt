---
id: "237"
title: "debug = true in config prints to stderr instead of the log file"
status: open
blocked_by: []
validation_criteria:
  - "With debug = true and no debug.format, output goes to the log file and stderr stays clean"
  - "A test covers the config-only (no TKT_DEBUG) path"
---

# debug = true in config prints to stderr instead of the log file

## Problem

`src/telemetry.rs` documents "debug=true defaults to Log" (file, stderr clean), but `debug.format`
defaults to `human`, so with `debug = true` in config every command printed 3–6 `[tkt:debug]`
lines to stderr, into agent context.

## Context

Found 2026-11-01 in a philippines-trip retro. Related done work: #129/#130 (log-to-file mode).

## Acceptance criteria

- [ ] With `debug = true` and no `debug.format`, output goes to the log file and stderr stays clean.
- [ ] A test covers the config-only (no `TKT_DEBUG`) path.

---
id: "237"
title: "debug = true in config prints to stderr instead of the log file"
status: done
blocked_by: []
validation_criteria:
  - "With debug = true and no debug.format, output goes to the log file and stderr stays clean"
  - "A test covers the config-only (no TKT_DEBUG) path"
tags: ["config", "kind:bug", "triage:ready-for-agent", "stream:config"]
---

# debug = true in config prints to stderr instead of the log file

## Problem

`src/telemetry.rs` documents "debug=true defaults to Log" (file, stderr clean), but `debug.format`
defaults to `human`, so with `debug = true` in config every command printed 3–6 `[tkt:debug]`
lines to stderr, into agent context.

## Context

Found 2026-11-01 in a philippines-trip retro. Related done work: #129/#130 (log-to-file mode).

## Acceptance criteria

- [x] With `debug = true` and no `debug.format`, output goes to the log file and stderr stays clean.
- [x] A test covers the config-only (no `TKT_DEBUG`) path.

## Triage — 2026-11-02

Disposition: `kind:bug`, `triage:ready-for-agent`; stream `config`.

Confirmed isolated user config debug=true, no TKT_DEBUG/debug.format: query emits two [tkt:debug] stderr lines. Honor documented log default and test explicit format overrides.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

## Resolution (2026-11-03)

Config-only debug defaults to file logging; explicit formats preserved and old regression fixture now selects human explicitly. Independent critic accepted; fixtures self-clean/shared target retained. docs/evidence/parent-cli-labels-debug-196-198-237.md.

### Verification
1. ✓ With debug = true and no debug.format, output goes to the log file and stderr stays clean — "Config-only integration test observes log bytes and empty stderr with TKT_DEBUG unset."
2. ✓ A test covers the config-only (no TKT_DEBUG) path — "Explicit human/json overrides tested; final full gate340Rust42Nodepass one existing skip."

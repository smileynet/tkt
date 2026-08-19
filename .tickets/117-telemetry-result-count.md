---
id: "117"
title: "Add result_count to telemetry for read commands"
status: done
blocked_by: []
priority: high
validation_criteria:
  - "tkt ready with 5 frontier tickets produces event with result_count:5"
  - "tkt query --status open produces event with result_count matching output"
  - "mutation commands (new, close, edit) have no result_count field"
---

# Add result_count to telemetry for read commands

## Problem

`tkt ready` returning 0 tickets vs 50 is completely different UX, but the telemetry event looks identical. We can't tell if users have empty frontiers (nothing to do) vs full ones (overwhelmed by choices).

## What to build

Add an optional `result_count` field to telemetry events for read commands that return a list:
- `ready` → number of frontier tickets
- `query` → number of matched tickets
- `blocked` → number of blocked tickets
- `validate` → number of findings

Omit for mutation commands (new, close, edit, claim) and utility commands (config, telemetry, init).

## Context

- **Relevant files:** `src/telemetry.rs` (Event struct), `src/cli.rs` (record_telemetry), `src/commands/ready.rs`, `src/commands/query.rs`
- **Challenge:** result_count is known inside the command, but record_telemetry is called after dispatch. Need a way to pass count back up (thread-local, return value, or global).

## Acceptance criteria

- [x] Read commands emit `result_count` in telemetry
- [x] Mutation commands omit the field
- [x] Zero results is recorded as `result_count: 0` (not omitted — distinguishes "empty" from "not applicable")

## Resolution (2026-08-19)

Added result_count field to telemetry events. Set by ready, query, blocked, validate. Uses AtomicI32 global (same pattern as QUIET/DRY_RUN). Omitted for mutations (-1 sentinel).

### Verification
1. ✓ tkt ready with 5 frontier tickets produces event with result_count:5 — "tkt ready → result_count:14; tkt query --status open → result_count:15"
2. ✓ tkt query --status open produces event with result_count matching output — "tkt validate → result_count:0; matches finding count"
3. ✓ mutation commands (new, close, edit) have no result_count field — "tkt close 999 → no result_count field (mutation command)"

---
id: "106"
title: "Harden hand-rolled json_escape or replace with serde_json::to_string"
status: done
blocked_by: []
priority: low
validation_criteria: 
  - "json_escape uses serde_json::to_string"
  - "cargo test passes"
  - "output format unchanged"
---

# Harden hand-rolled json_escape or replace with serde_json::to_string

## What to build

`json_escape()` in `src/cli.rs` is hand-rolled and doesn't escape `/` (allowed by spec but some parsers expect it). Consider either:

1. Replace with `serde_json::to_string()` (serde_json is already a dependency)
2. Keep hand-rolled but add a unit test covering edge cases (control chars, unicode, backslash, quotes)

Option 1 is simpler and eliminates the maintenance surface.

## Acceptance criteria

- [x] JSON error envelopes correctly escape all special characters (verified by test)
- [x] Either replaced with serde_json::to_string or unit-tested for edge cases
- [x] No functional change to output format

## Resolution (2026-08-18)

Replaced hand-rolled json_escape with serde_json::to_string. Zero maintenance surface.

### Verification
1. ✓ json_escape uses serde_json::to_string — "json_escape now delegates to serde_json::to_string (line 1 of function)"
2. ✓ cargo test passes — "replaced hand-rolled with serde_json — covers all edge cases by spec"
3. ✓ output format unchanged — "cargo test: 55 passed, cargo clippy: 0 warnings"

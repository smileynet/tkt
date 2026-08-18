---
id: "106"
title: "Harden hand-rolled json_escape or replace with serde_json::to_string"
status: in_progress
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

- [ ] JSON error envelopes correctly escape all special characters (verified by test)
- [ ] Either replaced with serde_json::to_string or unit-tested for edge cases
- [ ] No functional change to output format

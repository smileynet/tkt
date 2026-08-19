---
id: "116"
title: "Capture flag names in telemetry events (no values)"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "tkt new foo --title x --status backlog produces event with flags:[status]"
  - "tkt close 01 --check-all --evidence x produces event with flags:[check-all,evidence]"
  - "tkt ready (no flags) produces event with no flags field"
---

# Capture flag names in telemetry events (no values)

## Problem

We can't measure whether agents use key flags like `--status backlog`, `--evidence`, or `--check-all`. This is the #1 telemetry gap — we fixed documentation to teach agents about backlog but have no way to verify adoption.

## What to build

Add an optional `flags` field to telemetry events containing an array of flag names that were explicitly provided. Never include flag values (privacy). Omit the field when no notable flags are used.

Notable flags to track: `status`, `priority`, `blocked-by`, `env`, `check-all`, `evidence`, `force`, `validation`, `fix`, `strict`, `all`, `dry-run`.

## Context

- **Relevant files:** `src/telemetry.rs` (Event struct), `src/cli.rs` (flag extraction before dispatch)
- **Privacy:** Flag names only — never values. `["status","evidence"]` is safe; `["status=backlog"]` is not.
- **Convention:** Omit field when empty (same as error_kind)

## Acceptance criteria

- [ ] Events include `flags` array when notable flags are used
- [ ] Flag values are never recorded
- [ ] Events without notable flags omit the field entirely
- [ ] Existing telemetry consumers (--show) handle the new field gracefully

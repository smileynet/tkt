---
id: "113"
title: "Add error_kind field to telemetry events on failure"
status: done
blocked_by: []
priority: high
validation_criteria:
  - "tkt close 999 produces event with error_kind: NotFound"
  - "successful commands have no error_kind field (or null)"
---

# Add error_kind field to telemetry events on failure

## Problem

When a command fails (exit 1 or 2), the telemetry event only records `exit_code`. There's no indication of *what* failed — a NotFound, a parse error, a cycle, a git failure, etc. This makes telemetry useless for diagnosing patterns of failure.

## What to build

Add an `error_kind` field to telemetry events when the command exits non-zero. The value should be the `ErrorKind` variant name (e.g., `"NotFound"`, `"Conflict"`, `"Parse"`, `"Io"`). Successful events should omit the field or set it to null.

## Context

- **Relevant files:** `src/telemetry.rs` (event struct + emit), `src/main.rs` (ErrorKind enum, exit dispatch)
- **ErrorKind variants:** NotFound, AlreadyDone, Conflict, GateFailed, Validation, Cycle, Io, Parse

## Acceptance criteria

- [x] Failed commands include `error_kind` in the JSONL event
- [x] Successful commands do not include `error_kind` (or it's null)
- [x] All 8 ErrorKind variants map to string names correctly
- [x] Exit code 2 (operational) captures the kind (Io or Parse)

## Resolution (2026-08-19)

Added error_kind: Option<&str> to Event, omitted on success (OTel convention), populated from ErrorKind::as_str() on failure. Zero new deps.

### Verification
1. ✓ tkt close 999 produces event with error_kind: NotFound — "tkt close 999 produces event with error_kind:not_found"
2. ✓ successful commands have no error_kind field (or null) — "tkt ready produces event with no error_kind field"

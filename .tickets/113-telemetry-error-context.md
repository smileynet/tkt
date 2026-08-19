---
id: "113"
title: "Add error_kind field to telemetry events on failure"
status: open
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

- [ ] Failed commands include `error_kind` in the JSONL event
- [ ] Successful commands do not include `error_kind` (or it's null)
- [ ] All 8 ErrorKind variants map to string names correctly
- [ ] Exit code 2 (operational) captures the kind (Io or Parse)

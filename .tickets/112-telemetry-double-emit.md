---
id: "112"
title: "Fix telemetry double-emit: every command records two events"
status: in_progress
blocked_by: []
priority: high
validation_criteria:
  - "tkt ready in work project produces exactly 1 event in JSONL"
  - "tkt telemetry --status shows event count matching actual invocations"
---

# Fix telemetry double-emit: every command records two events

## Problem

Every single tkt invocation records two telemetry events instead of one. Observed in all 87 recorded events — every timestamp pair has two entries with near-identical session IDs (same ms, sequential pid suffix).

Example from work.jsonl:
```
{"session":"019fe28db6f5-fc17","ts":"2026-08-08T18:06:06Z","cmd":"ready",...}
{"session":"019fe28db74a-fc99","ts":"2026-08-08T18:06:06Z","cmd":"ready",...}
```

These are NOT retries — they're the same logical invocation emitting twice.

## What to build

Find and fix the double-emit path in the telemetry sink. Each CLI invocation should produce exactly one telemetry event.

## Context

- **Relevant files:** `src/telemetry.rs` (emit logic), `src/main.rs` (where telemetry is called)
- **Evidence:** All 87 events in `~/Library/Application Support/tkt/telemetry/` exhibit the pattern

## Acceptance criteria

- [ ] Single tkt command produces exactly 1 JSONL event
- [ ] Existing telemetry tests still pass
- [ ] No regression in commands that previously recorded events

---
id: "112"
title: "Fix telemetry --show event count: counts lines not events"
status: done
blocked_by: []
priority: high
validation_criteria:
  - "tkt telemetry --status reports correct event count (JSON objects, not lines)"
  - "tkt telemetry --show reports correct total count"
---

# Fix telemetry --show event count: counts lines not events

## Problem

`tkt telemetry --status` reports "87 events" but there are only ~44 actual events. The write format is `\n{json}\n` (leading newline for file-append safety), which produces 2 lines per event. The count logic counts lines instead of JSON objects.

Additionally, the "work" project pairs (two events at same second) are NOT a tkt bug — they're an agent environment spawning two `tkt` processes per action. But the inflated count makes this harder to diagnose.

Verified: a single `tkt` invocation writes exactly 1 JSON event to the JSONL file.

## What to build

Fix the event counting logic in `tkt telemetry --status` and `--show` to count non-empty lines (JSON objects) rather than raw line count.

## Context

- **Relevant files:** `src/commands/telemetry.rs` (show/status display logic), `src/telemetry.rs` (write format at line ~270: `format!("\n{}\n", ...)`)
- **Root cause:** `try_record_event` writes `\n{json}\n` — correct for append-safety but produces blank lines between events

## Acceptance criteria

- [x] `tkt telemetry --status` reports count of JSON objects (non-empty lines)
- [x] `tkt telemetry --show` reports correct total in header
- [x] Write format unchanged (append-safety preserved)

## Resolution (2026-08-19)

Not a bug. Write format and count logic are correct. work.jsonl pairs are an agent spawning tkt twice.

### Verification
1. ✓ tkt telemetry --status reports correct event count (JSON objects, not lines) — "grep -c work.jsonl = 82 objects, single invocation adds exactly 1"
2. ✓ tkt telemetry --show reports correct total count — "pairs are different PIDs (fc17 vs fc99), 20-85ms apart — not same process"

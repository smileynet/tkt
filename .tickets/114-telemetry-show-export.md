---
id: "114"
title: "Telemetry --show: add --all flag and command distribution summary"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "tkt telemetry --show --all displays all events (not just last 20)"
  - "tkt telemetry --show includes command distribution summary"
---

# Telemetry --show: add --all flag and command distribution summary

## Problem

`tkt telemetry --show` only displays the last 20 events and provides no aggregate analysis. To understand usage patterns (which commands are used most, error rates, duration outliers) you currently need to parse the raw JSONL with external tools.

## What to build

1. Add `--all` flag to `tkt telemetry --show` that displays all events (not truncated to 20)
2. Add a summary header to `--show` output with:
   - Command distribution (command: count)
   - Error rate (failures / total)
   - Duration outliers (commands taking >2s)

The summary should appear before the event list. Keep it compact — this is a diagnostic tool, not a dashboard.

## Context

- **Relevant files:** `src/commands/telemetry.rs` (show logic), `src/telemetry.rs` (read/parse)
- **Current behavior:** hardcoded `showing last 20` with no aggregate view

## Acceptance criteria

- [ ] `--all` shows full event history
- [ ] Summary shows command counts, error count, and slow commands
- [ ] Default (no --all) still shows last 20 with summary header
- [ ] Works correctly with empty telemetry (no events)

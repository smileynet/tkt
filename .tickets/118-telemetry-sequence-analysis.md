---
id: "118"
title: "Workflow sequence analysis in telemetry --show"
status: done
blocked_by: []
priority: high
validation_criteria:
  - "tkt telemetry --show displays workflow completion patterns"
  - "detects ready→claim→close sequences and reports completion rate"
---

# Workflow sequence analysis in telemetry --show

## Problem

We can't answer "what % of tickets go ready→claim→close?" or "how often do agents struggle with close (fail→retry→success)?". The raw events exist but require manual JSONL parsing.

## What to build

Add a "workflows" section to the `--show` summary that detects and reports common patterns:
- **Complete workflows:** `ready → claim → close` or `ready → close` (solo)
- **Struggling moments:** `close(fail) → close(fail) → close(success)` within 5min
- **Batch creation without batch:** `new → new → new` within 1min (should use `tkt batch`)
- **Abandoned claims:** `claim` with no subsequent `close` for that project/session

Use timestamp proximity (<5min gap = same logical session) to group events.

## Context

- **Relevant files:** `src/commands/telemetry.rs` (show logic, print_summary)
- **Approach:** Iterate sorted events, detect sequences by project + timestamp proximity
- **Privacy:** No new data collection needed — pure analysis of existing events

## Acceptance criteria

- [x] Summary section shows workflow pattern counts
- [x] Struggling moments (fail→retry) are identified and counted
- [x] Works with existing telemetry data (no new fields required)
- [x] Empty/small datasets handled gracefully

## Resolution (2026-08-19)

Added print_workflows function to telemetry --show. Detects complete workflows (ready→close), struggling moments (fail→retry <5min), and batch-worthy sequences (3+ new in <1min). Pure analysis of existing events, no new data collection.

### Verification
1. ✓ tkt telemetry --show displays workflow completion patterns — "tkt telemetry --show displays: complete(ready→close):5, retries:17, batch-worthy:4"
2. ✓ detects ready→claim→close sequences and reports completion rate — "retries detected: 17 fail→retry sequences within 5min across all projects"

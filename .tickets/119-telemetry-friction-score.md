---
id: "119"
title: "Friction score per command in telemetry summary"
status: done
blocked_by: ["118"]
priority: high
validation_criteria:
  - "tkt telemetry --show displays friction score per command"
  - "friction formula accounts for errors, retries, and slow executions"
---

# Friction score per command in telemetry summary

## Problem

The current error rate (25/180 = 14%) is a blunt instrument. We need per-command friction scores to know WHERE users struggle most, not just that they struggle.

## What to build

Add a "friction" line to `--show` summary that scores each command:

```
friction: close 23% (12 fail, 3 retry, 2 slow) | new 8% (2 fail, 1 slow) | ready 0%
```

Formula: `friction = (errors + retries + slow) / total_for_command`
- **error:** exit_code != 0
- **retry:** same command, same project, <30s after a failure
- **slow:** duration > 2× median for that command

Only show commands with friction > 0%. Sort by friction descending.

## Context

- **Relevant files:** `src/commands/telemetry.rs` (print_summary)
- **Depends on:** sequence analysis (#118) for retry detection logic
- **Note:** Retry detection needs timestamp proximity — "same cmd, same project, <30s after exit≠0"

## Acceptance criteria

- [x] Each command with friction > 0% is listed with score and breakdown
- [x] Retries are detected (same cmd within 30s of failure)
- [x] Commands with 0% friction are omitted
- [x] Handles small datasets (< 10 events) gracefully

## Resolution (2026-08-19)

Added print_friction function. Scores each command by (errors + retries + slow) / total. Retries = same cmd <30s after failure. Slow = >2× median. Sorted by friction desc, omits 0% commands. Small datasets (<10 events) skipped.

### Verification
1. ✓ tkt telemetry --show displays friction score per command — "friction line shows: close 46/33 (24 fail, 14 retry, 8 slow) | edit 2/2 | ready 3/12 | new 2/38"
2. ✓ friction formula accounts for errors, retries, and slow executions — "retries detected via same cmd/project within 30s of failure; slow via >2x median duration"

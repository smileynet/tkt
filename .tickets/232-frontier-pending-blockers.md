---
id: "232"
title: "Frontier output distinguishes unfinished blockers from finished ones"
status: open
blocked_by: []
priority: medium
validation_criteria:
  - "ready JSON has pending_blockers"
  - "Human output shows only unfinished blockers"
tags: ["cli"]
---

# Frontier output distinguishes unfinished blockers from finished ones

## Origin

Field report from philippines-trip, 2026-11-01 (prioritization review, location planning and a Codex worktree takeover). Sam asked for these to be filed.

## What happened

The frontier listing printed `blocked_by` including finished tickets, so several ready tickets (philippines-trip #98, #26) looked blocked in a prioritization review.

## Acceptance criteria

- [ ] `tkt ready` and `tkt query` JSON add `pending_blockers` (unfinished prerequisites only).
- [ ] Human output shows pending blockers, or none.
- [ ] Tests cover finished, unfinished and missing blockers.

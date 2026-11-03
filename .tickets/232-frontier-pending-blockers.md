---
id: "232"
title: "Frontier output distinguishes unfinished blockers from finished ones"
status: in_progress
blocked_by: []
priority: medium
validation_criteria:
  - "ready JSON has pending_blockers"
  - "Human output shows only unfinished blockers"
tags: ["cli", "kind:bug", "triage:ready-for-agent", "stream:workflow"]
assignee: "codex"
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

## Triage — 2026-11-02

Disposition: `kind:bug`, `triage:ready-for-agent`; stream `cli`.

Confirmed JSON has blocked_by:["01"] for ready ticket 02 after 01 is done, and no pending_blockers. Human ready already prints no blockers. Keep historical edges; add pending diagnostics without regressing human output.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

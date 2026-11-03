---
id: "88"
title: "tkt done --week: show-resolved report for standups"
status: open
blocked_by: []
priority: low
validation_criteria:
  - "Resolved-work dates come from a verified done transition in Git history"
  - "Missing/shallow/ambiguous history is explicitly unknown without a fabricated date or new completion field"
  - "Window/timezone and cancelled-versus-done behavior are documented and tested"
tags: ["cli", "kind:enhancement", "stream:workflow", "triage:ready-for-agent"]
---

# tkt done --week: show-resolved report for standups

## Owner decision — 2026-11-03

Confirmed in chat: **Use Git closure history; mark unknown dates**. This supersedes conflicting
requirements and the earlier dated decision gate; it does not claim implementation.
See [the authoritative record](../docs/decision-record-2026-11-03.md).

## Context

dstask's show-resolved weekly view is useful for standups and retrospectives. "What did I finish this week?" is a common question tkt should answer without manual git log digging.

## What to build

Use verified Git closure history as the completion-time source. When history is absent, shallow, ambiguous or cannot establish identity after renumbering, report an unknown date rather than inventing a timestamp or silently using an unrelated date. Define the reporting window/timezone explicitly and distinguish cancellation from completed work.

## Acceptance criteria

- [ ] Resolved-work dates come from a verified done transition in Git history
- [ ] Missing/shallow/ambiguous history is explicitly unknown without a fabricated date or new completion field
- [ ] Window/timezone and cancelled-versus-done behavior are documented and tested

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:needs-info`; stream `cli`.

Deferred reporting feature. Decide authoritative completion timestamp and behavior when git history is unavailable; mtime is not proof of completion time.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

## Planning gate

Resolved by the owner's 2026-11-03 chat submission: **Use Git closure history; mark unknown dates**.
See [the decision record](../docs/decision-record-2026-11-03.md). Backlog remains deliberately deferred.

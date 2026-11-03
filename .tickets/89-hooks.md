---
id: "89"
title: "Lifecycle hooks: on-new, on-claim, on-close scripts"
status: backlog
blocked_by: []
priority: low
validation_criteria:
  - "Hooks are opt-in and advisory; failure does not block publishing or alter ticket lifecycle truth"
  - "Portable invocation, bounded execution and failure/retry behavior are explicit and tested"
  - "No configuration preserves current behavior and existing verification policy"
tags: ["integrations", "kind:enhancement", "stream:guidance", "triage:ready-for-agent"]
---

# Lifecycle hooks: on-new, on-claim, on-close scripts

## Owner decision — 2026-11-03

Confirmed in chat: **Start with advisory hooks**. This supersedes conflicting
requirements and the earlier dated decision gate; it does not claim implementation.
See [the authoritative record](../docs/decision-record-2026-11-03.md).

## Context

Taskwarrior's hook system creates a platform effect — time tracking, email notifications, mobile sync, and gamification all plug in. Hooks let users extend tkt without modifying the tool itself.

## What to build

Start with explicitly configured advisory lifecycle hooks. Publishing remains independent of hook success; define bounded portable invocation, timeout/failure reporting and retry behavior without changing ticket truth. Preserve the sequential runtime architecture and avoid an implicit global hook or mandatory verification gate. Default behavior remains unchanged without opt-in configuration.

## Acceptance criteria

- [ ] Hooks are opt-in and advisory; failure does not block publishing or alter ticket lifecycle truth
- [ ] Portable invocation, bounded execution and failure/retry behavior are explicit and tested
- [ ] No configuration preserves current behavior and existing verification policy

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:needs-info`; stream `integrations`.

Deferred lifecycle hooks, distinct from git-hook bypass in #235 and installer #157. Resolve the contradictory timeout/nonblocking-push contract and supported Windows invocation.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

## Planning gate

Resolved by the owner's 2026-11-03 chat submission: **Start with advisory hooks**.
See [the decision record](../docs/decision-record-2026-11-03.md). Backlog remains deliberately deferred.

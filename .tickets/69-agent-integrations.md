---
id: "69"
title: "Define the focused remaining harness-integration scope"
status: in_progress
blocked_by: ["68"]
priority: low
validation_criteria:
  - "Delivered integration pieces are identified with current source evidence"
  - "Remaining follow-up scope is bounded, tied to an observed need and has an explicit validation plan"
  - "Unneeded broad exploration is distinguished from real remaining work without fabricating completion"
tags: ["integrations", "kind:enhancement", "stream:quality", "triage:ready-for-agent"]
assignee: "codex"
---

# Define the focused remaining harness-integration scope

## Owner decision — 2026-11-03

Confirmed in chat: **Define a focused follow-up scope**. This supersedes conflicting
requirements and the earlier dated decision gate; it does not claim implementation.
See [the authoritative record](../docs/decision-record-2026-11-03.md).

## What to build

Inventory the delivered skill, ownership and session-context integrations from current source, then define a focused remaining follow-up scope tied to an observed gap. Prefer one or two bounded hypotheses with explicit acceptance evidence over a broad hooks/MCP exploration. Do not claim that all original integration possibilities were implemented, and do not build a new integration merely because it is possible.

## Acceptance criteria

- [ ] Delivered integration pieces are identified with current source evidence
- [ ] Remaining follow-up scope is bounded, tied to an observed need and has an explicit validation plan
- [ ] Unneeded broad exploration is distinguished from real remaining work without fabricating completion

## Pending decision (2026-10-29)

Partly delivered: 0.5.0 ships Claude, Codex and Antigravity plugins and a Claude
SessionStart hook that injects the frontier. What is missing is the recorded decision
(criterion 2). Write that decision, check what is met, and close; or retire via #182.

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:needs-info`; stream `integrations`.

Skills and plugins exist; the remaining deliverable is a recorded integration decision, not a second plugin implementation. Confirm which historical ACs are satisfied before closing or cancelling.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

## Disposition after the unblocked-work pass — 2026-11-02

Keep this deliberately deferred in backlog with its decision gate. Workflow plugins
and the Claude SessionStart hook are delivered; the remaining broader exploration
and maintenance tradeoffs are not evidenced as a completed spike. No new MCP or
cross-harness integration commitment is made by this pass. Owner input can choose
a bounded follow-up exploration or retirement; no implementation or research
completion is claimed here.

## Planning gate

Resolved by the owner's 2026-11-03 chat submission: **Define a focused follow-up scope**.
See [the decision record](../docs/decision-record-2026-11-03.md). Backlog remains deliberately deferred.


## Reactivation — 2026-11-03

The owner requested review of all parked work and execution of actionable tickets. This supersedes earlier blanket deferral. See [the parked-work review](../docs/parked-review-2026-11-03.md) for current scope and sequencing. Independent critic review is required before closure.

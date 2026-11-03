---
id: "148"
title: "Support primary-agent role pattern via init snippets and documentation"
status: open
blocked_by: []
priority: low
validation_criteria:
  - "Role examples use invocation identity, assignee and eligibility rather than machine capabilities for agent scheduling"
  - "Tags, requires, eligibility and ownership stay distinct, including explicit force overrides"
  - "Affected standalone guidance examples are consistent with current capabilities and claim behavior"
tags: ["guidance", "kind:enhancement", "stream:guidance", "triage:ready-for-agent"]
---

# Support primary-agent role pattern via init snippets and documentation

## Owner decision — 2026-11-03

Confirmed in chat: **Use invocation identity and eligibility**. This supersedes conflicting
requirements and the earlier dated decision gate; it does not claim implementation.
See [the authoritative record](../docs/decision-record-2026-11-03.md).

## What to build

Revise primary-agent role examples around invocation identity, optional ownership and agent eligibility. Keep tags for work streams and requires for all-of machine capabilities. Show safe force overrides, role coordination and claims without using machine capabilities to schedule agent identities. Update affected standalone instruction/skill examples coherently.

## Acceptance criteria

- [ ] Role examples use invocation identity, assignee and eligibility rather than machine capabilities for agent scheduling
- [ ] Tags, requires, eligibility and ownership stay distinct, including explicit force overrides
- [ ] Affected standalone guidance examples are consistent with current capabilities and claim behavior

## Research

Findings in `.scratch/research/`:
- `agent-role-scoping.md` — TBAC, Linear/Jira models, enforcement patterns
- `capabilities-as-roles.md` — ABAC vs RBAC, why flat capabilities > role hierarchies
- `multi-agent-roles.md` — CIV pattern, PER pipeline, 4-persona system
- `common-agent-roles.md` — industry role taxonomy (5 primary + extended)
- `primary-secondary-collaboration.md` — handoff protocols, Code Mower, ACS
- `ticket-visibility-patterns.md` — Freshdesk scope, exclusion rules, guardrails
- `role-config-deployment.md` — per-agent files, permission scoping, persona patterns

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:needs-info`; stream `guidance`.

init --role observer exits 2 (missing feature). The old use of machine capabilities for agent scheduling conflicts with #224's separate identity/eligibility contract. Redesign examples before implementation.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

## Planning gate

Resolved by the owner's 2026-11-03 chat submission: **Use invocation identity and eligibility**.
See [the decision record](../docs/decision-record-2026-11-03.md). Backlog remains deliberately deferred.


## Reactivation — 2026-11-03

The owner requested review of all parked work and execution of actionable tickets. This supersedes earlier blanket deferral. See [the parked-work review](../docs/parked-review-2026-11-03.md) for current scope and sequencing. Independent critic review is required before closure.

---
id: "290"
title: "Unify ticket authority, selection, and external-source workflow"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "legacy external execution and provider-close variables are absent from tkt-work"
  - "orient, work, init, and core guidance agree on status, sequence, and selection authority"
  - "mise run check passes and guidance deployment succeeds"
tags: ["stream:workflow", "kind:bug", "triage:ready-for-agent"]
---
# Unify ticket authority, selection, and external-source workflow

## Intent source

Owner-approved audit on 2026-11-13 and implementation approval on 2026-11-15. `tkt-orient` gives plan ordering priority, `tkt-work` says to pick the first frontier row and calls the plan the status source of truth, and its external issue path can bypass local readiness and closure.

## Context

- `skills/tkt-orient/SKILL.md` owns orientation and proposal.
- `skills/tkt-work/SKILL.md` owns execution of the selected ticket.
- `skills/tkt-reconcile/SKILL.md` owns end-of-session ordering.
- `docs/plan.md` and ticket 187 establish ticket files as lifecycle truth and plans as derived status plus human sequencing.
- `src/commands/init.rs` embeds consumer-facing workflow snippets.
- Ticket 86 and `docs/github-import.md` provide the bounded GitHub import bridge.

## What to build

All agent-facing surfaces describe one contract: ticket files own lifecycle and evidence; the plan owns human sequencing; `ready` computes eligible candidates; orientation selects/proposes; work executes that selection. Local tickets are the execution source. External issues enter through import rather than a parallel GitHub/GitLab execution source or provider-close side effect.

Correct statements that all projects enforce validation criteria/evidence: describe the actual configurable defaults and distinguish this repository's stricter project policy. Align init snippets with orientation-aware selection without changing frontier sorting.

## Acceptance criteria

- [ ] `tkt-work` contains no `CREW_TICKET_SOURCES`, GitLab execution path, or `CREW_TICKET_SYNC` provider-close behavior.
- [ ] `tkt-orient` and `tkt-work` agree that ticket files own status/evidence and the plan owns sequencing.
- [ ] `tkt-work` no longer instructs agents to pick the first ready row independently of orientation.
- [ ] Init snippets and core workflow descriptions use the same authority/selection language.
- [ ] Close-policy guidance states actual defaults rather than universal enforcement.
- [ ] The full maintainer gate passes and workflow guidance is deployed.

## Out of scope

Removing GitHub import, changing claim races, changing priority/ID sorting, or adding provider synchronization.

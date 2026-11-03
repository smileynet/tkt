---
id: "240"
title: "Show workstreams and decision/evidence gates in the ticket graph"
status: in_progress
blocked_by: []
validation_criteria:
  - "Task graph distinguishes open unblocked work, ticket dependencies, decision gates and evidence gaps without changing frontier ordering"
  - "Snapshot includes every ticket and graph workstreams preserve visible cross-stream dependencies"
  - "Dashboard and graph model checks pass; unavailable rendered checks are explicitly unverified"
tags: ["tooling", "kind:enhancement", "triage:ready-for-agent", "stream:quality", "gate:evidence"]
---

# Show workstreams and planning gates in the ticket graph

## What to build

Organize the all-ticket dashboard graph into workstreams, with unblocked work,
unfinished ticket prerequisites, explicit decisions and missing evidence shown
separately. Preserve lifecycle status and the priority/ID frontier. Deferred work
stays visible and deferred. No dependency edge is added for display ordering.

## Context

Intent: owner request on 2026-11-02 after the board reconciliation. Ticket 228
introduced the Plan tab; this task adds planning-gate and workstream semantics.
Canonical renderer, snapshot helper and graph fixtures live in grey-matter's
delivery-dashboard skill. Ticket files supply stream/gate tags and gate reasons;
the dashboard snapshot is a projection, not a second planning database.

## Acceptance criteria

- [x] Workstream lanes partition all remaining tickets without hiding cross-stream edges.
- [x] Decision/evidence gates have specific ticket-owned reasons and are not reported as unblocked work.
- [x] Ticket dependencies, completed blockers, deferred work and lifecycle status keep their distinct meanings.
- [x] Snapshot regeneration preserves all entries and includes every ticket.
- [x] Graph fixtures, dashboard checker, strict ticket validation and plan checks pass.
- [ ] Rendered laptop/phone layout and interactions are inspected when a permitted browser is available.

## Planning gate

Model and source checks pass; rendered workstream layout remains unverified while local browser use is disabled.

## Implementation evidence — 2026-11-02

Canonical contract fixtures pass (38 tests, no skips). The adopter model audit passes
with 240 tickets, 67 unique unfinished workstream memberships, 18 actual dependency
edges and the unchanged 30-ID CLI frontier. Its exclusive buckets are 26 unblocked,
7 dependency-blocked, 2 decision-gated, 2 evidence-gated, 2 in progress and 28 deferred.
All 17 earlier delivery entries remain unchanged; snapshot regeneration is repeatable.
See [durable graph evidence](../docs/evidence/dashboard-workstream-gates-240.md).

Rendered layout/interactions and the browser gate remain unavailable because local
browser use is disabled. Keep rendered acceptance unverified, without asking the
user for dashboard review. Temporary scripts/results are promoted into the evidence
above, then sent to OS trash; no captures or new runtime dependencies were created.

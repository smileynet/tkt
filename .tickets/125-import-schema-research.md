---
id: "125"
title: "Design the first external-system import adapter contract"
status: in_progress
blocked_by: ["77"]
priority: medium
validation_criteria:
  - "One identified source/version has an explicit adapter or mapping contract grounded in primary documentation and a real use case"
  - "Fixtures cover identity, state/relations, unsupported values and unknown-field preservation without pretending the nonexistent map interface exists"
  - "The first-adapter scope and remaining systems are separated with a concrete implementation/verification plan"
tags: ["integrations", "kind:enhancement", "stream:quality", "triage:ready-for-agent"]
assignee: "codex"
---

# Design the first external-system import adapter contract

## Owner decision — 2026-11-03

Confirmed in chat: **Design one explicit adapter contract first**. This supersedes conflicting
requirements and the earlier dated decision gate; it does not claim implementation.
See [the authoritative record](../docs/decision-record-2026-11-03.md).

## Problem

`tkt migrate` (ticket 77) establishes the migration infrastructure with `tk` as the first adapter. To reduce switching costs and enable adoption, tkt needs import compatibility with popular ticket systems that teams are already using.

## What to build

Define one explicit, versioned external-system adapter/mapping contract before broadening template research. Identify the initial source from recorded use cases and document any assumption; use primary source documentation and representative fixtures. Specify identity, status/dependency/parent mapping, unsupported values, unknown-field preservation and migration preview. Additional systems remain outside this first adapter scope.

## Context

- **Depends on:** ticket 77 (migrate infrastructure must exist first)
- **Related:** ticket 86 (GitHub import — one-way pull, different from schema migration)
- **Architecture:** `migrate.toml` is the seam — agents research and produce the map, CLI applies it mechanically

## Acceptance criteria

- [ ] One identified source/version has an explicit adapter or mapping contract grounded in primary documentation and a real use case
- [ ] Fixtures cover identity, state/relations, unsupported values and unknown-field preservation without pretending the nonexistent map interface exists
- [ ] The first-adapter scope and remaining systems are separated with a concrete implementation/verification plan

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:needs-info`; stream `integrations`.

Deferred research depends on an unimplemented --map interface: migrate --map example.toml exits 2. First choose adapter/mapping contract; no third-party schema facts were verified in this audit.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

## Planning gate

Resolved by the owner's 2026-11-03 chat submission: **Design one explicit adapter contract first**.
See [the decision record](../docs/decision-record-2026-11-03.md). Backlog remains deliberately deferred.


## Reactivation — 2026-11-03

The owner requested review of all parked work and execution of actionable tickets. This supersedes earlier blanket deferral. See [the parked-work review](../docs/parked-review-2026-11-03.md) for current scope and sequencing. Independent critic review is required before closure.

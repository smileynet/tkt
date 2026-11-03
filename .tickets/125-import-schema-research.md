---
id: "125"
title: "Design the first external-system import adapter contract"
status: done
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

## Current source correction — 2026-11-03

There is no migrate.toml/--map executor in the current CLI. The first external adapter design is github-issues/v1, grounded in the explicitly requested one-way GitHub import in ticket 86. See [the contract](../docs/github-adapter-contract.md). This supersedes the historical architecture assertion below; existing tk migration remains separate.

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

- [x] One identified source/version has an explicit adapter or mapping contract grounded in primary documentation and a real use case
- [x] Fixtures cover identity, state/relations, unsupported values and unknown-field preservation without pretending the nonexistent map interface exists
- [x] The first-adapter scope and remaining systems are separated with a concrete implementation/verification plan

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:needs-info`; stream `integrations`.

Deferred research depends on an unimplemented --map interface: migrate --map example.toml exits 2. First choose adapter/mapping contract; no third-party schema facts were verified in this audit.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

## Planning gate

Resolved by the owner's 2026-11-03 chat submission: **Design one explicit adapter contract first**.
See [the decision record](../docs/decision-record-2026-11-03.md). Backlog remains deliberately deferred.


## Reactivation — 2026-11-03

The owner requested review of all parked work and execution of actionable tickets. This supersedes earlier blanket deferral. See [the parked-work review](../docs/parked-review-2026-11-03.md) for current scope and sequencing. Independent critic review is required before closure.

## Resolution (2026-11-03)

Designed github-issues/v1 adapter for explicit86usecase, no generic map/import runtime claimed; independent critic accepted final mapping and12syntheticcases.

### Verification
1. ✓ One identified source/version has an explicit adapter or mapping contract grounded in primary documentation and a real use case — "The GitHub adapter contract pins REST 2026-03-10, cites official sources and the explicit request in ticket 86. Current migrate help supports only tk; no generic map executor is claimed."
2. ✓ Fixtures cover identity, state/relations, unsupported values and unknown-field preservation without pretending the nonexistent map interface exists — "Twelve synthetic cases cover identity/replay, state, pull requests, unknown/null fields, relations, provenance conflicts and mid-pagination changes. They are design requirements, not passing importer tests."
3. ✓ The first-adapter scope and remaining systems are separated with a concrete implementation/verification plan — "The implementation brief agrees with the design and gives staged verification requirements. Independent critic review accepted the final artifacts; docs-only accuracy, links and diff checks passed."

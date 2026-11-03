---
id: "142"
title: "Fix CREW_ENV=either hides everything and ac edge cases"
status: cancelled
blocked_by: []
priority: low
validation_criteria:
  - "CREW_ENV=either shows all tickets"
tags: ["cli", "kind:bug", "triage:needs-info", "stream:quality", "gate:evidence"]
---

# Fix CREW_ENV=either hides everything and ac edge cases

## What to build

Re-triage 2026-10-29: not reproduced. In a scratch repo with three tickets (one with
`requires: [gpu]`), `CREW_ENV=either tkt ready` on 0.5.0 showed two of three rather than
none. The "ac edge cases" in the title are not described anywhere.

## Acceptance criteria

- [ ] A reproduction (inputs, command, observed output) is recorded, or the ticket is retired via #182

## Triage — 2026-11-02

Disposition: `kind:bug`, `triage:needs-info`; stream `cli`.

Earlier no-reproduction result remains unresolved. Supply a minimal CREW_ENV=either fixture and expected capability semantics; the unspecified AC edge cases are not build scope.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

## Planning gate

Capture a minimal CREW_ENV=either reproduction with the expected capability semantics.

## Cancellation

- Reason: wontfix
- Note: No current defect demonstrated: fresh CREW_ENV=either fixture returns plain/either tickets and correctly excludes a GPU requirement. Historical hidden-all report and unspecified AC edge cases remain unverified. Reopen with a minimal counterexample and expected semantics.
- Date: 2026-11-03

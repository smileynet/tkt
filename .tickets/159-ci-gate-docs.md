---
id: "159"
title: "Document optional consumer CI with accurate enforcement limits"
status: done
blocked_by: []
priority: low
validation_criteria:
  - "An explicitly optional consumer recipe runs strict validation and lint checks without adding an automatic workflow here"
  - "Guidance accurately explains bypass and configuration limits without calling CI universally unbypassable"
  - "Affected guidance surfaces agree and this repository verification/release authorization remains unchanged"
tags: ["docs", "guidance", "kind:enhancement", "stream:guidance", "triage:ready-for-agent"]
assignee: "codex"
---

# Document optional consumer CI with accurate enforcement limits

## Owner decision — 2026-11-03

Confirmed in chat: **Document optional consumer CI and its limits**. This supersedes conflicting
requirements and the earlier dated decision gate; it does not claim implementation.
See [the authoritative record](../docs/decision-record-2026-11-03.md).

## What to build

Document an opt-in consumer recipe for validation/lint checks and accurate bypass limits. Keep this repository on its existing local verification/manual hosted-dispatch policy. Do not add an automatic hosted workflow here or claim that CI is universally unbypassable. One authoritative enforcement-model note is referenced from the appropriate surfaces.

## Context

- **Relevant files:** `README.md`, `AGENTS.md`, possibly `skills/tkt-core/references/commands.md`
- Research: `.scratch/subagent-raw/guardrail-warn-vs-block.md` (open question 2 — confirm projects actually wire CI, or blocks are advisory-only in practice)
- Complements #157/#158 (which are the warn-layer); this documents where the block-layer actually lives.

## Acceptance criteria

- [x] An explicitly optional consumer recipe runs strict validation and lint checks without adding an automatic workflow here
- [x] Guidance accurately explains bypass and configuration limits without calling CI universally unbypassable
- [x] Affected guidance surfaces agree and this repository verification/release authorization remains unchanged

## Out of scope

- Implementing hooks (#157/#158)

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:needs-info`; stream `guidance`.

Old automatic-PR-CI prescription conflicts with current standing manual-dispatch-only verification rule. Propose optional consumer enforcement and accurate bypass limits; do not add hosted CI or describe it as universally unbypassable.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

## Planning gate

Resolved by the owner's 2026-11-03 chat submission: **Document optional consumer CI and its limits**.
See [the decision record](../docs/decision-record-2026-11-03.md). Implementation remains open; the owner-policy gate is cleared.

## Resolution (2026-11-03)

Documented optional consumer CI and accurate limits with one authoritative enforcement note and standalone skill summary. Critic accepted; probe proof promoted to docs/evidence/consumer-enforcement-159.md. Probe repositories and helpers/logs trashed. Independently observed plan comparison defect captured in open ticket 248, without activating deferred narrative parsing.

### Verification
1. ✓ An explicitly optional consumer recipe runs strict validation and lint checks without adding an automatic workflow here — "Current target CLI probe proves strict validation and check lint positive/negative exits; optional plan step documented; docs/evidence/consumer-enforcement-159.md"
2. ✓ Guidance accurately explains bypass and configuration limits without calling CI universally unbypassable — "Official GitHub primary pages HTTP200 support bounded bypass/skip claims; independent critic accepted note identity"
3. ✓ Affected guidance surfaces agree and this repository verification/release authorization remains unchanged — "Applicable surfaces updated; no workflow/code/hook/release authorization changed; existing automatic trigger fact disclosed; validate and plan checks pass"

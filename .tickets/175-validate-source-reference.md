---
id: "175"
title: "validate: advisory no-source-reference finding (union of spec/#NN/parent/link)"
status: done
blocked_by: ["174"]
priority: medium
validation_criteria:
  - "validate warns when a ticket has no spec, no blocked_by, and no #NN/docs/.memory reference in body (test: integration::validate_no_source_ref)"
  - "any one reference kind satisfies the check (test: integration::validate_source_ref_union)"
  - "advisory by default (exit 0); [validate] config key opts into hard gate (test: integration::validate_source_ref_gate)"
tags: ["quality", "kind:enhancement", "triage:ready-for-agent", "stream:guidance"]
assignee: "codex"
---

# validate: advisory no-source-reference finding (union of spec/#NN/parent/link)

## Current advisory contract — 2026-11-03

Only open/in-progress tickets are checked; backlog and terminal work are exempt. The finding has info severity by default, remaining advisory under --strict. validate.require_source_reference=true explicitly makes absence an error. The union includes nonempty spec, blocked_by, parent, local body citation, docs/specs/.memory path or HTTP(S) link outside fenced examples. Presence is separate from existing target-integrity rules. This clarifies the historical word warn below and supersedes any implication of a default strict-mode gate.

## Context

Parent: #172. No tkt rule requires a ticket to point at its source of truth. Fields:
`blocked_by` (integrity-checked but optional), `spec` (optional free-text, only
length/charset-validated at new.rs:31, never checked as a real reference),
validation_criteria (presence configurable). A ticket with empty blocked_by, no spec,
and a stub body is fully valid across all health commands. Corpus: 12 of 25 open
tickets have no source reference at all (79, 83, 87, 88, 89, 90, 127, 161, 163, 164, 165, 172).

## Prior art (research: .scratch/research/traceability.md)

The upstream source link ("why does this exist?") is the one worth enforcing — it
catches scope creep. Capture live at creation, not reconstructed before review (the
#1 documented failure mode is a stale after-the-fact matrix). Accept a UNION of link
kinds; enforce advisory-first with hard-gate opt-in (mirrors tkt's [close] config).

## What to build

Add a `no-source-reference` advisory finding to `validate`: warn when a ticket has
none of — a non-empty `spec:` field, a non-empty `blocked_by`, or a body reference
(`#NN`, `docs/`, `.memory/`, `specs/`, or a URL). Any one satisfies it. Advisory
(exit 0) by default; add a `[validate] require_source_reference` config key to opt
into a hard gate. Keep refs machine-checkable so a later pass can flag dangling refs
like blocked_by cycles.

Blocked by #174 (lands the validate-finding scaffold first).

## Acceptance criteria

- [x] `validate` warns when a ticket has no spec, no blocked_by, and no body reference
- [x] Any one reference kind (spec / blocked_by / #NN / docs link) satisfies the check
- [x] Advisory by default (exit 0); `[validate] require_source_reference` opts into a hard gate
- [x] Spike/backlog exemption considered (research open question)
- [x] `mise run check` passes

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:ready-for-agent`; stream `quality`.

Deferred source-reference advisory after completed #174. Own reusable source-reference predicate; #173 should reuse it. Durable inline requirements suffice; missing historical scratch research is not required.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.


## Reactivation — 2026-11-03

The owner requested review of all parked work and execution of actionable tickets. This supersedes earlier blanket deferral. See [the parked-work review](../docs/parked-review-2026-11-03.md) for current scope and sequencing. Independent critic review is required before closure.

## Resolution (2026-11-03)

Implemented reusable source-reference advisory and explicit opt-in policy; final full gate and independent critic accepted.

### Verification
1. ✓ validate warns when a ticket has no spec, no blocked_by, and no #NN/docs/.memory reference in body (test: integration::validate_no_source_ref) — "Active absent-source fixture yields info with strict exit0; explicit policy yields error exit1. Final mise run check403Rust/42Node passed,1existingWindows skip. docs/evidence/source-reference-175.md."
2. ✓ any one reference kind satisfies the check (test: integration::validate_source_ref_union) — "Eight union sources covered:spec,deps,parent,localcitation,docs,.memory,specs,HTTP link. Fenced-only references rejected; terminal/backlog excluded; shared citation regressions pass."
3. ✓ advisory by default (exit 0); [validate] config key opts into hard gate (test: integration::validate_source_ref_gate) — "Defaultinfo never escalates understrict; typedvalidate.require_source_reference config explicitly errors. Guidance/init surfaces align; critic accepted final product and fixture correction."

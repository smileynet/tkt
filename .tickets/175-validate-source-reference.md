---
id: "175"
title: "validate: advisory no-source-reference finding (union of spec/#NN/parent/link)"
status: open
blocked_by: ["174"]
priority: medium
validation_criteria:
  - "validate warns when a ticket has no spec, no blocked_by, and no #NN/docs/.memory reference in body (test: integration::validate_no_source_ref)"
  - "any one reference kind satisfies the check (test: integration::validate_source_ref_union)"
  - "advisory by default (exit 0); [validate] config key opts into hard gate (test: integration::validate_source_ref_gate)"
---

# validate: advisory no-source-reference finding (union of spec/#NN/parent/link)

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

- [ ] `validate` warns when a ticket has no spec, no blocked_by, and no body reference
- [ ] Any one reference kind (spec / blocked_by / #NN / docs link) satisfies the check
- [ ] Advisory by default (exit 0); `[validate] require_source_reference` opts into a hard gate
- [ ] Spike/backlog exemption considered (research open question)
- [ ] `mise run check` passes

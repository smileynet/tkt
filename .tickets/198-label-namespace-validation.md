---
id: "198"
title: "label-namespace validation: single-select + closed-set"
status: done
blocked_by: ["193"]
priority: high
validation_criteria:
  - "check_label_namespaces flags a namespaced tag whose value is outside the declared set"
  - "A single-select namespace appearing more than once on a ticket is a validate error"
  - "Tags outside any declared namespace are unaffected; tests mirror existing findings tests"
tags: ["config", "kind:enhancement", "triage:ready-for-agent", "stream:config"]
---

# label-namespace validation: single-select + closed-set

## What to build

Enforce the label namespaces parsed in ticket 193, via `tkt validate`.

- `src/findings.rs`: `check_label_namespaces` -> a tag whose prefix matches a
  declared namespace must use a value from that namespace's closed set; a
  `single_select` namespace may appear at most once per ticket.
- `src/commands/validate.rs`: wire into `collect_findings`.
- Tags outside any declared namespace are unaffected (free-form as today).

This is the native replacement for the external `jq` "exactly one triage tag"
lint script.

## Acceptance criteria

- [x] `check_label_namespaces` flags a namespaced tag whose value is outside the declared set
- [x] A single-select namespace appearing more than once on a ticket is a validate error
- [x] Tags outside any declared namespace are unaffected; tests mirror existing findings tests

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:ready-for-agent`; stream `config`.

Keep dependency #193. Validate declared namespaces only; unrelated free-form tags retain their existing behavior.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

## Resolution (2026-11-03)

Declared label values and cardinality validated; independent critic accepted. Source/tests/evidence promoted, fixtures self-clean, shared target retained for queue. docs/evidence/parent-cli-labels-debug-196-198-237.md.

### Verification
1. ✓ check_label_namespaces flags a namespaced tag whose value is outside the declared set — "Public closed-value fixture red before, passes after."
2. ✓ A single-select namespace appearing more than once on a ticket is a validate error — "Single-select violation public fixture passes."
3. ✓ Tags outside any declared namespace are unaffected; tests mirror existing findings tests — "Undeclared/free tags, multiselect and emptyset fixtures pass; full local gate passed."

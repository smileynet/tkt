---
id: "198"
title: "label-namespace validation: single-select + closed-set"
status: open
blocked_by: ["193"]
priority: high
validation_criteria:
  - "check_label_namespaces flags a namespaced tag whose value is outside the declared set"
  - "A single-select namespace appearing more than once on a ticket is a validate error"
  - "Tags outside any declared namespace are unaffected; tests mirror existing findings tests"
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

- [ ] `check_label_namespaces` flags a namespaced tag whose value is outside the declared set
- [ ] A single-select namespace appearing more than once on a ticket is a validate error
- [ ] Tags outside any declared namespace are unaffected; tests mirror existing findings tests

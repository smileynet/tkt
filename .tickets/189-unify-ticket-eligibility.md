---
id: "189"
title: "Unify env and requires into one ticket eligibility model"
status: done
blocked_by: []
priority: high
validation_criteria:
  - "Mixed env and requires receives an explicit, unambiguous boundary outcome"
  - "ready includes tickets when their canonical eligibility is satisfied"
  - "Legacy env, capability, and config inputs have compatibility tests"
  - "The design decision records compatibility and migration behavior"
tags: ["frontier", "data-model"]
---

# Unify env and requires into one ticket eligibility model

## What to build

`env` and `requires` currently express overlapping but separately evaluated eligibility rules.
A ticket that carries both can be rejected by the legacy environment filter before capability
matching, even when the configured machine capabilities satisfy its intent. That makes a
frontier omission ambiguous to both people and automation.

Choose and record one canonical eligibility representation (for example a capability set with
a legacy-environment adapter), or reject/normalize mixed `env` + `requires` at the input
boundary. The resulting model must be the sole source used by `ready`, context/environment
filtering, config defaults, and serialization. Preserve a deliberate compatibility path for
existing environment-only tickets rather than changing their eligibility silently.

Related #142 remains a narrow legacy `CREW_ENV=either` edge case; this ticket owns the broader
data-model decision and implementation.

## Acceptance criteria

- [x] Mixed `env` and `requires` receives an explicit, unambiguous boundary outcome
- [x] `ready` evaluates one canonical eligibility model and includes a satisfied ticket
- [x] Environment-only legacy tickets and configuration defaults retain documented compatibility
- [x] Capability, environment, mixed-field, and no-match cases have regression coverage
- [x] The design decision records serialization and migration behavior

## Resolution (2026-10-19)

Unified frontier eligibility around the requires capability set while preserving env-only compatibility.

### Verification
1. ✓ Mixed env and requires receives an explicit, unambiguous boundary outcome — "Mixed non-either env plus requires is rejected by parser, new, and batch; regression coverage verifies the boundary."
2. ✓ ready includes tickets when their canonical eligibility is satisfied — "ready now merges machine capabilities with the legacy CREW_ENV/default_env value once, then performs a single capability eligibility check."
3. ✓ Legacy env, capability, and config inputs have compatibility tests — "Legacy env, explicit capability, no-match, and mixed input regression coverage passes in mise run check (172 unit, 95 integration)."
4. ✓ The design decision records compatibility and migration behavior — "README and AGENTS.md document the compatibility and migration rule: move combined constraints into requires."

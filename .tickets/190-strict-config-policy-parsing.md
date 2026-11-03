---
id: "190"
title: "Parse known config keys into strict typed policies"
status: done
blocked_by: []
priority: high
validation_criteria:
  - "Invalid known config values fail with structured field context"
  - "Evidence, push, priority, and environment policies use typed parsed values"
  - "A typo cannot silently disable required evidence or publishing"
  - "Configuration compatibility cases are covered by tests"
tags: ["config", "integrity", "kind:bug", "triage:ready-for-agent", "stream:config"]
---

# Parse known config keys into strict typed policies

## What to build

Known project and user configuration keys are read as raw strings in several policy paths.
Typos such as `push.enabled = "treu"` can currently collapse to false, while an unrecognized
evidence value can disable a required gate. The configuration boundary must either produce a
valid typed domain value or return a structured, actionable error; it must not silently choose
a permissive policy.

Parse booleans, evidence policy, priority, environment, and other closed-domain keys into
typed values once while retaining source/key context for diagnostics. Surface validation through
the ordinary configuration/command error path, including merged user/project settings. Keep
#139 focused on evidence-gate semantics and #165 on safely writing project-scoped config; this
ticket owns safe reading and interpretation.

## Acceptance criteria

- [x] Invalid known config values fail with structured key, value, and source context
- [x] Evidence, push, priority, and environment policies consume typed parsed values
- [x] A typo cannot silently disable required evidence, publishing, or a filter
- [x] Unknown-key compatibility behavior is documented and tested separately from known keys
- [x] User/project precedence and valid legacy values have regression coverage

## Triage — 2026-11-02

Disposition: `kind:bug`, `triage:ready-for-agent`; stream `config`.

Confirmed: push.enabled="treu" followed by config --show exits 0 and reports push.enabled="false" (project). Parse at the config boundary; do not silently weaken policy.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

## Resolution (2026-11-03)

Known effective policies are typed and validated contextually without weakening gates. Config repair and environment precedence fixed after independent critic findings. Source/tests/evidence promoted; fixtures self-clean, shared target retained through queue. docs/evidence/config-tags-diagnostics-190-233-151.md.

### Verification
1. ✓ Invalid known config values fail with structured field context — "Public malformed policy fixtures report key/value/source Validation errors."
2. ✓ Evidence, push, priority, and environment policies use typed parsed values — "Bool/EvidencePolicy/optional Priority Env u64 consumers replace raw interpretation."
3. ✓ A typo cannot silently disable required evidence or publishing — "Bad push/evidence fixtures reject creation/close without writing; no permissive fallback."
4. ✓ Configuration compatibility cases are covered by tests — "Seven config policy fixtures cover precedence, unknown warnings, repair and overrides; final gate350Rust42Nodepass one existing skip."

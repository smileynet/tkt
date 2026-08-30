---
id: "139"
title: "Fix evidence gates partially dead: unreachable warn branch + silent discard"
status: open
blocked_by: []
priority: medium
validation_criteria:
  - "partial evidence triggers warning correctly"
  - "evidence on criteria-less ticket produces error"
---

# Fix evidence gates partially dead: unreachable warn branch + silent discard

> Source: #128 **F10** (P2, *verified*, 2026-08-23 architecture audit). #128 is done; evidence + fix below.
> **Coordination:** the partial-evidence **error-kind** fix (bare `domain_bail!` → `GateFailed`,
> #149 F-R4 / #152 F5) is now owned by **#170** (close-gate rework, telemetry-driven), which
> touches the same `close.rs` gate lines. Leave the error kind to #170 to avoid a collision;
> this ticket owns only the dead-branch + silent-discard behavior. Coordinate on merge order.

## What to build

The close command's evidence gates must actually run. Today two paths are dead/wrong:

1. **Unreachable warn branch** — `parse_evidence` (`close.rs:331-341`) bails unconditionally
   when any evidence slot is missing, so the config severity switch (`warn`/`false`) for
   partial evidence (`close.rs:94-102`) is never reached. Partial evidence can never
   degrade to a warning as configured.
2. **Silent discard** — `close X --evidence` on a criteria-less ticket silently discards the
   input (`close.rs:47-51`) instead of warning that evidence was given with no criteria to map.

Restructure so the completeness check lives inside the severity-handled gate (respecting
`warn`/`true`/`false`), and warn when evidence is provided without criteria. The
partial-evidence **error kind** (→ `GateFailed`) is out of scope here — **#170** owns it as
part of its close-gate rework on the same lines.

## Context

- **Location (#128 F10, verified):** `src/commands/close.rs:331-341` (early bail), `:94-102` (unreachable severity branch), `:47-51` (silent discard); #149 F-R4 error kind at the partial-evidence bail (~`close.rs:82`).
- **Fix (#128):** move the completeness check into the severity-handled gate; warn on evidence-without-criteria; correct the error kind.

## Acceptance criteria

- [ ] Partial evidence under `require_validation_evidence = "warn"` emits a warning (not a hard bail)
- [ ] `close --evidence` on a criteria-less ticket warns that evidence was discarded
- [ ] The previously-unreachable severity branch is now reachable (or removed if truly dead) with a test proving it
- [ ] Regression tests for warn / true / false severities and the criteria-less case
- [ ] cargo fmt && cargo clippy --all-targets && cargo test pass clean

(Partial-evidence error-kind → `GateFailed` is verified in #170's ACs, not here.)

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
> Also folds in #149 **F-R4** (per #152 F5): correct the partial-evidence bail's error kind during this restructure.

## What to build

The close command's evidence gates must actually run. Today two paths are dead/wrong:

1. **Unreachable warn branch** — `parse_evidence` (`close.rs:331-341`) bails unconditionally
   when any evidence slot is missing, so the config severity switch (`warn`/`false`) for
   partial evidence (`close.rs:94-102`) is never reached. Partial evidence can never
   degrade to a warning as configured.
2. **Silent discard** — `close X --evidence` on a criteria-less ticket silently discards the
   input (`close.rs:47-51`) instead of warning that evidence was given with no criteria to map.
3. **Wrong error kind (#149 F-R4)** — the partial-evidence bail uses bare `domain_bail!`
   (defaults to `Validation`) where `GateFailed` is correct, matching the no-evidence gate.

Restructure so the completeness check lives inside the severity-handled gate (respecting
`warn`/`true`/`false`), warn when evidence is provided without criteria, and use `GateFailed`.

## Context

- **Location (#128 F10, verified):** `src/commands/close.rs:331-341` (early bail), `:94-102` (unreachable severity branch), `:47-51` (silent discard); #149 F-R4 error kind at the partial-evidence bail (~`close.rs:82`).
- **Fix (#128):** move the completeness check into the severity-handled gate; warn on evidence-without-criteria; correct the error kind.

## Acceptance criteria

- [ ] Partial evidence under `require_validation_evidence = "warn"` emits a warning (not a hard bail)
- [ ] Partial evidence under `"true"` bails with `ErrorKind::GateFailed` (not `Validation`)
- [ ] `close --evidence` on a criteria-less ticket warns that evidence was discarded
- [ ] The previously-unreachable severity branch is now reachable (or removed if truly dead) with a test proving it
- [ ] Regression tests for warn / true / false severities and the criteria-less case
- [ ] cargo fmt && cargo clippy --all-targets && cargo test pass clean

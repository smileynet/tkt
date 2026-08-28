---
id: "166"
title: "validate --fix: abort if a fix introduces new findings (non-regressing gate)"
status: open
blocked_by: ["140"]
priority: high
validation_criteria:
  - "validate --fix compares finding identities (file+rule) before/after; aborts exit 1 if any NEW finding appears (test: fix::regression_gate_trips)"
  - "normal fixes that only reduce findings are unaffected (existing --fix tests stay green)"
  - "regression abort advises git checkout .tickets/ and does not use a new exit code (stays within 0/1/2 taxonomy)"
tags: ["contract"]
---

# validate --fix: abort if a fix introduces new findings (non-regressing gate)

## What to build

Hardening for `tkt validate --fix`. Today `run_with_fix` applies repairs (fix.rs writes
per-file eagerly) then re-runs validate to *show* remaining state, but never checks
whether the fix made the corpus WORSE. #162's unique-match guard makes regression
unlikely today, but this is a standing tripwire that protects ALL fix tiers, present
and future.

Research + code/docs review (2026-08-28, `.scratch/fix1-research/`, `.scratch/fix1-review/`):

### Compare finding IDENTITIES, not counts
A raw count can stay equal while one finding is swapped for a worse one (ESLint's
circular-fix guard exists for exactly this). Regression = any finding in `after` not
present in `before`, keyed by identity `(file, rule)`. tkt's `Finding { file, rule,
message, severity }` — `(file, rule)` is a stable identity.

### Exit code 1 (domain), NOT a new code
AGENTS.md:126 defines a closed taxonomy: 0=success, 1=domain failure, 2=operational
crash (I/O/git/parse only), enforced in main.rs + published in capabilities.rs. A
regression-abort is a domain failure -> exit 1 via DomainError. No surveyed tool
(cargo/eslint/prettier/rustfmt) signals "made worse" with a distinct code — cargo fix
reverts+warns. A new code would break the published contract.

### v1: git is the revert primitive
Full in-memory speculative-rollback (cargo fix / BitsAI model) needs fix::run_fix
refactored to defer writes (it currently writes per-file in-loop). v1: since .tickets/
is always git-tracked, on regression abort with advisory `git checkout .tickets/`.
Full speculative-rollback + per-file transactions + idempotence pass-cap deferred to a
follow-up ticket.

### Implementation
- Extract finding collection (validate.rs L13-38) into shared `collect_findings(&dir)
  -> Vec<Finding>` used by both `run()` and the pre-fix baseline (no drift). `strict`
  not needed in the collector — it only affects status/printing.
- Gate INSIDE the `!dry_run && !result.repairs.is_empty()` block so advisory-only (#154)
  and dry-run paths are untouched.
- On regression early-return, store RESULT_COUNT manually (else telemetry stays -1).

### Callers verified safe
doctor.rs does not call run_with_fix (--fix stub). capabilities.rs documents exit 2 for
io/parse only — using exit 1 sidesteps that gap. 3 existing --fix integration tests must
stay green; #162's test leaves 1 dangling (after ⊆ before) so the gate must NOT trip.

### Test
- unit: identity set-difference predicate (catches the swap case).
- integration: gate does NOT trip on normal reducing fixes (existing tests).
- integration: gate DOES trip — needs a `#[cfg(test)]`-only fix tier that deliberately
  introduces a dangling ref, proving abort + exit 1 + git-checkout advisory.

Note on blocked_by 140: 140 (exit-code taxonomy correctness) is the natural predecessor
for the "domain refusals = exit 1" principle, but this ticket's design already respects
the current taxonomy (exit 1), so it does not hard-depend on 140 landing first.

## Acceptance criteria

- [ ] `collect_findings` extracted; `run()` and pre-fix baseline share it
- [ ] `validate --fix` compares finding identities (file+rule) before/after
- [ ] any NEW finding after fix aborts with exit 1 (DomainError) + `git checkout .tickets/` hint
- [ ] normal reducing fixes unaffected; existing --fix tests stay green
- [ ] no new exit code (stays within 0/1/2)
- [ ] unit test on the regression predicate + integration test proving the gate trips (test: fix::regression_gate_trips)

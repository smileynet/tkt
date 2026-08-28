---
id: "166"
title: "validate --fix: abort if a fix introduces new findings (non-regressing gate)"
status: done
blocked_by: []
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

- [x] `collect_findings` extracted; `run()` and pre-fix baseline share it
- [x] `validate --fix` compares finding identities (file+rule) before/after
- [x] any NEW finding after fix aborts with exit 1 (DomainError) + `git checkout .tickets/` hint
- [x] normal reducing fixes unaffected; existing --fix tests stay green
- [x] no new exit code (stays within 0/1/2)
- [x] unit test on the regression predicate + integration test proving the gate trips (test: fix::regression_gate_trips)

## Resolution (2026-08-28)

validate --fix now captures finding identities (file+rule) before the fix pass and aborts with exit 1 + git-checkout advice if any NEW finding appears. Extracted shared collect_findings so run() and the baseline agree. Identity comparison catches swaps, not just count increases. v1 uses git as revert primitive; full speculative-rollback deferred. Removed soft blocked_by 140 — design respects current taxonomy (exit 1), no hard dep.

### Verification
1. ✓ validate --fix compares finding identities (file+rule) before/after; aborts exit 1 if any NEW finding appears (test: fix::regression_gate_trips) — "unit regression_detected_when_new_finding_appears + no_regression_when_findings_only_reduced + identity_is_file_plus_rule_not_message"
2. ✓ normal fixes that only reduce findings are unaffected (existing --fix tests stay green) — "existing --fix tests stay green: test_validate_fix_quotes_ids, test_fix_normalizes_blocked_by_padding_and_slug, test_validate_fix_advises_on_hand_flipped_done; 204 tests pass, clippy 0, fmt clean"
3. ✓ regression abort advises git checkout .tickets/ and does not use a new exit code (stays within 0/1/2 taxonomy) — "integration test_fix_regression_gate_aborts_on_new_finding: real closed->done trips gate, exit 1, message includes git checkout; installed tkt(e2d00d1)==HEAD verified exit 1 on regression / exit 0 on reducing fix"

---
id: "170"
title: "close: batch unmet gates into one message + populate hints + fix G5 mis-kinding"
status: done
blocked_by: []
priority: high
validation_criteria:
  - "close with multiple unmet gates lists all in one message (test: integration::close_gates_batched)"
  - "each close gate populates a hint naming the missing flag (test: integration::close_gate_hints)"
  - "G5 partial-evidence emits err=gate_failed not validation (test: integration::close_partial_evidence_kind)"
---

# close: batch unmet gates into one message + populate hints + fix G5 mis-kinding

## Context

Telemetry review (2026-08-30): `close` shows a systematic fail→retry pattern — `close exit=1 err=gate_failed` followed seconds later by a successful close (observed in recall, tkt, teach-me, game-design-salon). 47/172 close events show friction.

## Root cause (verified in source)

Gates in `close.rs::run` are a linear sequence of independent `domain_bail!` early-returns, each returning on the FIRST failing gate. No aggregation. A user missing two inputs gets two rejections and two retries — each rejection reveals only one missing flag. That is the telemetry pattern.

Gate order: G1 force-disabled (`close.rs:22`) → G2 resolution (`:29`) → G3 validation_criteria (`:52`) → G4 evidence-missing (`:60`) → G5 partial-evidence (`:82`) → G6 unchecked-ACs (`:141`).

Verified defects:
- **No `hint` is populated on ANY close gate** (`hint: None` everywhere) despite infra support (`common.rs:15-21`) and JSON envelope rendering (`cli.rs:905-908`).
- **G5 mis-kinded**: `close.rs:87` uses bare `domain_bail!("...")` → `ErrorKind::Validation`, not `GateFailed` like the other five gates. Telemetry keying on `err=gate_failed` undercounts it. (Verified by read.)
- G6 (`:141`) is the model to copy — it names all remedies inline (`--ac`, `--check-all`, `--force`).
- `capabilities.rs:166` advertises `gate_failed` as `retryable:false` — misleading; it IS fixable-then-retryable.

## Prior art (research: .scratch/research/close-gate-ux.md)

Consensus: **fail-fast — check all required fields up front and list every missing one at once.** Named error token, echo the offending state, copy-pasteable remedy placed last. For AI-agent consumers the error text becomes a retry-loop prompt — it must name exact flags.

## What to build

1. Aggregate unmet gates: evaluate all gates, collect failures, emit ONE message listing every missing input ("close needs: --resolution, --check-all, --evidence").
2. Populate `hint:` on each gate with the exact missing flag (minimum-viable first step even before full batching).
3. Fix G5 to use `GateFailed` (`domain_bail!(GateFailed, ...)`).
4. G3: name the remedy (`tkt edit <id> --validation ...`), not just `--force`.
5. Correct `capabilities.rs` `retryable` wording for gate_failed (fixable-then-retry).

## Acceptance criteria

- [x] A close blocked by multiple gates lists all missing inputs in one message
- [x] Each close gate populates a hint naming the missing flag
- [x] G5 partial-evidence emits `err=gate_failed` (not `validation`)
- [x] G3 message names how to add validation_criteria
- [x] `capabilities` retryable wording corrected for gate_failed
- [x] `cargo fmt && cargo clippy --all-targets && cargo test` clean

## Resolution (2026-08-30)

close gates now aggregate into one message ('close blocked by N unmet gate(s): ...') evaluated up front, so all missing inputs are fixed in one retry; every gate populates a hint naming the exact flags; G5 partial-evidence re-kinded from Validation to GateFailed; capabilities retryable wording clarified.

### Verification
1. ✓ close with multiple unmet gates lists all in one message (test: integration::close_gates_batched) — "cargo test: 69 passed 0 failed incl close_gates_batched/close_gate_hint_populated/close_partial_evidence_kind_is_gate_failed"
2. ✓ each close gate populates a hint naming the missing flag (test: integration::close_gate_hints) — "e2e installed binary 26e7083: bare close emits 'close blocked by 3 unmet gate(s)' in one message; JSON envelope has kind=gate_failed + hint naming --resolution/--evidence/--check-all"
3. ✓ G5 partial-evidence emits err=gate_failed not validation (test: integration::close_partial_evidence_kind) — "clippy --all-targets clean; rustfmt --check clean across src+tests"

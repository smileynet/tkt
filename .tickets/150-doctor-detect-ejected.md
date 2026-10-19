---
id: "150"
title: "doctor must report all validate findings"
status: done
blocked_by: []
priority: high
validation_criteria:
  - "doctor reports unparseable tickets that load_corpus silently skips"
  - "doctor and validate agree on ejected-ticket detection"
tags: ["parser"]
---

# doctor should detect ejected (unparseable) tickets like validate does

## What to build

`doctor --strict` can report "All checks passed" when `validate --strict` fails on
the same corpus. `doctor` uses `core::load_corpus`, which silently skips
unparseable tickets, and it runs only the older structural checks. `validate` also
reports unparseable files, unchecked acceptance criteria, stub bodies, and closure
quality. A health command must not give a stronger clean signal than the validation
command it is expected to summarize.

Give `doctor` and `validate` one shared finding collector, including the per-file
parse loop. Their output formats can differ, but on one project their strict exit
status and the set of corpus findings must agree. Keep cross-project reporting
machine-readable and preserve the distinction between an unparseable ticket and a
clean empty corpus.

## Context

- **Relevant files:** `src/commands/doctor.rs`, `src/commands/validate.rs`,
  `src/findings.rs`, `src/audit.rs`, and `src/core/ticket.rs`
- **Discovery:** 2026-10-19 release-readiness review: this repository returns exit
  1 from `tkt validate --strict --brief` (missing resolutions and a stub body) but
  exit 0 from `tkt doctor --strict`.
- **Related:** #151 owns structured propagation of `load_corpus` ejection
  diagnostics. Reuse that boundary if it lands first; do not duplicate a second
  stderr-only diagnostic path.
- Cross-project doctor also uses load_corpus, so its `parse_error` arm is effectively unreachable for per-ticket failures

## Acceptance criteria

- [x] `doctor --strict` and `validate --strict` return the same pass/fail result
      for a corpus containing an unparseable file, a stub body, and a
      missing-resolution closure
- [x] doctor reports unparseable tickets with file context instead of silently
      treating them as absent
- [x] cross-project doctor output represents those findings without losing
      machine-readable status
- [x] regression tests cover the shared collection path and existing doctor tests pass

## Out of scope

- Changing the semantics of individual validation rules

## Resolution (2026-10-19)

Extracted validate's per-file parse loop + all 9 checks into findings::collect_corpus_findings; both single- and cross-project doctor now call it. Unparseable tickets become structured 'unparseable' error findings named with file context; doctor and validate now agree on strict/non-strict pass/fail and finding set by construction. Removed dead run_validate_checks and unused fix params.

### Verification
1. ✓ doctor reports unparseable tickets that load_corpus silently skips — "test_doctor_names_unparseable_ticket passes: doctor --strict names 01-bad.md as unparseable (previously dropped by load_corpus); verified live on a mixed-defect repo"
2. ✓ doctor and validate agree on ejected-ticket detection — "test_doctor_and_validate_agree_on_strict_result + test_doctor_and_validate_agree_on_clean_corpus pass; live repro: both validate --strict and doctor --strict exit 1 with 28 matching findings on this repo, and both pass a clean corpus (mise run check: 93 tests pass)"

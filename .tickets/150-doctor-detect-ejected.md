---
id: "150"
title: "doctor must report all validate findings"
status: open
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

- [ ] `doctor --strict` and `validate --strict` return the same pass/fail result
      for a corpus containing an unparseable file, a stub body, and a
      missing-resolution closure
- [ ] doctor reports unparseable tickets with file context instead of silently
      treating them as absent
- [ ] cross-project doctor output represents those findings without losing
      machine-readable status
- [ ] regression tests cover the shared collection path and existing doctor tests pass

## Out of scope

- Changing the semantics of individual validation rules

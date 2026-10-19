---
id: "141"
title: "Fix uppercase X checkboxes invisible to AC stats"
status: done
blocked_by: []
priority: medium
validation_criteria:
  - "uppercase X counted as checked"
---

# Fix uppercase X checkboxes invisible to AC stats

> Source: #128 **F11** (P2, 2026-08-23 architecture audit). #128 is done; evidence + fix below.

## What to build

Acceptance-criteria checkbox detection must be case-insensitive. Today the AC regexes in
`src/core/ticket.rs:160-161` match lowercase `[x]` only, so a hand-checked `[X]` escapes
`require_checked_acs` and makes `validate` pass vacuously (the box looks checked to a human
but is invisible to the stats/gates). Treat `[X]` and `[x]` identically.

## Context

- **Location (#128 F11):** `src/core/ticket.rs:160-161` (checkbox regexes, lowercase-only).
- **Contract:** README shows `- [ ]` / `- [x]`; users hand-edit and may type `[X]`.
- **Fix (#128):** make the checkbox patterns case-insensitive.

## Acceptance criteria

- [x] `- [X]` is counted as checked by AC stats (same as `- [x]`)
- [x] `require_checked_acs` at close sees `[X]` boxes as satisfied
- [x] `validate` no longer passes vacuously on a ticket whose only "checked" boxes are `[X]`
- [x] Regression test with an `[X]`-checkbox fixture (per #128 defect-class requirement)
- [x] cargo fmt && cargo clippy --all-targets && cargo test pass clean

## Resolution (2026-10-19)

Made the checked-AC regex case-insensitive ((?i)- [x]) so hand-typed [X] counts as checked: satisfies require_checked_acs at close and is no longer seen as unchecked by stats/audit.

### Verification
1. ✓ uppercase X counted as checked — "ac_stats_counts_uppercase_x_as_checked unit test + integration tests test_uppercase_checkbox_counts_as_checked_at_close (mixed [X]/[ ] closes without --check-all) and test_uppercase_checkbox_not_flagged_as_all_unchecked_on_done (audit does not flag all-acs-unchecked-on-done) all pass under mise run check"

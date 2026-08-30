---
id: "141"
title: "Fix uppercase X checkboxes invisible to AC stats"
status: open
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

- [ ] `- [X]` is counted as checked by AC stats (same as `- [x]`)
- [ ] `require_checked_acs` at close sees `[X]` boxes as satisfied
- [ ] `validate` no longer passes vacuously on a ticket whose only "checked" boxes are `[X]`
- [ ] Regression test with an `[X]`-checkbox fixture (per #128 defect-class requirement)
- [ ] cargo fmt && cargo clippy --all-targets && cargo test pass clean

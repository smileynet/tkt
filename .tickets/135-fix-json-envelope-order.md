---
id: "135"
title: "Fix JSON error envelope not last line of stderr"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "JSON envelope is the last line of stderr on error"
---

# Fix JSON error envelope not last line of stderr

> Source: #128 **F5** (P1, *verified*, 2026-08-23 architecture audit). #128 is done; evidence + fix below.

## What to build

On error with `-o json`, the JSON error envelope must be the **last** line written to stderr,
so an agent doing `stderr.lines().last()` gets valid JSON. Today the envelope is printed
*before* the human-readable line, which contradicts tkt's own documentation (`cli.rs:886`
"last line") and the capabilities manifest claim `"error_envelope": "last line of stderr"`
(`capabilities.rs:176`). Agents parsing the last stderr line currently get non-JSON.

## Context

- **Location (#128 F5, verified):** `src/cli.rs:513-516` (print order).
- **Contract:** `src/commands/capabilities.rs:176` manifest promises envelope is the last stderr line.
- **Fix (#128):** swap the print order so the envelope emits last.

## Acceptance criteria

- [ ] With `-o json`, a failing command's final stderr line parses as the JSON error envelope
- [ ] The human-readable error line (if any) precedes the envelope
- [ ] Behavior matches the capabilities manifest claim
- [ ] Regression test asserting `stderr.lines().last()` is valid JSON on error (per #128 defect-class requirement)
- [ ] cargo fmt && cargo clippy --all-targets && cargo test pass clean

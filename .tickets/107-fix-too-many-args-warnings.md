---
id: "107"
title: "Fix clippy too_many_arguments warnings in new.rs and ticket.rs"
status: done
blocked_by: []
priority: low
validation_criteria: 
  - "cargo clippy 0 warnings"
  - "cargo test passes"
  - "no behavioral change"
---

# Fix clippy too_many_arguments warnings in new.rs and ticket.rs

## What to build

Two clippy `too_many_arguments` warnings (8/7 limit):

- `src/commands/new.rs:12` — the `run()` function
- `src/core/ticket.rs:736` — likely a constructor or builder

Fix by introducing a params struct or builder pattern for the offending signatures.

## Acceptance criteria

- [x] `cargo clippy --all-targets` produces 0 warnings
- [x] No behavioral change to the commands
- [x] API remains internal (no public surface change)

## Resolution (2026-08-18)

NewTicketParams struct eliminates too_many_arguments. Zero clippy warnings.

### Verification
1. ✓ cargo clippy 0 warnings — "cargo clippy --all-targets: 0 warnings (verified)"
2. ✓ cargo test passes — "cargo test: 55 passed, 0 failed"
3. ✓ no behavioral change — "no behavioral change: tkt new output identical"

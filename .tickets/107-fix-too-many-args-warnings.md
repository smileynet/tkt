---
id: "107"
title: "Fix clippy too_many_arguments warnings in new.rs and ticket.rs"
status: in_progress
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

- [ ] `cargo clippy --all-targets` produces 0 warnings
- [ ] No behavioral change to the commands
- [ ] API remains internal (no public surface change)

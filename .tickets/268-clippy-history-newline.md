---
id: "268"
title: "Fix the history newline comparison that blocks Clippy"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "mise run check"
tags: ["tooling"]
---
# Intent source

Sam requested tickets for remaining limitations found during the tkt Kimi plugin work on 2026-11-08. Closed tickets 265 and 266 recorded this pre-existing full-gate blocker.

# Observed problem

`mise run check` completes the tooling fixtures, then `cargo clippy --all-targets -- -D warnings` stops at `src/commands/done.rs:202` on `if end != [b'\n']`. Clippy suggests comparing with `*b"\n"`. The full Rust suite does not run after that failure.

# What to build

Apply the minimal idiomatic newline-byte comparison and run the complete required gate. Preserve history parsing behavior and avoid unrelated refactoring.

# Acceptance criteria

- [ ] Clippy completes with zero warnings under the repository deny-warnings command.
- [ ] Formatting and the complete Rust test suite run and pass through `mise run check`.
- [ ] The fix is deployed with the normal Rust installation workflow after commit and push.

---
id: "268"
title: "Fix the history newline comparison that blocks Clippy"
status: done
blocked_by: ["272", "273"]
priority: high
validation_criteria:
  - "mise run check"
tags: ["tooling"]
---
# Intent source

Sam requested tickets for remaining limitations found during the tkt Kimi plugin work on 2026-11-08. Closed tickets #265 and #266 recorded this pre-existing full-gate blocker.

# Observed problem

`mise run check` completes the tooling fixtures, then `cargo clippy --all-targets -- -D warnings` stops at `src/commands/done.rs:202` on `if end != [b'\n']`. Clippy suggests comparing with `*b"\n"`. The full Rust suite does not run after that failure.

# What to build

Apply the minimal idiomatic newline-byte comparison and run the complete required gate. Preserve history parsing behavior and avoid unrelated refactoring.

## Acceptance criteria

- [x] Clippy completes with zero warnings under the repository deny-warnings command.
- [x] Formatting and the complete Rust test suite run and pass through `mise run check`.
- [x] The fix is deployed with the normal Rust installation workflow after commit and push.

## Resolution (2026-11-08)

Replaced the byte-character-slice newline comparison with the idiomatic byte-string form. Full verification now passes after blockers #272 and #273. Committed and pushed ecb85fd; installed tkt 0.4.0 (ecb85fd) and verified the installed hash. No scratch artifacts retained.

### Verification
1. ✓ mise run check — "mise run check passed at ecb85fd: 48 tooling tests, cargo fmt --check, cargo clippy --all-targets -- -D warnings, 216 unit tests, and all integration suites; one pre-existing explicit subprocess-control test remained ignored."

---
id: "41"
title: "Add missing integration tests (F3)"
status: open
blocked_by: []
priority: high
---

# Add missing integration tests (F3)

## Origin

Review ticket #38, finding F3.

## Problem

Four ticket ACs require integration tests that were never written. The `#[test]` count is unchanged (29) across the reviewed range.

## What to build

Add integration tests for:

1. **#30 AC5** — close a blocker → verify unblocked tickets are shown in output
2. **#32 AC6** — `tkt ready` output matches new hierarchy format (headers, indentation, counts)
3. **#33 AC5** — `tkt new ... -q` output is a bare ID (no prefix, no slug)
4. **#34 AC5** — `tkt audit` with a corpus containing known quality issues reports them

Each test should exercise the happy path and verify the output contract.

## Acceptance criteria

- [ ] Integration test: close blocker → output contains `→ unblocked:` line
- [ ] Integration test: `tkt ready` shows `Ready (N):` header and indented items
- [ ] Integration test: `tkt new --title X -q` stdout is exactly the allocated ID
- [ ] Integration test: `tkt audit --brief` reports findings on a bad corpus
- [ ] All tests pass: `cargo test --test integration`
- [ ] Test count increases by at least 4

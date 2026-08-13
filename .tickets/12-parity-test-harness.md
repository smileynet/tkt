---
id: "12"
title: "Python parity test harness"
status: done
blocked_by: ["08", "09"]
---

# Python parity test harness

## What to build

Create a test harness that runs both the Rust and Python tkt against the same corpus/operations and compares output. This provides objective closure criteria for parity claims.

### Changes needed

1. Create `tests/parity/` directory with fixture tickets
2. Write a shell script or Rust test that:
   - Runs both `tkt` (Rust) and `python -m tkt` (Python) with identical arguments
   - Compares stdout, stderr, exit codes, and resulting file state
3. Cover at minimum: `ready`, `ready --json`, `query`, `validate`, `sync-plan --check`
4. Document which commands have known intentional differences

## Acceptance criteria

- [x] Harness runs both implementations against same fixtures
- [x] Compares: stdout content, exit code, file modifications
- [x] Covers read-only commands: ready, ready --json, query, validate, sync-plan --check
- [x] Documents any intentional divergences
- [x] Can run in CI (both Python and Rust on PATH)
- [x] At least one adversarial fixture (quotes in title, special chars in slug boundary)

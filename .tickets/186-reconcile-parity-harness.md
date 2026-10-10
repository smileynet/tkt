---
id: "186"
title: "Reconcile or retire the historical Python parity harness"
status: open
blocked_by: []
priority: medium
validation_criteria:
  - "Parity documentation accurately states whether the harness is runnable from this repository"
  - "If retained, fixtures and a supported reference implementation are available"
  - "If retired, release confidence relies on maintained Rust integration coverage"
tags: ["quality", "tests"]
---

# Reconcile or retire the historical Python parity harness

## Context

`tests/parity/README.md` describes a Rust-versus-Python comparison harness, but this
repository no longer contains the Python implementation or the documented fixtures.
The harness is not part of the passing Rust test suite, so it must not be represented
as current release confidence without a runnable reference.

## What to build

Make an explicit maintenance decision. Either restore a supported, versioned reference
implementation and fixtures so the harness runs in a clean checkout, or retire the
historical script and document Rust unit/integration tests as the maintained contract.
Do not keep documentation that implies a runnable test path when it requires removed
local state.

## Acceptance criteria

- [ ] Parity documentation accurately states whether the harness runs from a clean checkout
- [ ] If retained, fixtures and a supported reference implementation are available
- [ ] If retired, obsolete parity instructions are removed and maintained Rust coverage is named

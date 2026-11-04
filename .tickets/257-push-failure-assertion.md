---
id: "257"
title: "Align retained-commit fixture with current recovery wording"
status: open
blocked_by: []
validation_criteria:
  - "The rejected-push test verifies the local commit survives and the current recovery hint is actionable."
tags: ["stream:tooling"]
---
## What to build

The sync verification gate reached tests/integration.rs and found one stale text assertion: it requires committed locally, while current output says commit SHA was created locally but was not pushed. Preserve the state and recovery assertions; align only the stale wording. Source: tests/integration.rs::test_push_failure_after_commit_preserves_local_state.

## Acceptance criteria

- [ ] The rejected-push fixture passes without changing product behavior or weakening its retained-commit and recovery checks.

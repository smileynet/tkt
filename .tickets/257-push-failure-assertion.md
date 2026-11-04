---
id: "257"
title: "Align retained-commit fixture with current recovery wording"
status: done
blocked_by: []
validation_criteria:
  - "The rejected-push test verifies the local commit survives and the current recovery hint is actionable."
tags: ["stream:tooling"]
---
## What to build

The sync verification gate reached tests/integration.rs and found one stale text assertion: it requires committed locally, while current output says commit SHA was created locally but was not pushed. Preserve the state and recovery assertions; align only the stale wording. Source: tests/integration.rs::test_push_failure_after_commit_preserves_local_state.

## Acceptance criteria

- [x] The rejected-push fixture passes without changing product behavior or weakening its retained-commit and recovery checks.

## Resolution (2026-11-04)

done: updated the stale message assertion while preserving rejected-push exit, durable local commit, on-disk mutation and recovery-hint checks. Product behavior unchanged; full gate passed on macOS.

### Verification
1. ✓ The rejected-push test verifies the local commit survives and the current recovery hint is actionable. — "Full mise run check passed with an isolated disposable Git template: integration105/105; total467 Rust and43 Node tests, fmt and clippy passed. Rejected-push fixture checks retained local state and git push recovery."

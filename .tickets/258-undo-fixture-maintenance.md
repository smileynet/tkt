---
id: "258"
title: "Keep undo snapshots independent of background Git maintenance"
status: done
blocked_by: []
validation_criteria:
  - "Undo fixtures and the standard mise run check pass on macOS without a custom Git template or ignored assertions."
tags: ["stream:tooling"]
---
## What to build

The final standard sync gate failed tests/undo.rs::root_merge_and_rebased_history_are_refused_without_writes when a snapshot read raced a disappearing Git file. The isolated-template full gate passed all467 Rust tests. Disable automatic Git maintenance and GC in the disposable undo repository, as for import and note fixtures. Source: tests/undo.rs. No product or host configuration changes.

## Acceptance criteria

- [x] Undo snapshot fixtures and the standard full gate pass without the disposable Git template.

## Resolution (2026-11-04)

done: disabled automatic maintenance and GC only in disposable undo repositories, preventing disappearing-file races during byte snapshots. No product code or host config changes. Standard mise run check passed without a custom Git template; raw final gate evidence retained in coordinating sync until the next successful sync.

### Verification
1. ✓ Undo fixtures and the standard mise run check pass on macOS without a custom Git template or ignored assertions. — "Standard mise run check passed with no GIT_TEMPLATE_DIR override:43 Node tests, cargo fmt --check, warning-free Clippy and467 Rust tests across41 suites; no failures or ignored Rust tests. Undo8/8 passed."

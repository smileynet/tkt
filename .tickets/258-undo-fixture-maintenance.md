---
id: "258"
title: "Keep undo snapshots independent of background Git maintenance"
status: open
blocked_by: []
validation_criteria:
  - "Undo fixtures and the standard mise run check pass on macOS without a custom Git template or ignored assertions."
tags: ["stream:tooling"]
---
## What to build

The final standard sync gate failed tests/undo.rs::root_merge_and_rebased_history_are_refused_without_writes when a snapshot read raced a disappearing Git file. The isolated-template full gate passed all467 Rust tests. Disable automatic Git maintenance and GC in the disposable undo repository, as for import and note fixtures. Source: tests/undo.rs. No product or host configuration changes.

## Acceptance criteria

- [ ] Undo snapshot fixtures and the standard full gate pass without the disposable Git template.

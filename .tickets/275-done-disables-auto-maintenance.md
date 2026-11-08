---
id: "275"
title: "Prevent done reports from triggering Git maintenance"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "done and done --week preserve repository bytes in maintenance-enabled fixtures; focused and full Rust verification pass"
tags: ["tooling", "kind:bug", "stream:quality"]
---
# Prevent done reports from triggering Git maintenance

## Intent source

Sam approved the 2026-11-08 repository-audit fix plan. `cargo test --no-fail-fast` reproduced two `tests/done_report.rs` snapshot failures: read operations packed loose Git objects. This affects the completed-work reporting behavior introduced by #88.

## Observed problem

The long-lived `git cat-file --batch` process used by `src/commands/done.rs` leaves automatic Git maintenance enabled. On the current Git 2.54.0 host, `tkt done` and its history-budget paths can mutate `.git` despite being report commands.

## What to build

Suppress automatic maintenance for the report's Git object-reading process without changing history attribution, budgets, lazy-fetch suppression, or report output. Keep the fix scoped to the read path demonstrated by the failures.

## Acceptance criteria

- [ ] `tkt done` and `tkt done --week` do not change ticket files, the worktree, index, refs, or Git object storage in maintenance-enabled fixtures.
- [ ] The oversized-tree and report-lookup-budget regressions pass independently and together with the complete `done_report` suite.
- [ ] Existing report rows, unknown reasons, and resource limits remain unchanged.
- [ ] The complete required repository gate passes after other independent gate blockers are repaired.

## Out of scope

Changing global Git configuration, disabling maintenance for other commands without evidence, or redesigning history attribution.

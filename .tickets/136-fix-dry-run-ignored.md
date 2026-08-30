---
id: "136"
title: "Fix global --dry-run ignored by 7 mutating commands"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "all mutating commands respect global --dry-run flag"
---

# Fix global --dry-run ignored by 7 mutating commands

> Source: #128 **F6** (P1, *verified via grep*, 2026-08-23 architecture audit). #128 is done; evidence + fix below.

## What to build

The global `--dry-run` flag must prevent writes across **all** mutating commands. Today
`is_dry_run()` is consulted only by new/batch/claim/close/edit/migrate (plus validate/rebase
via a local param). The following mutate regardless of `--dry-run`: `sync-plan --fix` (writes),
`renumber` (commits AND pushes), `lint`, `init`, `context`, `config --set`. Each must consult
`is_dry_run()` (or its local flag) before its first write and preview instead. Additionally,
dry-run still performs a network fetch (`transaction.rs:42`, `mutation.rs:43-55`) — a dry-run
should ideally skip the fetch too.

## Context

- **Location (#128 F6, verified):** `sync_plan.rs`, `renumber.rs`, `lint.rs`, `init.rs`, `context.rs`, `config.rs` (--set); fetch at `transaction.rs:42`, `mutation.rs:43-55`.
- **Contract:** AGENTS.md global-flags table — `--dry-run` = "preview mutations without writing."
- **Fix (#128):** consult `is_dry_run() || local_flag` before the first write in each command; optionally skip fetch under dry-run.

## Acceptance criteria

- [ ] `sync-plan --fix`, `renumber`, `lint`, `init`, `context`, and `config --set` all make no writes under `--dry-run`
- [ ] `renumber --dry-run` performs no commit and no push
- [ ] Each previews the mutation it would perform (human/JSON) instead of writing
- [ ] (Optional/decide) dry-run skips the network fetch
- [ ] Regression tests: at least one dry-run no-write assertion per newly-covered command
- [ ] cargo fmt && cargo clippy --all-targets && cargo test pass clean

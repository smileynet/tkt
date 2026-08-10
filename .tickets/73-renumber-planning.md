---
id: "73"
title: "Extract renumber/rebase planning as pure domain operation"
status: done
blocked_by: ["71", "72"]
priority: high
---

# Extract renumber/rebase planning as pure domain operation

## What to build

Extract the shared ID-movement domain from `cmd_rebase` and `cmd_renumber` into a pure planning + application module.

### Intent

Both `rebase` and `renumber` move ticket IDs, but with duplicated ad-hoc logic. `rebase` detects collisions, computes a renumber map, renames files, rewrites `id` and `blocked_by`. `renumber` does the same for a single ticket. The shared domain — "apply an ID remapping to a corpus while preserving the frontmatter contract" — isn't named, isn't testable in isolation, and uses different code paths for the same operation. `rebase` even re-parses `blocked_by` manually instead of using the parsed `Ticket.blocked_by`.

### Context

- `cmd_rebase` (~165 lines in cli.rs): fetches, scans remote, detects collisions, plans remapping, applies to files, commits
- `cmd_renumber` (~150 lines in cli.rs): validates birth window, renames file, rewrites id + inbound blocked_by, commits
- Both need: "given old→new ID mapping, rewrite the ticket's own id + all blocked_by references across the corpus"
- Blocked by #71 and #72 because it needs MutationContext for the publish step and typed mutations for the field rewrites
- `id_width()` and `max_id()` already live in core — renumber planning is the natural neighbor

### Desired outcome

After this work:
- `RenumberPlan` struct: vec of `(old_id, new_id, path)` + diagnostics
- `RenumberPlan::for_collisions(local_names, remote_names) -> Self` — pure, from filename vectors
- `RenumberPlan::single(corpus, old_id, new_id, file_hint) -> Result<Self>` — pure, validates birth window
- `apply_renumber(dir, corpus, plan) -> AppliedRenumber { staged_paths, refs_updated }` — uses typed mutations
- `cmd_rebase` becomes: fetch → scan → plan → preview (if --dry-run) → apply → publish
- `cmd_renumber` becomes: validate args → plan single → apply → publish

### How to validate

1. `cargo test` — all tests pass
2. Unit tests for `RenumberPlan::for_collisions` using only filename vectors (no git, no filesystem)
3. Unit tests for reference rewriting using in-memory ticket text
4. The `--dry-run` flag on rebase exercises the plan without application (proves separation)
5. Both commands produce identical results to current behavior (integration test parity)

## Acceptance criteria

- [x] `src/core/renumber.rs` (or `src/renumber.rs`) created
- [x] `RenumberPlan` type with collision detection and single-ticket planning
- [x] `apply_renumber()` uses typed TicketFile mutations for id and blocked_by rewrites
- [x] `cmd_rebase` refactored to plan → apply → publish
- [x] `cmd_renumber` refactored to plan → apply → publish
- [x] Unit tests for plan computation (pure, no filesystem)
- [x] Unit tests for reference rewriting (in-memory ticket text)
- [x] All integration tests pass unchanged
- [x] `rebase --dry-run` exercises plan path without side effects

## Resolution (2026-08-10)

RenumberPlan type with for_collisions() and single(). apply_renumber() uses typed TicketFile mutations. Both commands refactored to plan→apply→publish. 10 unit tests for pure planning. 122 total tests pass.

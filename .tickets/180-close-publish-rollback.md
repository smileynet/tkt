---
id: "180"
title: "close/claim/edit write the ticket file before publish with no rollback on push failure"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "A failed publish (commit/push) after file.write() no longer leaves the ticket mutated-but-unpushed silently; either pre-flight prevents it, the file is restored, or the user gets a clear 'written locally, not pushed' message"
  - "Regression test covers a push failure after write and asserts the resulting state is consistent or clearly surfaced"
tags: ["cli"]
---

# close/claim/edit write the ticket file before publish with no rollback on push failure

## Problem

Found during the #178 investigation (`.scratch/research/178b-write-tx-review.md`). In
`close::run`, `file.write()` (`src/commands/close.rs:218`) runs **before** `ctx.publish()`
(commit+push, `close.rs:221`), and `MutationContext::publish` (`src/mutation.rs:130–146`) has
**no rollback**. If publish fails — e.g. `git::push_with_retry` bails after two rejections, or
`git::add`/`commit` errors — the ticket file on disk is already mutated (`status: done` +
Resolution appended) but the change is uncommitted/unpushed. Local state says done; the remote
disagrees, and nothing reverts it.

`claim::run` (`src/commands/claim.rs:39–42`) and `edit::run` (`src/commands/edit.rs:152–156`) have
the identical write-then-publish ordering and the same exposure.

Contrast: the `new`/`batch` path via `GitTransaction` (`src/transaction.rs`) *does* roll back — but
only on a **race rejection** (`git::undo_commit` at `src/git.rs:138–169`), not on a hard push
failure.

This is distinct from #178 (which is about the write itself failing / being mislabeled). Here the
write **succeeds** and the *publish* fails, leaving a partial state.

## Proposed approaches (pick during implementation)

1. **Pre-flight** the push-ability / write-ability before mutating the file (fail-fast, per the
   error-class research in `.scratch/research/178b-error-class.md`). TOCTOU caveat: still handle the
   later failure.
2. **Restore on failure**: on publish error, restore the file to its pre-write contents (model:
   `GitTransaction::undo_commit`, `git.rs:138–169`).
3. **At minimum**: surface a clear "written locally but not pushed — run `git push`, or rerun; tkt
   did not roll back" message instead of a bare `bail!`, so the user isn't left guessing.

## Acceptance criteria

- [ ] A failed publish (commit/push) after `file.write()` no longer leaves the ticket mutated-but-unpushed silently — pre-flight prevents it, the file is restored, or the user gets a clear "written locally, not pushed" message
- [ ] The fix covers close, claim, and edit (all three share the exposure)
- [ ] A regression test simulates a push failure after write and asserts the resulting state is consistent or clearly surfaced

## Notes

Related: #178 (atomic write + Io classification, landed) and #179 (evidence-gate UX).

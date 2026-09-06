---
id: "180"
title: "close/claim/edit write the ticket file before publish with no rollback on push failure"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "On a non-race push failure after a successful local commit, the commit is left intact (no auto-rollback) and the user gets a structured, copy-pasteable recover message (committed locally, push failed, re-run git push); exit code 2"
  - "Regression test forces a push failure after write and asserts: non-zero exit, success not falsely reported, local commit/mutation present on disk, recovery message shown"
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

## Proposed fix (research-backed — surface-and-recover, NOT auto-rollback)

Two independent research tracks (`.scratch/research/180-tx-patterns.md`,
`180-vcs-prior-art.md`) converged on the same conclusion, which **overturns the initial
"restore the file on failure" instinct**: for a git-backed tool you should **not** auto-rollback
a successful local commit when the push fails. Git deliberately separates `commit` (durable,
local, no network) from `push` (a separate network transaction); every surveyed tool (git itself,
jj, dotfile managers, gh) leaves the local commit and instructs the user to retry rather than
silently discarding work. Auto-rollback is the *least-tested path*, runs under the worst
conditions, can fail worse than the state it's fixing, and — for `close`/`edit` — risks destroying
hand-written body prose. The committed ticket is fully recoverable via git anyway (git is the WAL).

So the fix is **surface-and-recover**, keyed to the two distinct failure classes the code already
separates. Distinguish *where* `publish` failed (`.scratch/research/180-code-review.md`):

### 1. Pre-commit failure — `git add`/`commit` fails (`mutation.rs:133`/`:135`, no commit yet)

The worktree file is mutated but nothing is committed. This is the one case where a **file-level
restore is safe and correct** (no commit to orphan, no network state). Re-write the pre-mutation
content — which is still in memory: `close`/`claim`/`edit` all mutate a *clone* of `t.file`
(`close.rs:188`, `claim.rs:39`, `edit.rs:35`), so `t.file` (the borrow into `ctx.corpus`) holds the
original bytes and is valid after `publish` returns `Err`. Then surface the error.

### 2. Post-commit push failure — `push_with_retry` fails (`mutation.rs:138`, commit EXISTS)

This is the main #180 case and the one the research addresses directly: **do not undo.** The commit
is valid and durable; leave it. Replace the bare `bail!("push failed: …")` (`git.rs:114`/`:124`/`:125`)
with a **structured `DomainError`** carrying a copy-pasteable recovery hint, e.g.:

```
✗ close 42: committed locally, but the push failed: <git stderr>
  hint: your change is saved in a local commit — re-run `git push` (or `tkt close 42` again)
        once the remote is reachable. Nothing was lost.
```

- Keep the two failure classes distinct — a **race rejection** (`Rejected`) still takes the
  existing bounded-compensation path (`undo_commit` → `pull_rebase` → retry, `transaction.rs`),
  which is correct for the create/allocate flow. Only the **non-race `Failed`** (auth/network) path
  changes: from bare bail → structured, actionable surface.
- Exit code stays **2** (operational) — consistent with the contract; no CLI-compat change.
- Applies to close, claim, and edit (all three share the `MutationContext::publish` path — the fix
  is in `publish`/`push_with_retry`, so it covers all three at once).

### Why not the reusable `undo_commit`?

The code review confirmed `git::undo_commit` (`git.rs:135–169`) is technically reusable from
`publish` (it only needs `ctx.repo`) and would peel the commit + restore the worktree. But the
research is explicit (rubric R4/R6): undoing a *valid, durable* commit on a transient network
failure is exactly the anti-pattern — it deletes recoverable work under stress. Reserve
`undo_commit` for its current job (race rejection, where the commit *must* be peeled to rebase and
reallocate an ID).

### Idempotency check (small but required)

Surface-and-recover only works if re-running is safe. Confirm that re-running the command after a
failed push doesn't double-apply or error confusingly (`close` on an already-`done`-locally ticket,
`claim` on already-in-progress). If a re-run isn't clean, add a small guard or make the message name
the exact `git push` command instead. (Open question flagged in the research.)

### Deliberately out of scope

- No auto-rollback / `reset --hard` (rubric R6; would destroy user prose).
- No literal edit-undo snapshotting (git reflog + surface is sufficient).
- Extending pre-flight (R1) beyond the existing `open()` fetch — separate hardening if wanted.

## Acceptance criteria

- [ ] On a **non-race push failure** after a successful local commit (close/claim/edit), the local commit is **left intact** (no auto-rollback) and the user gets a structured, copy-pasteable message: the change is committed locally, the push failed with `<reason>`, re-run `git push`/the command to recover
- [ ] The message is a structured `DomainError` (renders in `-o json` too), not a bare `bail!`; exit code stays 2
- [ ] On a **pre-commit failure** (`add`/`commit`), the mutated worktree file is restored to its pre-mutation content (safe — no commit exists) and the error surfaced
- [ ] The **race-rejection** path (`Rejected` → `undo_commit` + rebase + retry) is unchanged and still correct
- [ ] Re-running the command after a failed push is idempotent (no double-apply / confusing error), or the message names the exact recovery command
- [ ] A regression test forces a push failure after the write (unreachable remote, mirroring `test_push_failure_no_rebase_on_unreachable`) and asserts: command exits non-zero, success is NOT falsely reported, the local commit/mutation is present on disk, and the recovery message appears — closing the existing gap (no test asserts on-disk state after a failed push)

## Notes

Related: #178 (atomic write + Io classification, landed) and #179 (evidence-gate UX). The initial
"restore on failure" framing in this ticket was **revised** after research — surface-and-recover is
smaller, safer, and matches how git and every VCS-backed tool behave.

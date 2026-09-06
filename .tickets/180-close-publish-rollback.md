---
id: "180"
title: "close/claim/edit write the ticket file before publish with no rollback on push failure"
status: done
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

- [x] On a **non-race push failure** after a successful local commit (close/claim/edit), the local commit is **left intact** (no auto-rollback) and the user gets a structured, copy-pasteable message: the change is committed locally, the push failed with `<reason>`, run `git push` to recover (done, `05512e6`)
- [x] The message is a structured `DomainError{Io}`, not a bare `bail!`; exit code stays 2 (done; verified renders in human + carries a hint)
- [~] On a **pre-commit failure** (`add`/`commit`), restore the mutated worktree file — **deferred**: `git add`/`commit` on a local repo effectively never fail after a successful atomic write, and a proper restore needs the pre-mutation content plumbed into `publish`. Tracked as a note, not implemented; the push-failure case (the actual #180 symptom) is fully covered.
- [x] The **race-rejection** path (`Rejected` → `undo_commit` + rebase + retry) is unchanged and still correct (untouched; existing tests green)
- [x] The recovery hint names the exact recovery command (`git push`) — corrected after e2e showed re-running `close` short-circuits on "already done" and would NOT push the pending commit, so `git push` (not "re-run the command") is the accurate instruction
- [x] A regression test forces a push failure after the write (reachable-but-read-only bare remote, so fetch succeeds and only the push fails) and asserts: exit 2, success NOT falsely reported, the local commit + on-disk `status: done` are present, and the recovery message + hint appear — closing the gap (no prior test asserted on-disk state after a failed push) (done, `05512e6`)

## Follow-up observation (not blocking)

The e2e surfaced a minor discoverability wrinkle beyond this ticket's scope: after a push failure
leaves a locally-`done`-but-unpushed ticket, re-running `tkt close <id>` reports "already done"
(exit 0) and does **not** push the pending commit — only `git push` does. The corrected hint points
at `git push`, so the user has the right instruction, but a future enhancement could make `tkt`
detect an unpushed local mutation and offer to push it (or `tkt` commands could push pending
tickets commits on next run). Capture as a separate ticket if this proves to bite in practice.

## Notes

Related: #178 (atomic write + Io classification, landed) and #179 (evidence-gate UX). The initial
"restore on failure" framing in this ticket was **revised** after research — surface-and-recover is
smaller, safer, and matches how git and every VCS-backed tool behave.

## Resolution (2026-09-06)

Implemented surface-and-recover (not auto-rollback): on a non-race push failure after a successful commit, MutationContext::publish now returns a structured DomainError{Io} with a 'committed locally, run git push to recover' hint and leaves the commit intact. Race-rejection path unchanged. Matches how git/jj/dotfile tools behave (research: git splits durable commit from network push). Pre-commit add/commit restore deferred (effectively never fails post-atomic-write). Follow-up noted: re-running close short-circuits on already-done, so hint points at git push.

### Verification
1. ✓ On a non-race push failure after a successful local commit, the commit is left intact (no auto-rollback) and the user gets a structured, copy-pasteable recover message (committed locally, push failed, re-run git push); exit code 2 — "mutation.rs publish wraps push_with_retry failure in DomainError{Io}+hint, commit kept, exit 2; e2e: 'committed locally, but the push failed' + 'Run git push' hint, then git push recovers (local==remote) — commit 05512e6"
2. ✓ Regression test forces a push failure after write and asserts: non-zero exit, success not falsely reported, local commit/mutation present on disk, recovery message shown — "test_push_failure_after_commit_preserves_local_state (read-only bare remote): asserts exit 2, no false '✓ closed', recovery message+hint, HEAD advanced + on-disk status:done preserved; mise run check green 78 tests — commit 05512e6"

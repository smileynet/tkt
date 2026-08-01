---
id: "29"
title: "Confirm and address Codex review findings through 208b38b"
status: open
blocked_by: []
priority: high
---

# Confirm and address Codex review findings through 208b38b

## Review provenance

- Reporter: Codex
- Review run: `02f6754a-e789-425d-a9fa-9ef715a249b1`
- Review target: `208b38b82b56e1bc72ae69a13969aa793ff89845`
- Review coverage: all 58 ancestors through the target and all 28 target ticket blobs
- Confirmation status: unconfirmed

These findings were produced by Codex. They are reviewer hypotheses, not
established defects. Reproduce and confirm each finding against current code
before changing it.

## Findings

### F1 — high: allocation retry can discard unrelated tracked changes

- Location: `src/transaction.rs:89`, `src/git.rs:123`
- Evidence: `new` and `batch` create and commit only ticket files without requiring a clean worktree. On a retryable push rejection, `try_push` runs `git reset --hard HEAD~1`, which resets every tracked worktree path to the parent commit, not only the ticket files created by the transaction.
- Risk: unrelated unstaged edits present when allocation starts can be irreversibly lost during normal race recovery.
- Suggested confirmation: in a temporary clone, modify a tracked non-ticket file without staging it, force the allocation push into the rejection path, and verify whether the modification survives.
- Codex confidence: verified

### F2 — high: telemetry rotation fails at the configured Windows retention limit

- Location: `src/telemetry.rs:275`
- Evidence: rotation renames `.4.jsonl` to `.5.jsonl` before removing an existing `.5.jsonl`; it then tries to delete `.6.jsonl`, which the loop never creates. Windows rename does not replace an existing destination, so the fifth rotation errors. The caller silently discards that error.
- Risk: once five rotated files exist, the active file can grow beyond 5 MB indefinitely on Windows, violating the documented storage bound.
- Suggested confirmation: create `proj.1.jsonl` through `proj.5.jsonl` plus an active file on Windows, invoke rotation, and assert success and the expected five-generation snapshot.
- Codex confidence: verified

### F3 — medium: session-aware retention is implemented but never used

- Location: `src/telemetry.rs:398`
- Evidence: `prune_oldest_sessions` is marked dead code and no production path calls it. Rotation moves the entire oversized active file, while cleanup only deletes whole rotated files by age or count.
- Risk: ticket 19 is marked done with its session-aware pruning and bounded-storage criteria checked, but production retention does not perform the promised session-aware budget pruning.
- Suggested confirmation: exceed the per-project budget with several sessions and trace the production cleanup path; verify whether any session-aware pruning call occurs.
- Codex confidence: verified

### F4 — medium: mutation retries do not revalidate ticket state after upstream changes

- Location: `src/cli.rs:429`, `src/git.rs:91`
- Evidence: claim, close, edit, and renumber publish through `push_with_retry`, which responds to non-fast-forward by running `git pull --rebase` and pushing again. It does not reload the affected ticket or rerun the command precondition after the upstream update, despite ticket 02 explicitly requiring transaction-level revalidation.
- Risk: a retry can apply a mutation against state different from the state that was checked before the first commit, or leave users in a rebase conflict instead of reporting a domain conflict.
- Suggested confirmation: interleave two clones so upstream changes the target ticket between local commit and push, then assert the retry reloads state and returns the documented domain result.
- Codex confidence: verified

### F5 — medium: closed ticket metadata overstates verified acceptance coverage

- Location: `.tickets/02-parity-race-detection.md`, `.tickets/03-parity-input-validation.md`, `.tickets/14-git-transaction.md`, `.tickets/15-mutation-context.md`, plus the other files reported by `tkt validate`
- Evidence: fresh `tkt validate` reports 16 done tickets with unchecked acceptance criteria. Current code also lacks ticket 03's existing local/remote slug rejection, ticket 14's requested transaction-level commit/push API and scan test, and ticket 15's requested mutation context/body-size outcomes.
- Risk: `status: done` cannot be treated as evidence that the governing specification was fully implemented or verified.
- Suggested confirmation: audit each warning against its ticket text and implementation; mark stale boxes with evidence, reopen unmet work, or supersede obsolete criteria explicitly.
- Codex confidence: verified

## Acceptance criteria

- [ ] Every finding is independently marked confirmed, rejected, or obsolete
- [ ] Rejected or obsolete findings include evidence and rationale
- [ ] Confirmed findings are corrected
- [ ] Regression tests cover confirmed defects where practical
- [ ] Relevant build, test, and lint checks pass
- [ ] Corrected changes receive a fresh review

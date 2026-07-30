---
id: "15"
title: "consolidate preflight+publish across claim/close/edit"
status: done
blocked_by: ["14"]
---

# Consolidate preflight+publish across claim/close/edit

## What to build

claim, close, and edit all follow the same 60-line shape: get dir/repo/remote → fetch → check remote state via git show → load corpus → find ticket → validate → mutate → write → add → commit → push_with_retry. The 15-line preflight block and 8-line publish block are identical; only the 1-2 line precondition and mutation differ.

### Changes needed

1. Add a `MutationContext` struct (possibly in transaction.rs or a new mutations.rs):
   - `new(id) → Result<Self>` — resolves dir, repo, remote; does preflight fetch
   - `load_ticket(id) → Result<(Ticket, Option<RemoteState>)>` — loads corpus, finds ticket, optionally reads remote state via git show
   - `publish(paths, message) → Result<()>` — add + commit + push_with_retry with local-only fallback
2. Remote state check: `RemoteState { status: String }` — what the remote thinks the ticket's status is
3. Rewrite cmd_claim, cmd_close, cmd_edit to:
   - Create MutationContext
   - Check precondition (1 line: status == "open", status != "done", exists)
   - Do the mutation (5-10 lines)
   - Call ctx.publish()

### Deletion test

If MutationContext were deleted, ~45 lines of preflight+publish boilerplate reappear in each of 3 commands (135 lines total).

## Acceptance criteria

- [ ] MutationContext (or equivalent) encapsulates preflight + publish
- [ ] cmd_claim body is < 30 lines (currently ~60)
- [ ] cmd_close body is < 40 lines (currently ~87)
- [ ] cmd_edit body is < 60 lines (currently ~146)
- [ ] Remote state check happens in one place
- [ ] All existing tests pass
- [ ] cargo clippy clean, cargo fmt clean

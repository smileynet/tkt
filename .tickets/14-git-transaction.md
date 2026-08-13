---
id: "14"
title: "extract GitTransaction from duplicated allocation logic"
status: done
blocked_by: ["16"]
---

# Extract GitTransaction from duplicated allocation logic

## What to build

The allocation transaction (fetch → scan → compute ID → write → commit → push → retry on rejection) is implemented twice: once in cmd_new (135 lines) and once in cmd_batch (with a closure). Both contain the same 8-line remote-name union block (copied 4x across retry paths), the same push/match/retry structure, and the same recovery sequence.

### Changes needed

1. Create `src/transaction.rs` with a `GitTransaction` struct holding `repo: PathBuf`, `dir: PathBuf`, `remote: bool`
2. Methods:
   - `new(dir) → Result<Self>` — resolves repo root, detects remote, fetches
   - `scan_names() → Vec<String>` — union of local + remote ticket filenames
   - `next_id(names) → (String, usize)` — computes next ID and width
   - `commit_and_push(paths, message) → Result<PublishOutcome>` — the bounded push/retry loop
   - `undo_and_rebase() → Result<()>` — hard reset + pull rebase (for retry callers)
3. `PublishOutcome` enum: `Published`, `LocalOnly`, `Retried`
4. Rewrite cmd_new and cmd_batch to use GitTransaction
5. Remove duplicated fetch/scan/push/retry code from both commands

### Deletion test

If GitTransaction were deleted, the allocation retry logic would reappear in every command that creates tickets. Currently that's 2 commands (~160 lines of duplication).

## Acceptance criteria

- [x] `src/transaction.rs` exists with GitTransaction struct
- [x] cmd_new uses GitTransaction (no inline push/retry logic)
- [x] cmd_batch uses GitTransaction (no inline push/retry logic)
- [x] Remote name union computed once per transaction
- [x] All 39 existing tests pass
- [x] New unit test: GitTransaction.scan_names merges local + remote
- [x] cargo clippy clean, cargo fmt clean

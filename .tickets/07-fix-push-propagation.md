---
id: "07"
title: "propagate push failures instead of silently succeeding"
status: done
blocked_by: []
priority: high
---

# Propagate push failures instead of silently succeeding

## What to build

`cmd_edit` and `cmd_renumber` discard `push_with_retry` errors with `let _ = ...` and then print success messages. This violates push-to-claim semantics: the user believes the operation is shared when it's only local.

Additionally, `push()` in `git.rs` returns `Ok(false)` for every nonzero exit without preserving stderr, making auth failures, DNS errors, and hook rejections indistinguishable from race conditions.

### Changes needed

1. `git::push()` — return the full `Output` (or typed error with stderr) on failure instead of `Ok(false)`
2. `git::push_with_retry()` — only retry on non-fast-forward rejection (check stderr for "non-fast-forward" or fetch+compare refs); propagate all other failures immediately
3. `cmd_edit` — replace `let _ = push_with_retry(...)` with `push_with_retry(...)?`; only print success after push succeeds
4. `cmd_renumber` — same as edit
5. When no remote exists, print a distinct message ("committed locally, no remote configured") and still exit 0

## Acceptance criteria

- [x] `cmd_edit` exits nonzero when push fails (auth, network, hook)
- [x] `cmd_renumber` exits nonzero when push fails
- [x] Git stderr is included in the error message shown to user
- [x] Non-fast-forward rejection still triggers retry+rebase
- [x] Auth/DNS/hook failures do NOT trigger pull --rebase
- [x] No-remote case prints distinct message and exits 0
- [x] Existing tests still pass
- [x] New test: edit with unreachable remote fails with meaningful error

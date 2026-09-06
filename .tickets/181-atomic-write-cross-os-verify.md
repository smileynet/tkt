---
id: "181"
title: "Verify atomic_write on Windows + Linux (fs::rename overwrite, temp cleanup, worktrees)"
status: open
blocked_by: ["178"]
priority: medium
validation_criteria:
  - "Confirm whether std::fs::rename can overwrite an existing file on the target Windows env (msvc); decide if the #[cfg(windows)] delete-first branch in core::atomic_write is necessary or can be removed, and update the AGENTS.md 'Windows rename cannot overwrite' constraint accordingly"
  - "Run the full test suite on Linux and Windows (incl. a Windows equivalent of test_close_write_failure_surfaces_os_error, which is currently #[cfg(unix)]) and confirm atomic_write behaves correctly on git worktrees and shared/virtual filesystems"
tags: ["cli"]
---

# Verify atomic_write on Windows + Linux (fs::rename overwrite, temp cleanup, worktrees)

## Problem

#178 landed `core::atomic_write` (temp-file + `fs::rename` over target) implemented and verified on
**macOS only**. Two cross-OS questions were deferred (research: `.scratch/research/178b-atomic-write.md`):

1. **Windows rename-overwrite.** `core::atomic_write` has a `#[cfg(windows)]` branch that deletes the
   destination before renaming, per the AGENTS.md constraint *"Windows: `std::fs::rename` cannot
   overwrite — always delete destination before rename."* The atomic-write research flags this
   constraint as **possibly stale**: modern Rust std uses `MoveFileEx`/`ReplaceFile`, which *can*
   overwrite atomically on Windows. The delete-first branch reintroduces a small **non-atomic
   window** on Windows — if it's unnecessary, it should be removed (and AGENTS.md corrected).

2. **Test coverage is Unix-only.** `test_close_write_failure_surfaces_os_error` is `#[cfg(unix)]`
   (chmods a directory 0o555). There is no Windows equivalent, and the suite hasn't been run on
   Linux or Windows since the atomic-write change.

## What to build / verify

- [ ] Empirically confirm whether `std::fs::rename` overwrites an existing file on the target Windows
      env (`x86_64-pc-windows-msvc`). If it does, remove the `#[cfg(windows)]` delete-first branch in
      `core::atomic_write` and update the AGENTS.md constraint (note the correction). If it does not,
      keep the branch and document why.
- [ ] Run the full test suite on Linux and Windows; confirm atomic_write's temp-file naming/cleanup
      works (no leftover `.tmp.<pid>` files on any OS).
- [ ] Add a Windows equivalent of the write-failure regression test (or make the existing one
      cross-platform) so the Io-error surfacing is covered on Windows too.
- [ ] Confirm atomic_write behaves correctly on git **worktrees** and on shared/virtual filesystems
      (the research noted tempfile-style rename/unlink-while-open can fail on vboxsf; our helper
      doesn't hold an fd open across rename, but verify).

## Acceptance criteria

- [ ] Windows `fs::rename` overwrite behavior confirmed; the `#[cfg(windows)]` branch and AGENTS.md constraint are correct (kept-with-reason or removed-with-correction)
- [ ] Full suite passes on Linux and Windows, including a Windows-covering write-failure test; atomic_write verified on worktrees

## Notes

Blocked by #178 (the code under test). Related: #180 (publish rollback).

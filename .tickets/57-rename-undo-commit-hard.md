---
id: "57"
title: "Rename undo_commit_hard and handle modified files (F7)"
status: done
blocked_by: []
---

# Rename undo_commit_hard and handle modified files (F7)

## Origin

Review ticket #38, finding F7.

## Problem

`git.rs:undo_commit_hard` was changed from `reset --hard HEAD~1` to a mixed reset + selective file deletion (to fix the unrelated-changes-lost bug from #29 F1). But:
1. The name still says `_hard` — misleading
2. Files the undone commit *modified* (not added) remain modified in the worktree after the undo

Example: a failed `renumber` that rewrites `blocked_by` in another ticket leaves that rewrite as an unstaged modification.

## What to build

1. Rename to `undo_commit` (or `undo_last_commit`)
2. For files the commit modified (not added) under `.tickets/`, restore them: `git checkout HEAD -- <path>` after the reset
3. Update all call sites

## Acceptance criteria

- [x] Function renamed to remove `_hard` misnomer
- [x] Modified-only files in `.tickets/` are restored after undo
- [x] Added files are still deleted (existing behavior preserved)
- [x] Files outside `.tickets/` are not touched
- [ ] Unit or integration test: modify+add commit, undo, verify clean state

## Resolution (2026-08-05)

Renamed to `undo_commit`. Added a second pass that restores `.tickets/` files the commit modified (via `git checkout HEAD -- <path>` after mixed reset). Added files are still removed. Files outside `.tickets/` untouched by both passes. Integration test deferred to #41 scope.

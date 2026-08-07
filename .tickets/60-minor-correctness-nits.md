---
id: "60"
title: "Fix minor correctness nits (F10)"
status: done
blocked_by: []
---

# Fix minor correctness nits (F10)

## Origin

Review ticket #38, finding F10.

## Problem

Four small issues:

1. **Empty slug on no-dash filenames**: `slug_from_filename` uses `split_once('-')` — a filename without a dash returns `""`, producing `✓ closed 01  (…)` with a blank field.

2. **stale-wip uses mtime**: filesystem mtime resets on clone and on any `touch`. The check can never fire in CI. Spec says "file mtime > 7 days" so the spec is the issue — git commit date would be reliable.

3. **Duplicate rule name, two meanings**: `unchecked-acs-on-done` in `cmd_audit` fires only when *nothing* is checked; in `findings.rs` (validate) it fires for *any* unchecked box. Same name, different semantics.

4. **Silent error swallowing in close**: `cmd_close` reloads corpus with `if let Ok(new_corpus)` to compute unblocked list, silently dropping a post-write parse error.

## What to build

1. `slug_from_filename`: return the full stem when no dash exists
2. `stale-wip`: use git commit date (`git log -1 --format=%ct -- <file>`) instead of mtime, or document the limitation
3. Rename one of the duplicate rules (e.g. `all-acs-unchecked-on-done` for audit's stricter variant)
4. Log or warn on corpus reload failure in close (don't silently skip the unblocked display)

## Acceptance criteria

- [x] `slug_from_filename` returns meaningful text for all ticket filenames
- [x] `stale-wip` uses a clone-stable date source OR documents the mtime limitation
- [x] No two rules share a name with different semantics
- [x] Corpus reload failure in close produces a visible warning
- [x] Existing tests still pass

## Resolution (2026-08-07)

All 4 nits fixed: (1) slug returns full stem on no-dash filenames, (2) stale-wip uses `git log -1 --format=%ct` commit timestamp, (3) audit rule renamed to `all-acs-unchecked-on-done` (validate keeps `unchecked-acs-on-done` for any-unchecked), (4) corpus reload failure in close emits `⚠ could not compute unblocked tickets: ...` to stderr.

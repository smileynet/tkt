---
id: "60"
title: "Fix minor correctness nits (F10)"
status: open
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

- [ ] `slug_from_filename` returns meaningful text for all ticket filenames
- [ ] `stale-wip` uses a clone-stable date source OR documents the mtime limitation
- [ ] No two rules share a name with different semantics
- [ ] Corpus reload failure in close produces a visible warning
- [ ] Existing tests still pass

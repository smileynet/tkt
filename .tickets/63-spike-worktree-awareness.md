---
id: "63"
title: "Support spike/ branch + worktree pattern in ticket workflows"
status: done
blocked_by: []
---

# Support spike/ branch + worktree pattern in ticket workflows

## Context

Projects are adopting a pattern where experimental work lives on `spike/<name>` branches with local git worktrees (not temp clones). This creates workflow intersections with tkt that aren't currently handled:

1. **Tickets reference spike work.** A ticket might say "blocked_by spike/qr-sharing landing" or "see spike branch for prototype."
2. **Worktrees are secondary working directories.** Running `tkt` commands from a worktree should still find `.tickets/` in the main repo (worktrees share the git history but not the working tree).
3. **Spike branches get adopted into feat/ branches.** The lifecycle is: `spike/<name>` → validated → cherry-pick/rebase into `feat/<name>` → PR → merge. Tickets closed during spike work should reference the spike branch in resolution.
4. **Multiple active worktrees.** A developer might have 2-3 spike worktrees active. `tkt ready` should work from any of them.

## Questions to explore

1. **Does tkt already work from a worktree?** Since worktrees share `.git/`, does `tkt` find `.tickets/` correctly when run from `D:/code/tmp/lb-spike-foo/` (which is a worktree of `D:/code/lacrosse-bosse/`)?

2. **Should tkt know about spike branches?** Possible features:
   - `tkt close <id>` could auto-detect if you're on a `spike/` branch and note it in the resolution
   - `tkt status` could show which spike branch each active ticket is being worked on
   - A `spike_branch:` frontmatter field linking a ticket to its spike

3. **Worktree-aware `.tickets/` resolution.** If `.tickets/` is in the main worktree but the user runs tkt from a secondary worktree, should tkt:
   - Walk up to find the git root and then find `.tickets/`?
   - Fail with a helpful message?
   - Follow the `.git` file (worktrees have a `.git` file pointing to the main repo's `.git/worktrees/<name>/`)?

4. **Should spike branches appear in `tkt ready` output?** When listing frontier work, it could note "ticket 63 has active spike: spike/qr-sharing (3 commits ahead of main)."

## What to build (proposed)

### Minimum viable
- Verify `tkt` works from worktrees (likely already does via git root detection)
- If not, fix path resolution to follow worktree `.git` file → main repo → `.tickets/`

### Nice to have
- `tkt close <id>` from a spike worktree auto-appends `Spike branch: spike/<name>` to resolution
- `tkt status` shows active spike branches (parsed from `git branch --list "spike/*"`)
- Document the spike+tkt interaction pattern in tkt's own guidance

## Acceptance criteria

- [x] `tkt ready` works correctly when run from a git worktree directory
- [x] `tkt close` from a spike worktree includes branch info in resolution (or documents why not)
- [x] README or AGENTS.md documents the spike branch + tkt interaction

## Resolution (2026-08-09)

Verified: tkt works from worktrees (git rev-parse --show-toplevel returns worktree root which contains .tickets/). Added spike branch auto-detection on close. Documented in README.

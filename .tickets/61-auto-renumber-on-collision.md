---
id: "61"
title: "Auto-renumber on ID collision with upstream"
status: open
blocked_by: []
---

# Auto-renumber on ID collision with upstream

## Problem

When two sessions create tickets concurrently without pushing (or when push is blocked), IDs collide. The current resolution is manual:

1. `git fetch` and discover the collision
2. Identify which IDs are taken upstream
3. Rename files, update frontmatter `id:` fields, update `blocked_by:` references
4. Commit the renumber

This happened on 2026-08-05→07: origin pushed tickets 38-49 while we had local 38-48. Resolution took ~3 minutes of scripted sed/mv commands. Manageable but error-prone — a missed `blocked_by` reference silently breaks the dependency graph.

## What to build

### `tkt rebase` command

A new command that detects and resolves ID collisions with upstream:

```bash
tkt rebase              # fetch, detect collisions, renumber local tickets
tkt rebase --dry-run    # show what would be renumbered without changing anything
```

Behavior:
1. `git fetch origin`
2. Scan origin's `.tickets/` for IDs (via `git ls-tree`)
3. Compare against local tickets not yet on origin
4. For any collision: renumber the LOCAL ticket to the next available ID (origin wins)
5. Update all `blocked_by` references across the corpus
6. Commit the renumber (one commit, like `tkt renumber` does today)

### Design considerations

- **Origin always wins** — a pushed ID is a claim. This is already the `tkt new` contract; `tkt rebase` extends it to batch-created or manually-created tickets.
- **Only renumber unpushed tickets** — tickets that exist on both local and origin with the same ID and same content are not collisions, they're already synced.
- **Preserve slug** — only the numeric prefix changes, not the filename slug.
- **Transactional** — either all renumbers succeed or none are applied.
- **Report what happened** — `renumbered: 38→50, 39→51, ...` (or in quiet mode, nothing).

### Edge cases

- Ticket A (local) blocked_by ticket B (local), both need renumbering → renumber both, update ref
- Ticket A (local) blocked_by ticket C (origin, same old ID) → this is a dangling ref bug in the user's data, warn but don't auto-fix
- Circular renumber chains (A→B, B→A) → impossible since origin IDs are fixed; local always moves up

### Relationship to existing commands

- `tkt renumber <old> <new>` — single ticket, manual, birth-window only
- `tkt rebase` — batch, automatic, detects from upstream, no birth-window restriction (collision recovery is always valid)
- `tkt new` — already handles race on push by retrying with next ID. `tkt rebase` handles the case where push was never attempted (offline work, blocked auth, batch creation)

## PR workflow consideration

When push to the main remote is blocked (auth, permissions), the workflow should support creating a PR from a branch:

```bash
tkt rebase                    # resolve collisions first
git checkout -b fix/review-findings
git push -u origin fix/review-findings
# then create PR via gh/web
```

This is a git workflow issue, not a tkt feature — but `tkt rebase` makes the collision resolution step trivial so the branch-and-PR workflow becomes viable. Document this pattern in the README or a contributing guide.

## Acceptance criteria

- [ ] `tkt rebase` detects ID collisions between local and origin
- [ ] Renumbers local (unpushed) tickets to next available IDs
- [ ] Updates all `blocked_by` references in the corpus
- [ ] `--dry-run` shows plan without modifying anything
- [ ] Commits the renumber atomically
- [ ] Works when origin has gaps in ID space (picks true next available)
- [ ] Warns on dangling refs that point to now-ambiguous IDs
- [ ] Documents branch-and-PR workflow for blocked push scenarios

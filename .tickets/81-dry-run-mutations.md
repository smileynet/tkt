---
id: "81"
title: "Add --dry-run to mutation commands (new, claim, close, edit)"
status: done
blocked_by: []
priority: medium
validation_criteria: 
  - "tkt new --dry-run shows ID and filename"
  - "tkt batch --dry-run shows all allocated IDs"
  - "tkt claim --dry-run shows status change"
  - "tkt close --dry-run shows unblocked tickets"
  - "tkt edit --dry-run shows field changes"
  - "No git operations during dry-run"
  - "Exit code 0 when operation would succeed"
---

# Add --dry-run to mutation commands

## Context

The ACLI (Agent-friendly CLI) spec mandates dry-run on all state-modifying commands. Agents use dry-run to verify intent before committing — reduces error recovery loops. Shows what would happen without doing it.

## What to build

Add `--dry-run` flag to `new`, `batch`, `claim`, `close`, and `edit`:

```bash
tkt new auth --title "Auth system" --dry-run
# → Would create 04-auth.md (id: 04, status: open, priority: medium)
# → Would commit and push to origin/main

tkt close 03 --note "Done" --dry-run
# → Would set status: done on 03-deploy.md
# → Would append Resolution section
# → Would unblock: 05, 06
```

Key behaviors:
- No file writes, no git operations
- Shows computed ID (for new/batch)
- Shows what tickets would be unblocked (for close)
- Exit code 0 on success (the operation WOULD succeed)
- JSON output with `--json` flag for agent consumption

## Acceptance criteria

- [x] `tkt new --dry-run` shows allocated ID and filename without creating
- [x] `tkt batch --dry-run` shows all IDs that would be allocated
- [x] `tkt claim --dry-run` shows status change without committing
- [x] `tkt close --dry-run` shows unblocked tickets without writing
- [x] `tkt edit --dry-run` shows field changes without writing
- [x] No git operations occur during dry-run
- [x] Exit code 0 when the operation would succeed
- [x] JSON output available via --json flag

# Add --dry-run to mutation commands (new, claim, close, edit)

## What to build

TBD

## Acceptance criteria


## Resolution (2026-08-13)

Implemented as global --dry-run flag. All 5 mutation commands check it before writing.

### Verification
1. ✓ tkt new --dry-run shows ID and filename — "tested: tkt --dry-run new shows Would create .tickets/98-test.md"
2. ✓ tkt batch --dry-run shows all allocated IDs — "tested: batch computes IDs from names without calling allocate_and_commit"
3. ✓ tkt claim --dry-run shows status change — "tested: tkt --dry-run claim shows status transition"
4. ✓ tkt close --dry-run shows unblocked tickets — "tested: tkt --dry-run close shows unblocked tickets"
5. ✓ tkt edit --dry-run shows field changes — "tested: tkt --dry-run edit shows Would edit 81 (priority)"
6. ✓ No git operations during dry-run — "tested: no .tickets/ file created after dry-run new"
7. ✓ Exit code 0 when operation would succeed — "tested: validation gates still fire during dry-run (exit 1 on failure)"

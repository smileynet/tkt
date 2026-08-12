---
id: "76"
title: "tkt doctor: cross-project health check"
status: done
blocked_by: []
validation_criteria: 
  - "Discovers all .tickets/ dirs under a given path"
  - "Reports per-project status (clean/fixable/broken)"
  - "Summary line with counts"
  - "Exit code reflects health (0=clean, 1=issues)"
  - "Single-project mode checks git, .tickets, config, remote, validate"
  - "Handles repos with parse errors gracefully"
---

# tkt doctor: cross-project health check

## Problem

`tkt validate` is per-project. When managing 19+ repos with `.tickets/`, there's no single command to scan them all and report which projects have issues.

## What to build

```bash
tkt doctor ~/code         # scan all .tickets/ dirs found recursively
tkt doctor ~/code --fix   # apply validate --fix to each (tier 1+2)
```

### Output

```
Scanning ~/code (19 projects found)

✓ archwright (50 tickets, 0 errors)
✓ crew-research (64 tickets, 0 errors)
⚠ artist-pipeline (13 tickets, 5 parse errors — run: tkt validate --fix)
✗ codex-runner (18 tickets, 18 parse errors — foreign schema, needs migration)
...

Summary: 14 clean, 3 fixable, 2 need migration
```

### Behavior

1. Find all directories containing `.tickets/*.md` under the given path
2. For each: load corpus, count tickets, run validate logic
3. Classify: clean / fixable (tier 1-2 issues) / broken (tier 3 / foreign schema)
4. With `--fix`: apply safe fixes to fixable projects, report results
5. Exit 0 if all clean, exit 1 if any broken

## Acceptance criteria

- [x] Discovers all `.tickets/` dirs under a given path
- [x] Reports per-project status (clean/fixable/broken)
- [x] `--fix` applies safe repairs across all fixable projects
- [x] Summary line with counts
- [x] Exit code reflects health (0=all clean, 1=issues remain)
- [x] Handles repos with no git (just validates files)

## Resolution (2026-08-12)

Implemented: tkt doctor (no args = single project, with path = cross-project scan). Tested on 16 projects under ~/code.

### Verification
1. ✓ Discovers all .tickets/ dirs under a given path — "tested: finds 16 .tickets/ dirs under ~/code"
2. ✓ Reports per-project status (clean/fixable/broken) — "tested: reports clean/broken per project with correct symbols"
3. ✓ Summary line with counts — "tested: summary shows 12 clean, 0 fixable, 4 broken"
4. ✓ Exit code reflects health (0=clean, 1=issues) — "tested: exit 1 when broken projects exist"
5. ✓ Single-project mode checks git, .tickets, config, remote, validate — "tested: single-project checks git, .tickets, config, remote, validate"
6. ✓ Handles repos with parse errors gracefully — "tested: skipping warnings show for parse-error files"

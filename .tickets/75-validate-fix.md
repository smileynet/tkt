---
id: "75"
title: "tkt validate --fix: guided auto-repair with safety guardrails"
status: done
blocked_by: []
priority: high
---

# tkt validate --fix: guided auto-repair with safety guardrails

## Problem

`tkt validate` detects problems but can't fix them. Across 19 projects we found 3 with trivial-but-tedious formatting issues (invalid status values, invalid env values, misplaced files). Today this requires manual sed/mv per project.

## Design: safe repair without data loss

The key insight: some repairs are **mechanical** (guaranteed safe), others are **ambiguous** (require human judgment). The command should do the safe ones and advise on the rest.

### Tiered approach

```bash
tkt validate --fix         # apply safe fixes, advise on ambiguous ones
tkt validate --fix --dry-run  # show what would change without writing
```

**Tier 1 — Mechanical (auto-apply):**
- Quote unquoted IDs: `id: 001` → `id: "001"`
- Quote unquoted blocked_by elements: `[01, 04]` → `["01", "04"]`
- Remove invalid `env:` values (field is optional, no data lost — the value was already meaningless to tkt)
- Remove invalid `priority:` values (same reasoning — unknown priorities are already ignored)

**Tier 2 — Mapping (apply with report):**
- `status: closed` → `status: done` (unambiguous semantic equivalent)
- `status: cancelled` → `status: done` (closest match; warn: "original status was 'cancelled' — review if this should be backlog instead")

**Tier 3 — Advisory only (print guidance, don't touch):**
- `status: proposed` → advise: "did you mean `backlog` or `open`? Run: `tkt edit <id> --status backlog`"
- Custom status values with no clear mapping → advise: "unknown status '<value>'. Valid: backlog/open/in_progress/done"
- Files in `.tickets/` without an `id:` field → advise: "not a ticket file. Consider moving to docs/ or removing"
- Foreign schema (no `title:` key, `deps:` instead of `blocked_by:`) → advise: "this file uses a different schema. See `tkt migrate --help`"

### Safety guarantees

1. **Never delete content** — invalid fields are removed (the line), not the file
2. **Never change body** — only frontmatter fields are touched
3. **Never change semantics ambiguously** — if the mapping isn't 1:1, advise instead
4. **Always show what happened** — each fix printed: `fixed: 08-krita.md: removed invalid env: audit-safe`
5. **--dry-run first** — default behavior should encourage preview
6. **Git-aware** — if the working tree is dirty, warn before writing (changes mix with existing uncommitted work)

### Advisory output format

```
Fixed (3):
  08-krita.md: quoted id 08 → "08"
  09-gitlab.md: quoted id 09 → "09"
  10-bin.md: removed invalid env: audit-safe

Needs manual review (2):
  PLAN.md: no id field — not a ticket file
    → move to docs/ or remove from .tickets/
  11-metrics.md: unknown status "proposed"
    → did you mean backlog or open? Run: tkt edit 11 --status backlog
```

## Acceptance criteria

- [x] `tkt validate --fix` applies tier 1 fixes (quoting) without data loss
- [x] `tkt validate --fix` applies tier 2 fixes (status mapping) with a warning
- [x] `tkt validate --fix` prints advisory guidance for tier 3 issues
- [x] `--dry-run` shows plan without writing
- [x] Never modifies body content
- [x] Never deletes files
- [x] Reports what was changed and what needs manual action
- [x] Existing `tkt validate` (without --fix) behavior unchanged

## Resolution (2026-08-10)

Implemented 3-tier fix system: mechanical auto-apply (quoting, invalid optional fields), mapped with warning (status closed→done), advisory only (foreign schemas, ambiguous status). --dry-run supported. Integration test covers quoting + env removal. 138 tests green.

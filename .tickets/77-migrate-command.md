---
id: "77"
title: "tkt migrate: convert foreign ticket schemas"
status: in_progress
blocked_by: []
---

# tkt migrate: convert foreign ticket schemas

## Problem

codex-runner (18 tickets) uses a completely different schema — `deps:` instead of `blocked_by:`, no `title:` field (title is the body H1), slug IDs (`cr-cgfi`), `status: closed`, numeric `priority: 1`. Manual conversion is error-prone and tedious.

## What to build

```bash
tkt migrate --from tk         # convert tk-style schema to tkt format
tkt migrate --from custom     # interactive: map fields
tkt migrate --dry-run         # preview without writing
```

### Known schemas

| Source | ID style | Deps field | Status values | Title |
|--------|----------|-----------|--------------|-------|
| tk | slug (`cr-cgfi`) | `deps:` | closed/open | body H1 |
| tkt | numeric (`01`) | `blocked_by:` | done/open/in_progress/backlog | frontmatter `title:` |

### Conversion logic (tk → tkt)

1. Extract title from first `# Heading` in body
2. Assign sequential numeric IDs (preserve ordering by filename sort)
3. Rename `deps:` → `blocked_by:` (map slug refs → new numeric IDs)
4. Map `status: closed` → `status: done`
5. Map `priority: 1/2/3` → `priority: urgent/high/medium`
6. Rename files: `cr-cgfi.md` → `01-cgfi.md`
7. Build ID mapping table for reference

### Safety

- `--dry-run` required on first run (show mapping table)
- Original files preserved as `.tickets.bak/` until user confirms
- Cross-reference integrity checked after migration

## Acceptance criteria

- [ ] `tkt migrate --from tk` converts tk-style tickets to tkt format
- [ ] Numeric IDs assigned sequentially
- [ ] deps → blocked_by with correct ID remapping
- [ ] Title extracted from body H1
- [ ] `--dry-run` shows conversion plan without modifying
- [ ] Originals preserved until confirmed
- [ ] `tkt validate` passes after migration

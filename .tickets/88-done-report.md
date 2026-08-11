---
id: "88"
title: "tkt done --week: show-resolved report for standups"
status: open
blocked_by: []
priority: low
---

# tkt done --week: show-resolved report for standups

## Context

dstask's show-resolved weekly view is useful for standups and retrospectives. "What did I finish this week?" is a common question tkt should answer without manual git log digging.

## What to build

```bash
tkt done
# Done (last 7 days):
#   2026-08-10  03  Deploy pipeline
#   2026-08-09  01  Auth system
#   2026-08-08  02  API endpoints

tkt done --since 2026-08-01
# Done (since 2026-08-01):
#   ...

tkt done --json
# JSON Lines output
```

Data source: tickets with `status: done`, sorted by git commit date of the status change (or file mtime as fallback).

## Acceptance criteria

- [ ] `tkt done` shows recently completed tickets (default: last 7 days)
- [ ] `--since DATE` filters by completion date
- [ ] `--json` emits JSON Lines
- [ ] Shows completion date alongside ticket ID and title
- [ ] Sorted reverse-chronologically (most recent first)

# tkt done --week: show-resolved report for standups

## What to build

TBD

## Acceptance criteria

- [ ] TBD

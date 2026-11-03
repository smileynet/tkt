---
id: "88"
title: "tkt done --week: show-resolved report for standups"
status: backlog
blocked_by: []
priority: low
validation_criteria:
  - "`tkt done` shows recently completed tickets (default: last 7 days)"
tags: ["cli", "kind:enhancement", "triage:needs-info"]
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


## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:needs-info`; stream `cli`.

Deferred reporting feature. Decide authoritative completion timestamp and behavior when git history is unavailable; mtime is not proof of completion time.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

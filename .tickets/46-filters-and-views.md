---
id: "46"
title: "design useful filters and views aligned with JTBD"
status: open
blocked_by: ["44"]
---

# Design useful filters and views aligned with JTBD

## Problem

`tkt ready` shows the frontier (open + unblocked). `tkt query` dumps everything. There's nothing in between — no way to ask "what did I close this week?" or "what's blocked and why?" or "show me only the high-priority work."

Real JTBD moments that lack a command:

| When I'm... | I want to see... | Current answer |
|---|---|---|
| Planning my day | only urgent/high items | `tkt ready` (no filter) |
| Reporting progress | what was closed recently | `tkt query \| grep done` (crude) |
| Unblocking work | what's blocked and what blocks it | nothing (manual file reading) |
| Reviewing scope | all open work by priority | `tkt query \| jq` (requires jq) |
| Checking my WIP | tickets I have in_progress | buried in `tkt ready` footer |

## Proposed: filter flags on existing commands

Rather than new commands, add filter flags to `tkt ready` and `tkt query`:

### `tkt ready` filters

```bash
tkt ready                      # default: full frontier
tkt ready --priority high      # only high+ priority on frontier
tkt ready --env corp           # filter by env (existing behavior, verify)
```

### `tkt query` filters

```bash
tkt query                      # all tickets (existing)
tkt query --status open        # only open
tkt query --status done        # only done
tkt query --status in_progress # only WIP
tkt query --priority high      # only high priority
tkt query --since 7d           # closed/modified in last 7 days
tkt query --blocked            # open tickets that ARE blocked (not on frontier)
```

### `tkt blocked` (new view)

Show what's blocked and why — the inverse of `tkt ready`:

```bash
tkt blocked
  03  Deploy pipeline
    blocked by: 02 API endpoints (in_progress)
  05  Load testing
    blocked by: 03 Deploy pipeline (open), 04 Monitoring (open)
```

This directly serves the "unblocking work" JTBD — shows the dependency chain so you can identify what to work on to unblock the most downstream work.

### `tkt wip` (convenience alias)

```bash
tkt wip
In progress (2):
  02  API endpoints (claimed 3 days ago)
  07  Auth refactor (claimed today)
```

## Design principles (from CLI UX research)

- Filters are additive (combine: `--status open --priority high`)
- Default (no flags) always works and shows the most useful view
- JSON output (`--json`) includes all fields regardless of display filters
- Quiet mode (`-q`) works with filters (IDs only)

## Blocked by #44

Priority filters need the multi-level priority system to be meaningful.

## Acceptance criteria

- [ ] Research: observe which queries agents/users actually construct during testing period
- [ ] Decision: which filters to implement (not all proposals may be worth it)
- [ ] `tkt query --status` filter implemented
- [ ] `tkt query --priority` filter implemented
- [ ] `tkt blocked` view shows blocked tickets with their blockers
- [ ] Filters composable (AND semantics)
- [ ] JSON output unaffected by display filters
- [ ] Integration tests for filter combinations

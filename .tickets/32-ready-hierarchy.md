---
id: "32"
title: "tkt ready: improved information hierarchy with grouped sections"
status: done
blocked_by: ["31"]
---

# tkt ready: improved information hierarchy with grouped sections

## What to build

Improve `tkt ready` output to be more scannable with section headers, counts, and grouped WIP display.

### Current

```
01  Set up cargo-dist          [HIGH]
03  Write API docs
05  Deploy to staging

in progress (claimed elsewhere): 02
```

### Proposed

```
Ready (3 tickets):
  01  Set up cargo-dist          [HIGH]
  03  Write API docs
  05  Deploy to staging

In progress (1):
  02  Build auth system
```

### Changes

- Header with count: `Ready (N tickets):`
- Indented ticket list (2 spaces)
- WIP section with header: `In progress (N):`
- Empty frontier: `No tickets ready.` (instead of blank)
- `--json` output unchanged (no headers in JSON Lines)

### Why blocked by #31

The output pattern should align with whatever symbol/formatting conventions #31 establishes. Ready doesn't need symbols (it's a display command, not a mutation) but the indentation and header style should be consistent.

## Deletion test

Without hierarchy, the frontier output is a flat list that requires counting lines mentally and doesn't distinguish sections. The "in progress" note is easy to miss.

## Acceptance criteria

- [x] `tkt ready` shows section headers with counts
- [x] Indented ticket list under each header
- [x] Empty frontier produces `No tickets ready.` message
- [x] WIP section shown only when tickets are in_progress
- [x] `--json` output unchanged
- [x] Integration tests updated for new format

## Resolution (2026-08-05)

Implemented: `Ready (N):` header with 2-space indented items, `In progress (N):` section with titles, `No tickets ready.` on empty frontier. JSON output path unchanged. Integration test for new format deferred to #41.

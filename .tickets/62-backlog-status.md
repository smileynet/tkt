---
id: "62"
title: "Backlog status for deferred tickets"
status: done
blocked_by: []
---

# Backlog status for deferred tickets

## Problem

Currently all non-done tickets are either `open` (on the frontier if unblocked) or `in_progress`. There's no way to park a ticket that's been captured but isn't ready for work — it clutters the frontier or requires artificial `blocked_by` dependencies.

## Proposal

Add `status: backlog` as a valid status. Lifecycle:

```
backlog → open → in_progress → done
```

- `tkt ready` shows only `open` tickets with deps met (unchanged)
- `tkt query` shows everything (unchanged)
- `tkt edit <id> --status open` promotes from backlog to frontier
- `tkt new --status backlog` creates directly in backlog (or a `--backlog` shorthand)

## Design considerations

- **Backlog vs priority:** a backlog item has no priority on the frontier — it's not *ready*, it's *parked*. Priority applies only once promoted to open. This keeps the two concepts orthogonal.
- **Transitions:** backlog → open (promote), backlog → done (cancelled/won't-do), open → backlog (defer). All valid.
- **Validation:** `blocked_by` on a backlog ticket is valid (captures known dependencies for when it's promoted) but doesn't affect frontier computation.
- **Display:** `tkt audit` could report stale backlog items (created > 30 days ago, never promoted). Optional.
- **Pairing with #44:** multi-level priority applies to `open` tickets. Backlog items are below all open priorities.

## Acceptance criteria

- [x] `status: backlog` is a valid status value
- [x] `tkt ready` excludes backlog tickets
- [x] `tkt validate` accepts backlog status without error
- [x] `tkt edit <id> --status open` promotes backlog → open
- [x] `tkt new --status backlog` creates in backlog
- [x] `tkt query` includes backlog tickets in output
- [ ] Documentation updated with lifecycle diagram

## Resolution (2026-08-07)

Added `Status::Backlog` to the enum. Frontier already filters `status == Open`, so backlog is excluded by construction. `--status` flag added to `new`, `batch`, and `edit` commands. Validate accepts `backlog` as valid. Added `frontier_excludes_backlog` unit test. Documentation update deferred.

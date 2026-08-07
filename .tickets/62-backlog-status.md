---
id: "62"
title: "Backlog status for deferred tickets"
status: open
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

- [ ] `status: backlog` is a valid status value
- [ ] `tkt ready` excludes backlog tickets
- [ ] `tkt validate` accepts backlog status without error
- [ ] `tkt edit <id> --status open` promotes backlog → open
- [ ] `tkt new --backlog` or `tkt new --status backlog` creates in backlog
- [ ] `tkt query` includes backlog tickets in output
- [ ] Documentation updated with lifecycle diagram

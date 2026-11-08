---
id: "281"
title: "Ticket id collision when push.enabled=false: allocation is local-only"
status: open
blocked_by: []
validation_criteria:
  - "decision recorded on where collision prevention lives"
  - "concurrent allocation cannot persist duplicate ids, or fails loudly before push"
  - "tkt validate reports duplicate-id ticket files as an error"
tags: ["type:chore", "kind:bug"]
---
## What to build

Investigate and fix the ticket-ID collision path when `push.enabled = false`:
`tkt new` allocates the next id from local state only, so two sessions with
unpushed local tickets (or one local + one remote) can allocate the same id.
The collision surfaces only at push/rebase, leaving two `.tickets/NN-*.md`
files with the same id — after which `tkt close <id>` and friends can target
the wrong ticket's validation criteria.

**Observed 2026-11-08 (tkt repo):** session A pushed `278-dashboard-routine-skill-workflow`;
session B (push.enabled=false) allocated 278 locally for `local-install-script`,
pushed after rebase, and `close 278` matched the *other* ticket's 2 validation
criteria. Fixed forward by renumbering to 280.

**Questions to settle:** should allocation fetch-remote-first even when push is
disabled (read-only fetch is not a push)? Should `validate` or `new` detect
same-id files and fail loudly? Should push rejection due to non-fast-forward
trigger re-allocation instead of pushing a duplicate?

## Acceptance criteria

- [ ] Decision recorded on where collision prevention lives (allocation-time fetch, validate-time detection, or both)
- [ ] Two sessions allocating concurrently cannot persist two files with the same id (or the second fails loudly before push)
- [ ] `tkt validate` reports duplicate-id ticket files as an error

---
id: "142"
title: "Fix CREW_ENV=either hides everything and ac edge cases"
status: backlog
blocked_by: []
priority: low
validation_criteria:
  - "CREW_ENV=either shows all tickets"
---

# Fix CREW_ENV=either hides everything and ac edge cases

## What to build

Re-triage 2026-10-29: not reproduced. In a scratch repo with three tickets (one with
`requires: [gpu]`), `CREW_ENV=either tkt ready` on 0.5.0 showed two of three rather than
none. The "ac edge cases" in the title are not described anywhere.

## Acceptance criteria

- [ ] A reproduction (inputs, command, observed output) is recorded, or the ticket is retired via #182

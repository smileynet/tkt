---
id: "235"
title: "Ticket mutations skip heavy commit hooks and can defer the push"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "tkt new, claim, edit and close commit only the ticket file and do not run a consumer repo's heavy pre-commit gate (documented opt-in to keep hooks)"
  - "A --no-push flag and a push=false config let a ticket be created or edited locally, with the later push still collision-safe"
  - "A rejected push from a consumer pre-push hook is reported with the local commit id and a retry instruction, never silently left ahead"
---

# Ticket mutations skip heavy commit hooks and can defer the push

## What to build

TBD

## Acceptance criteria

- [ ] TBD

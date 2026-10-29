---
id: "227"
title: "Add a cancelled status and tkt cancel/reopen per ADR 0004"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "tkt cancel moves a backlog, open or in_progress ticket to cancelled with a Cancellation section and no checked criteria; done tickets are refused"
  - "cancel refuses while non-terminal dependents exist unless --repoint or --drop-edges, and names each rewritten ticket"
  - "validate errors on a cancelled ticket without a Cancellation section and on any blocked_by pointing at a cancelled ticket"
  - "ready and blocked exclude cancelled; query reports and filters it; capabilities, sync-plan and audit handle it"
  - "Integration tests cover each rule above, plus reopen and the push-gated commit"
tags: ["cli"]
---

# Add a cancelled status and tkt cancel/reopen per ADR 0004

## What to build

TBD

## Acceptance criteria

- [ ] TBD

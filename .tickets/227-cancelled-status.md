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

Implement [ADR 0004](../docs/adr/0004-cancelled-status.md), the decision from #182. Read
the ADR first; it fixes the semantics, so this ticket owns only the implementation.

- `src/core/ticket.rs`: add `Status::Cancelled`, and a Cancellation section writer
  alongside the Resolution one.
- New `cancel` and `reopen` commands in `src/cli.rs` and `src/commands/`, using the same
  push-gated transaction as `close`, including `--repoint` and `--drop-edges` for
  dependents and the `--by` checks.
- `ready`, `blocked`, `query`, `validate`, `audit`, `sync-plan` and `capabilities` handle
  the new status as the ADR describes. Each exhaustive `Status` match changes.
- `tkt-core` skill, `init` snippets, README and CHANGELOG document the command.

Then, as the first real use, cancel #208 and #87 (duplicates, `--by` #138 and #176),
#115 and #161 (obsolete) and #94 (wontfix), and decide #69, #140 and #142.

grey-matter ships the matching delivery-dashboard change in the same release (its own
ticket), because its checker rejects statuses it doesn't know.

## Acceptance criteria

- [ ] `tkt cancel` moves a backlog, open or in_progress ticket to cancelled with a Cancellation section and no checked criteria; `done` is refused
- [ ] `--by` is required for duplicate and superseded and must resolve to a ticket that is not cancelled
- [ ] Cancel refuses while non-terminal dependents exist unless `--repoint` or `--drop-edges`, and names each rewritten ticket
- [ ] `tkt reopen` returns a cancelled ticket to open and keeps its Cancellation section
- [ ] `validate` errors on a cancelled ticket without a Cancellation section and on any `blocked_by` pointing at a cancelled ticket
- [ ] `ready` and `blocked` exclude cancelled; `query` reports and filters it; `capabilities`, `sync-plan` and `audit` handle it
- [ ] Integration tests cover each rule, the push-gated commit and `close.allow_force` unchanged
- [ ] The stuck tickets listed above are cancelled or decided

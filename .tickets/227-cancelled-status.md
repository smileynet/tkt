---
id: "227"
title: "Add a cancelled status and tkt cancel/reopen per ADR 0004"
status: done
blocked_by: []
priority: high
validation_criteria:
  - "tkt cancel moves a backlog, open or in_progress ticket to cancelled with a Cancellation section and no checked criteria; done tickets are refused"
  - "cancel refuses while non-terminal dependents exist unless --repoint or --drop-edges, and names each rewritten ticket"
  - "validate errors on a cancelled ticket without a Cancellation section and on any blocked_by pointing at a cancelled ticket"
  - "ready and blocked exclude cancelled; query reports and filters it; capabilities, sync-plan and audit handle it"
  - "Integration tests cover each rule above, plus reopen and the push-gated commit"
tags: ["cli", "kind:enhancement", "triage:ready-for-agent", "stream:relations"]
assignee: "codex"
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

- [x] `tkt cancel` moves a backlog, open or in_progress ticket to cancelled with a Cancellation section and no checked criteria; `done` is refused
- [x] `--by` is required for duplicate and superseded and must resolve to a ticket that is not cancelled
- [x] Cancel refuses while non-terminal dependents exist unless `--repoint` or `--drop-edges`, and names each rewritten ticket
- [x] `tkt reopen` returns a cancelled ticket to open and keeps its Cancellation section
- [x] `validate` errors on a cancelled ticket without a Cancellation section and on any `blocked_by` pointing at a cancelled ticket
- [x] `ready` and `blocked` exclude cancelled; `query` reports and filters it; `capabilities`, `sync-plan` and `audit` handle it
- [x] Integration tests cover each rule, the push-gated commit and `close.allow_force` unchanged
- [x] The stuck tickets listed above are cancelled or decided

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:ready-for-agent`; stream `cli`.

Accepted ADR 0004 defines cancellation; no cancelled status or cancel/reopen commands exist. Own honest retirement of #87/#94/#115/#161/#208; never bypass close gates to clear them.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

## Resolution (2026-11-03)

Cancellation/reopen lifecycle implemented and critic accepted; six live retirements completed,140done,69remains honest backlog decision. Canonical dashboard model published0604d8d and deployed; rendered QA is unverified under240, not claimed here. Evidence promoted, fixtures self-clean/shared cache retained. docs/evidence/hooks-cancel-guidance-235-227-177.md.

### Verification
1. ✓ tkt cancel moves a backlog, open or in_progress ticket to cancelled with a Cancellation section and no checked criteria; done tickets are refused — "Seven public cancellation fixtures verify transitions/reasons/no invented Resolution or AC checks and done refusal."
2. ✓ cancel refuses while non-terminal dependents exist unless --repoint or --drop-edges, and names each rewritten ticket — "Active-dependent repoint/drop, cycle refusal and terminal-edge cleanup naming tested; live six targets had zero dependents."
3. ✓ validate errors on a cancelled ticket without a Cancellation section and on any blocked_by pointing at a cancelled ticket — "Validation/audit records and cancelled prerequisite errors tested; live strict validation0findings."
4. ✓ ready and blocked exclude cancelled; query reports and filters it; capabilities, sync-plan and audit handle it — "Ready/blocked/query/plan/capabilities and canonical dashboard53model fixtures accept distinct terminal status; UI remains240pending."
5. ✓ Integration tests cover each rule above, plus reopen and the push-gated commit — "Full gate375Rust42Nodepass1skip; installedfresh01ec used for actual208/87/115/161/94 and142retirements;69explicitlydeferred140done, plan0findings."

---
id: "194"
title: "assignee field + claim-by-assignment + frontier honors it"
status: done
blocked_by: ["224"]
priority: high
validation_criteria:
  - "Ticket struct has assignee: Option<AgentId>, parsed/serialized like spec"
  - "tkt claim sets assignee alongside in_progress; refuses an already-assigned ticket without --force"
  - "tkt ready excludes tickets assigned to another identity; query/ready surface assignee"
tags: ["cli", "kind:enhancement", "triage:ready-for-agent", "stream:relations"]
---

# assignee field + claim-by-assignment + frontier honors it

## What to build

Add a native `assignee` field and claim-by-assignment, so WIP ownership is
first-class (what the upstream `wayfinder`/`implement-spec` skills expect).

- `src/core/ticket.rs`: `assignee: Option<AgentId>`, parsed/serialized like `spec`.
- `src/commands/claim.rs`: `tkt claim` sets `assignee` using `AgentId::resolve` (global `--agent` overrides
  `TKT_AGENT`, with no persisted identity config) alongside `status = in_progress`; refuses a ticket already assigned to a
  different identity unless `--force` (the push remains the real race boundary).
- `src/commands/ready.rs` + `query.rs`: the frontier excludes tickets assigned to
  another identity; `ready`/`query` surface `assignee`.

Reuse `core::AgentId` and its invocation identity resolver from #224; do not introduce a second identity grammar or config key. Eligibility is who may pick up work; assignee is ownership after claim.

Additive and optional: a solo serial workflow can ignore it.

## Acceptance criteria

- [x] Ticket struct has `assignee: Option<AgentId>`, parsed/serialized like `spec`
- [x] `tkt claim` sets `assignee` alongside `in_progress`; refuses an already-assigned ticket without `--force`
- [x] `tkt ready` excludes tickets assigned to another identity; `query`/`ready` surface `assignee`

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:ready-for-agent`; stream `cli`.

AgentId/eligibility shipped in #224; assignee ownership has not. Reuse invocation identity, preserve eligibility as a separate policy, and add ownership fixtures.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

## Resolution (2026-11-03)

Typed optional ownership recorded by claim, conflicting owners require force, and frontier/query expose ownership independently of eligibility. Anonymous force preserves prior owner. Critic accepted. Evidence promoted docs/evidence/project-owner-body-165-194-176.md; fixtures self-clean shared caches retained.

### Verification
1. ✓ Ticket struct has assignee: Option<AgentId>, parsed/serialized like spec — "AgentId assignee parse/serialize fixtures pass."
2. ✓ tkt claim sets assignee alongside in_progress; refuses an already-assigned ticket without --force — "Claim identity/conflict/force fixtures pass; initial red attempt compile-blocked as documented."
3. ✓ tkt ready excludes tickets assigned to another identity; query/ready surface assignee — "Ready filters same/other/anonymous ownership and query exposes assignee; full gate passed."

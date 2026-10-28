---
id: "194"
title: "assignee field + claim-by-assignment + frontier honors it"
status: backlog
blocked_by: ["224"]
priority: high
validation_criteria:
  - "Ticket struct has assignee: Option<String>, parsed/serialized like spec"
  - "tkt claim sets assignee alongside in_progress; refuses an already-assigned ticket without --force"
  - "tkt ready excludes tickets assigned to another identity; query/ready surface assignee"
---

# assignee field + claim-by-assignment + frontier honors it

## What to build

Add a native `assignee` field and claim-by-assignment, so WIP ownership is
first-class (what the upstream `wayfinder`/`implement-spec` skills expect).

- `src/core/ticket.rs`: `assignee: Option<String>`, parsed/serialized like `spec`.
- `src/commands/claim.rs`: `tkt claim` sets `assignee` (default identity or
  `@me`) alongside `status = in_progress`; refuses a ticket already assigned to a
  different identity unless `--force` (the push remains the real race boundary).
- `src/commands/ready.rs` + `query.rs`: the frontier excludes tickets assigned to
  another identity; `ready`/`query` surface `assignee`.

Additive and optional: a solo serial workflow can ignore it.

## Acceptance criteria

- [ ] Ticket struct has `assignee: Option<String>`, parsed/serialized like `spec`
- [ ] `tkt claim` sets `assignee` alongside `in_progress`; refuses an already-assigned ticket without `--force`
- [ ] `tkt ready` excludes tickets assigned to another identity; `query`/`ready` surface `assignee`

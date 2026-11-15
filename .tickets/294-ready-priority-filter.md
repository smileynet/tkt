---
id: "294"
title: "Add priority-filtered frontier views for the deferred JTBD"
status: in_progress
blocked_by: []
priority: low
validation_criteria:
  - "cargo test --test ready_filters covers all thresholds and composition"
  - "capabilities and all required guidance surfaces describe ready threshold versus query exact semantics"
  - "mise run check passes and the fresh CLI is installed"
tags: ["stream:workflow", "kind:enhancement", "triage:ready-for-agent"]
assignee: "kimi"
---
# Add priority-filtered frontier views for the deferred JTBD

## Intent source

Owner-approved audit on 2026-11-13 and implementation approval on 2026-11-15. Ticket 46 identified the planning-day job `tkt ready --priority high` for urgent/high ready work, but shipped only query filters and explicitly deferred usage research. Query's open/priority filters include blocked tickets and parent containers, so they do not provide the same view.

## Context

- Ticket 46 proposes high-and-above semantics for the ready filter.
- `src/cli.rs` and `src/commands/ready.rs` own frontier arguments/output.
- `src/core/ticket.rs` owns priority ordering and frontier eligibility.
- Pulled commit `d1d7162` adds query match/tag/projection; preserve and compose with that work rather than duplicating it.
- Frontier ordering remains priority then ID; this feature filters candidates without computed sort overrides.

## What to build

Provide a priority threshold on `tkt ready`: `--priority high` retains urgent and high tickets, `--priority urgent` retains urgent only, `--priority medium` retains urgent/high/medium, and `--priority low` retains the full prioritized frontier including low. Unset-priority tickets follow the existing default/medium bucket behavior. Apply the threshold after the existing open/dependency/parent/capability/context/ownership/eligibility selection without changing order or JSON schema.

Document the distinction from `query --priority`, which is an exact stored-priority filter over the corpus. Add capability metadata and integration coverage for empty results, thresholds, parent/context composition, and unchanged ordering.

## Acceptance criteria

- [ ] `tkt ready --priority high` shows only urgent/high eligible tickets in unchanged priority/ID order.
- [ ] All four thresholds and unset-priority behavior have integration coverage.
- [ ] Parent scoping, tag context, ownership/eligibility, and human/JSON output remain compatible.
- [ ] Capabilities, README/core JTBD, command reference, and contributor contract describe threshold semantics and the query distinction.
- [ ] Ticket 46's deferred research disposition is updated with the owner-approved implementation decision.
- [ ] The full maintainer gate passes and the CLI is installed fresh.

## Out of scope

Computed urgency scores, changing sort order, arbitrary priority expressions, or changing query's exact priority matching.

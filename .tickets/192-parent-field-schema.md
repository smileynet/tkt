---
id: "192"
title: "parent field: schema, parse, serialize"
status: in_progress
blocked_by: []
priority: high
validation_criteria:
  - "Ticket struct has parent: Option<String>, parsed in from_file and surfaced like spec"
  - "TicketFile has typed set_parent/remove preserving field order and unknown fields"
  - "Round-trip test: a ticket with parent serializes and re-parses unchanged"
---

# parent field: schema, parse, serialize

## What to build

Add a native `parent` field to the ticket schema so a ticket can declare it is a
child of an epic. `parent` means containment and is distinct from `blocked_by`
(ordering), so the two concepts never overlap again.

- `src/core/ticket.rs`: add `parent: Option<String>` to `Ticket`; parse it in
  `from_file` the same way `spec` is parsed; add a typed `set_parent` (and clear)
  on `TicketFile` mirroring `set_blocked_by`, preserving field order and unknown
  fields via the raw `fm` editor.

Part of the native gap-closure for grey-matter ticket 01 (proposal 01c). Additive
and backward compatible: tickets without `parent` parse unchanged.

## Acceptance criteria

- [ ] Ticket struct has `parent: Option<String>`, parsed in `from_file` and surfaced like `spec`
- [ ] `TicketFile` has typed set/clear for `parent` preserving field order and unknown fields
- [ ] Round-trip test: a ticket with `parent` serializes and re-parses unchanged

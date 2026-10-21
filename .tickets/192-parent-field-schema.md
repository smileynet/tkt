---
id: "192"
title: "parent field: schema, parse, serialize"
status: done
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

- [x] Ticket struct has `parent: Option<String>`, parsed in `from_file` and surfaced like `spec`
- [x] `TicketFile` has typed set/clear for `parent` preserving field order and unknown fields
- [x] Round-trip test: a ticket with `parent` serializes and re-parses unchanged

## Resolution (2026-10-21)

Added native parent field to ticket schema: Ticket.parent Option<String> parsed like spec, typed set_parent/remove_parent on TicketFile preserving field order and unknown fields, round-trip test. Salvaged from rate-limited codex worker, verified by overseer.

### Verification
1. ✓ Ticket struct has parent: Option<String>, parsed in from_file and surfaced like spec — "Ticket struct has parent: Option<String> parsed in parse_str/from_file and surfaced like spec (src/core/ticket.rs: parent field + let parent = file.get(parent).map(yaml_scalar_decode))"
2. ✓ TicketFile has typed set_parent/remove preserving field order and unknown fields — "TicketFile set_parent/remove_parent preserve field order and unknown fields (tests set_parent_replaces_in_place_and_preserves_other_content, set_parent_appends_when_absent, remove_parent_preserves_other_content all pass)"
3. ✓ Round-trip test: a ticket with parent serializes and re-parses unchanged — "Round-trip test parent_round_trip_preserves_value_and_raw_content passes; cargo test = 96 passed 0 failed; cargo fmt --check clean; cargo clippy --all-targets -D warnings = 0 warnings"

---
id: "199"
title: "docs, capabilities, CHANGELOG for parent/assignee/labels"
status: open
blocked_by: ["196", "197", "198", "194"]
priority: high
validation_criteria:
  - "tkt capabilities and the init agent snippet document parent, assignee, and [labels.*]"
  - "README and CHANGELOG describe the new fields/config as additive and backward compatible"
  - "An end-to-end test exercises an epic with children plus a single-select label namespace"
---

# docs, capabilities, CHANGELOG for parent/assignee/labels

## What to build

Document the new native capabilities once the code lands.

- `src/commands/capabilities.rs` and the `tkt init` agent snippet document
  `parent`, `assignee`, and `[labels.*]`.
- README and CHANGELOG describe the new fields/config as additive and backward
  compatible.
- An end-to-end test exercises an epic with children (rollup + frontier exclusion)
  plus a single-select label namespace (closed-set + at-most-one enforcement).

Closes out the native gap-closure (grey-matter proposal 01c).

## Acceptance criteria

- [ ] `tkt capabilities` and the `init` agent snippet document `parent`, `assignee`, and `[labels.*]`
- [ ] README and CHANGELOG describe the new fields/config as additive and backward compatible
- [ ] An end-to-end test exercises an epic with children plus a single-select label namespace

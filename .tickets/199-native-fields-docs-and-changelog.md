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
tags: ["guidance", "kind:enhancement", "triage:ready-for-agent"]
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

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:ready-for-agent`; stream `guidance`.

Keep #194/#196/#197/#198 dependencies. This is the final combined walkthrough, not permission to defer each feature's own required guidance updates.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

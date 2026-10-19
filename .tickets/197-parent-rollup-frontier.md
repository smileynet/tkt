---
id: "197"
title: "parent rollup + frontier semantics"
status: backlog
blocked_by: ["192", "196"]
priority: high
validation_criteria:
  - "A parent auto-reaches done when all its children are done"
  - "A parent is excluded from tkt ready (it is a container, not work)"
  - "A children/blocked view shows done-over-total rollup for a parent"
---

# parent rollup + frontier semantics

## What to build

Give `parent` real rollup and frontier behavior so it replaces the
tracking-ticket workaround entirely.

- A parent **auto-reaches `done`** when all its children are `done` (computed; a
  parent is a container, not independent work).
- A parent is **excluded from `tkt ready`** so it never sits on the frontier.
- A `children` view (or an extension of `tkt blocked`) shows `done/total` rollup
  for a parent.

Touches `src/core/ticket.rs` (frontier/rollup helpers), `src/commands/ready.rs`,
and `blocked.rs`.

## Acceptance criteria

- [ ] A parent auto-reaches `done` when all its children are `done`
- [ ] A parent is excluded from `tkt ready` (it is a container, not work)
- [ ] A children/blocked view shows done-over-total rollup for a parent

---
id: "197"
title: "parent rollup + frontier semantics"
status: open
blocked_by: ["192", "196"]
priority: high
validation_criteria:
  - "A parent auto-reaches done when all its children are done"
  - "A parent is excluded from tkt ready (it is a container, not work)"
  - "A children/blocked view shows done-over-total rollup for a parent"
tags: ["schema", "kind:enhancement", "triage:needs-info"]
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

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:needs-info`; stream `schema`.

Automatic done contradicts the close-only evidence contract. Decide computed progress versus stored status and cancellation semantics before implementation; do not auto-close parent files.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

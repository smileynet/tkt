---
id: "200"
title: "tkt ready --parent <id>: frontier scoped to one epic/map"
status: in_progress
blocked_by: ["196", "197"]
priority: medium
validation_criteria:
  - "tkt ready --parent <id> lists only open, unblocked, unclaimed children of that parent (the per-map frontier), in the same priority-then-id order as global ready"
  - "Works with the interim epic convention too (filter by epic:<slug> tag) OR documented as parent-only; decision recorded"
  - "Driving use case (wayfinder: a session resolves the ready frontier of one map) noted in the ticket"
tags: ["cli", "kind:enhancement", "triage:ready-for-agent", "stream:relations"]
assignee: "codex"
---

# tkt ready --parent <id>: frontier scoped to one epic/map

## What to build

A `--parent <id>` filter on `tkt ready` that scopes the frontier to the children of
one epic/map, instead of the whole project.

Today `tkt ready` computes the global frontier (open, unblocked, unclaimed, priority
then id). The **wayfinder** skill (in the grey-matter skills repo) charts a large effort
as a `wayfinder:map` epic whose children are decision tickets; a session working that
map wants *its* ready children, not every ready ticket across the repo. With many maps
or other work in flight the global frontier is too noisy to drive one map.

- Add `tkt ready --parent <id>` returning only the ready children of that parent.
- Reuse the existing readiness predicate; only the candidate set changes (children of
  `<id>` rather than all tickets). Same sort order.
- Decide the interim story: either also honor the `epic:<slug>` tag convention
  (`tkt ready --epic <slug>`) so wayfinder works before native `parent` lands, or
  document this as parent-only and have wayfinder keep using `tkt ready` + manual
  filtering until 196/197 ship. Record the decision.

Blocked by 196 (parent CLI/query surface) and 197 (parent frontier semantics): a parent
must exist and be excluded from the frontier before scoping to it is meaningful.

Touches `src/commands/ready.rs` and the frontier helper in `src/core/ticket.rs`.

## Acceptance criteria

- [ ] `tkt ready --parent <id>` lists only open, unblocked, unclaimed children of that parent, in priority-then-id order
- [ ] Interim behavior (epic-tag filter vs parent-only) decided and documented
- [ ] Driving use case (wayfinder per-map frontier) noted here

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:ready-for-agent`; stream `cli`.

Keep #196/#197 dependencies. Parent-scoped frontier is absent; reuse readiness and priority/ID ordering. Decide parent-only versus interim tag alias without inventing computed sort overrides.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

## Implementation decision

Native-parent-only: parent fields and container behavior have shipped, so no interim epic-tag alias is needed. Existing context can still scope `epic:slug` tags. The wayfinder per-map session use case above drives this direct-child filter; it reuses the global frontier predicate and ordering.

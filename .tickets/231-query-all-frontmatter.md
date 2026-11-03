---
id: "231"
title: "tkt query outputs every frontmatter field, including tags"
status: open
blocked_by: []
priority: medium
validation_criteria:
  - "query JSON includes tags"
  - "query JSON includes every stored field"
tags: ["cli", "kind:bug", "triage:ready-for-agent", "stream:workflow"]
---

# tkt query outputs every frontmatter field, including tags

## Origin

Field report from philippines-trip, 2026-11-01 (prioritization review, location planning and a Codex worktree takeover). Sam asked for these to be filed.

## What happened

`tkt query` omitted `tags` for tickets that have them (for example philippines-trip #163), so grouping by work stream needed a separate parser.

## Acceptance criteria

- [ ] `tkt query` JSON includes `tags`, `validation_criteria`, `requires`, `agents` and any other stored frontmatter field.
- [ ] Capabilities and docs list the fields; a test covers a ticket with every field set.

## Triage — 2026-11-02

Disposition: `kind:bug`, `triage:ready-for-agent`; stream `cli`.

Confirmed: query omits tags/requires/custom even when stored, but agents is already emitted. Preserve agents and add raw-field coverage without losing typed known-field semantics; coordinate parent with #196.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

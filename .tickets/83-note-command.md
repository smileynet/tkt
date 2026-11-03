---
id: "83"
title: "Add a noninteractive ticket-note append operation"
status: open
blocked_by: []
priority: low
validation_criteria:
  - "Explicit note input is appended atomically to the correct ticket without launching an editor or changing lifecycle state"
  - "Existing specification, resolution and evidence are preserved; only intended paths are committed and push policy is honored"
  - "Missing IDs, dry-run and concurrent/retry behavior have public integration coverage"
tags: ["cli", "kind:enhancement", "stream:workflow", "triage:ready-for-agent"]
---

# Add a noninteractive ticket-note append operation

## Owner decision — 2026-11-03

Confirmed in chat: **Start with a noninteractive append operation**. This supersedes conflicting
requirements and the earlier dated decision gate; it does not claim implementation.
See [the authoritative record](../docs/decision-record-2026-11-03.md).

## Context

dstask has a `note` command that opens the task's markdown body in $EDITOR. Currently tkt users need to know the file path (`.tickets/{id}-{slug}.md`) to edit the body. A `tkt note <id>` command removes that friction.

## What to build

Start with a noninteractive append operation that resolves the ticket ID, accepts explicit note content and appends attributable text atomically. Commit only the intended ticket paths and honor push settings, dry-run and concurrency behavior. Preserve specification/resolution/evidence content and lifecycle status. Interactive editor launching is separate, deferred scope.

## Acceptance criteria

- [ ] Explicit note input is appended atomically to the correct ticket without launching an editor or changing lifecycle state
- [ ] Existing specification, resolution and evidence are preserved; only intended paths are committed and push policy is honored
- [ ] Missing IDs, dry-run and concurrent/retry behavior have public integration coverage

## Field report — 2026-11-01

Agents can't use an editor. Proposed non-interactive form: `tkt note <id> --append "text"` (or `--file`) adds a dated `## Note — YYYY-MM-DD` section and commits it. In one philippines-trip session, ticket bodies were hand-edited about 15 times for decisions and status notes.

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:needs-info`; stream `cli`.

Deferred proposal combines a noncommitting editor with a committing append form. Decide the mutation/commit contract and noninteractive input before implementation.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

## Planning gate

Resolved by the owner's 2026-11-03 chat submission: **Start with a noninteractive append operation**.
See [the decision record](../docs/decision-record-2026-11-03.md). Backlog remains deliberately deferred.

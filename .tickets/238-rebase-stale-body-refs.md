---
id: "238"
title: "Renumbering leaves stale ID references in ticket bodies"
status: in_progress
blocked_by: []
validation_criteria:
  - "tkt rebase reports, or rewrites, body references to renumbered IDs"
  - "tkt validate warns on #NNN / ticket NNN references to IDs that do not exist"
tags: ["cli", "kind:bug", "triage:needs-info", "stream:workflow", "gate:evidence"]
assignee: "codex"
---

# Renumbering leaves stale ID references in ticket bodies

## Problem

`tkt rebase` renumbered grey-matter 137→152 and 138→153, but prose references in ticket bodies
("see 138 item 7", "Companion to 137") kept the old numbers and had to be found by grep and fixed
by hand.

## Context

Found 2026-11-01 in a philippines-trip retro. Related done work: #134 (renumber birth-window
citation scan), which covers manual `renumber` but not the collision-recovery rebase path.

## Acceptance criteria

- [ ] `tkt rebase` reports, or rewrites, body references to renumbered IDs.
- [ ] `tkt validate` warns on `#NNN` / `ticket NNN` references to IDs that do not exist.

## Triage — 2026-11-02

Disposition: `kind:bug`, `triage:needs-info`; stream `cli`.

Confirmed dangling #999/ticket 998 body references pass validate, but collision-renumber stale-body behavior was not reproduced. Capture both local/upstream before/after bytes before changing rebase; false-positive/body-ownership policy needs a bounded design.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

## Planning gate

Capture local/upstream before-and-after bytes for a collision rebase and stale body references.

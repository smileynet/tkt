---
id: "238"
title: "Renumbering leaves stale ID references in ticket bodies"
status: done
blocked_by: []
validation_criteria:
  - "tkt rebase reports, or rewrites, body references to renumbered IDs"
  - "tkt validate warns on #NNN / ticket NNN references to IDs that do not exist"
tags: ["cli", "kind:bug", "triage:ready-for-agent", "stream:workflow"]
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

- [x] `tkt rebase` reports, or rewrites, body references to renumbered IDs.
- [x] `tkt validate` warns on `#NNN` / `ticket NNN` references to IDs that do not exist.

## Triage — 2026-11-02

Disposition: `kind:bug`, `triage:needs-info`; stream `cli`.

Confirmed dangling `#999`/`ticket 998` body-reference examples pass validate, but collision-renumber stale-body behavior was not reproduced. Capture both local/upstream before/after bytes before changing rebase; false-positive/body-ownership policy needs a bounded design.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

## Planning gate

The reproduction gate is cleared: the collision fixture captured local/upstream
Git blob identities and reproduced unchanged stale body citations. The implementation
reports candidates without rewriting prose; focused checks pass. See
[rebase evidence](../docs/evidence/rebase-body-238.md). Final full-gate verification passed; see the evidence report. External issue references in three historical tickets were qualified
without changing their recorded acceptance or resolution.

## Resolution (2026-11-03)

Rebase reports possible stale prose citations without rewriting bodies, and validation flags missing local numeric references. Independent critic accepted final source. Evidence promoted; shared cache retained for maintenance and temporary delivery helpers end after dashboard publication.

### Verification
1. ✓ tkt rebase reports, or rewrites, body references to renumbered IDs — "Collision fixture verifies old/new citation reporting in preview and real rebase while local/upstream bodies remain unchanged; docs/evidence/rebase-body-238.md."
2. ✓ tkt validate warns on #NNN / ticket NNN references to IDs that do not exist — "Both dangling citation syntaxes warn; code, URL and external-link exclusions pass. Installed strict corpus validation reports zero findings; final gate 384 Rust tests passed."

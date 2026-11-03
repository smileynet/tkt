---
id: "228"
title: "Migrate status dashboard to the current Plan-tab contract"
status: in_progress
blocked_by: []
validation_criteria:
  - "Dashboard preserves existing entries, renders an all-ticket Plan tab, supports snapshot refresh, and passes the current checker without warnings"
tags: ["tooling", "kind:enhancement", "triage:ready-for-agent"]
---

# Migrate status dashboard to the current Plan-tab contract

## What to build

Migrate docs/status.html to the current delivery-dashboard contract, preserving its history and adding the generated all-ticket Plan tab. Align the project editing instructions with the migrated template before using snapshot-tickets.mjs.

## Context

Intent source: the 2026-10-29 session wrap retrospective. The current dashboard checker exits successfully but warns that the gm-tickets JSON block is missing. The current snapshot helper requires that block. AGENTS.md currently permits edits only to gm-config and gm-entries, so this requires a coordinated migration rather than silently adding renderer/data blocks during unrelated work.

- Dashboard: docs/status.html.
- Current template and helpers: the installed delivery-dashboard skill (assets/dashboard-template.html; scripts/check-dashboard.mjs; scripts/snapshot-tickets.mjs).
- Preserve existing entries and relative assets such as docs/agent-eligibility-model.svg.
- #226 is a separate proposal for broader AGENTS.md restructuring; this ticket only updates the dashboard-specific editing contract.

## Acceptance criteria

- [ ] Preserve every existing dashboard entry and its linked assets.
- [ ] Add the current Plan-tab renderer and generated gm-tickets snapshot for the entire board, without an active tag context.
- [ ] Align AGENTS.md dashboard instructions with the generated-block ownership rules.
- [ ] Re-running snapshot-tickets.mjs refreshes the Plan tab without altering entries.
- [ ] Current check-dashboard.mjs passes without errors or warnings.
- [ ] Inspect overview, history, and Plan views at phone and laptop widths.

## Out of scope

Broad AGENTS.md restructuring, ticket status/priority changes, release/version changes, and rewriting dashboard history.

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:ready-for-agent`; stream `tooling`.

Dashboard lacks gm-tickets and Plan renderer. Migration is separate work; this reconciliation changes only gm-entries, preserves history, and records the existing checker warning.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

## Implementation progress — 2026-11-02

Migrated to the canonical installed dashboard template, preserving all 16 prior
entries and local asset links; refreshed all 239 tickets and 30 frontier IDs.
Updated only AGENTS.md's dashboard ownership paragraph. Snapshot repeatability,
canonical renderer identity, JavaScript syntax, all IDs/statuses and dependency
chain 193→198→199 passed static checks. See
[durable migration evidence](../docs/evidence/dashboard-migration-228.md).

The local browser tool returned “Local browser use is disabled in Work settings.”
Phone/laptop tabs, interactions and clipping remain unverified. No alternate route
was attempted and no accepted receipt was created. Keep this ticket in progress
until the final visual AC is observed; the dashboard carries an open review ask.
No scratch/capture artifacts were made.

---
id: "228"
title: "Migrate status dashboard to the current Plan-tab contract"
status: open
blocked_by: []
validation_criteria:
  - "Dashboard preserves existing entries, renders an all-ticket Plan tab, supports snapshot refresh, and passes the current checker without warnings"
tags: ["tooling"]
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

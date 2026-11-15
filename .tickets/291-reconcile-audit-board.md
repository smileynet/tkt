---
id: "291"
title: "Reconcile the current board and active-ticket quality"
status: done
blocked_by: []
priority: high
validation_criteria:
  - "tkt sync-plan --check --brief exits zero"
  - "tkt validate --brief has no active-ticket errors or undocumented readiness findings"
  - "historical statuses, criteria, resolutions, and evidence are unchanged"
tags: ["stream:quality", "kind:enhancement", "triage:ready-for-agent"]
assignee: "kimi"
---
# Reconcile the current board and active-ticket quality

## Intent source

Owner-approved audit on 2026-11-13 and implementation approval on 2026-11-15. At pulled HEAD `9b0cb2f`, `sync-plan --check` reports missing rows for tickets 287 and 288, the current-queue narrative says no ready ticket remains despite three unblocked open tickets, and validation reports active-ticket quality findings.

## Context

- `docs/plan.md` current maintenance/execution sections and Unplanned tickets table.
- `.tickets/287-plugin-manifest-single-sourcing.md` lacks body acceptance criteria.
- `.tickets/288-publish-github-marketplace.md` uses an unqualified local-looking reference to grey-matter ticket 292.
- Tickets 269/270 contain historical notes with literal escaped newlines alongside corrected notes.
- The owner explicitly prioritized the six audit follow-ups created in this session.

## What to build

The live plan distinguishes current work from historical receipts, contains every active ticket, and matches the actual frontier. Tickets 287 and 288 meet the repository's content/reference quality bar without changing their implementation scope or activation state. Historical notes remain truthful; only demonstrable formatting defects are normalized.

Record the six audit follow-ups in priority order and keep the existing assigned/gated tickets visible without inventing dependencies.

## Acceptance criteria

- [x] `tkt sync-plan --check --brief` exits zero with all active and new tickets represented.
- [x] The current plan no longer claims that no ready ticket remains and names the audit follow-ups as current work.
- [x] Ticket 287 has intent/context, behavioral scope, and concrete body acceptance criteria.
- [x] Ticket 288 qualifies grey-matter ticket 292 so validation emits no dangling local reference.
- [x] `tkt validate --brief` emits no findings for active tickets 90, 147, 184, 269, 270, 287, 288, or the new audit follow-ups, apart from explicitly documented informational readiness findings.
- [x] Existing historical warnings and evidence remain otherwise unchanged.

## Out of scope

Implementing tickets 287/288, clearing external evidence gates, changing ownership, or fabricating missing historical closure evidence.

## Resolution (2026-11-15)

Reconciled the live audit queue, added missing plan rows, groomed tickets 287/288, and preserved historical records.

### Verification
1. ✓ tkt sync-plan --check --brief exits zero — "tkt sync-plan --check --brief => pass (0 finding(s)); plan includes 287-294 and preserves existing work"
2. ✓ tkt validate --brief has no active-ticket errors or undocumented readiness findings — "tkt validate --brief => pass with only eight historical missing-resolution warnings; no active-ticket or readiness findings"
3. ✓ historical statuses, criteria, resolutions, and evidence are unchanged — "Scoped diff and git history show only new plan narrative/rows and approved ticket-body grooming; no historical statuses, criteria, resolutions or evidence changed"

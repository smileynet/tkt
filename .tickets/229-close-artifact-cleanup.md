---
id: "229"
title: "tkt-work close step cleans up the ticket's scratch and build artifacts"
status: done
blocked_by: []
priority: medium
validation_criteria:
  - "tkt-work's close step tells the agent to promote durable outputs, delete the ticket's scratch and build artifacts, and keep only items with a stated lifecycle scope"
  - "The Resolution names what was promoted, deleted or kept, and why anything kept is still needed"
  - "init snippets and tkt-reconcile carry the same rule without contradicting it"
tags: ["guidance", "kind:enhancement", "triage:ready-for-agent", "stream:guidance"]
---

# tkt-work close step cleans up the ticket's scratch and build artifacts

## What to build

Owner's rule (2026-10-29, also grey-matter ticket 97): when closing a ticket, clean up
its scratch and build artifacts. Promote what is needed durably into tracked files and
cite those; keep an uncommitted item only while something still needs it, with a clear
lifecycle scope (which tickets, release or shared purpose, and when it ends); keep a
reasonably sized history of attempts rather than everything or nothing; delete the rest,
including build output and work directories outside the repo.

tkt ships its workflow skills standalone, so `skills/tkt-work/SKILL.md` (close step,
around "Close with evidence") states the rule in its own words instead of pointing at
grey-matter. `tkt-reconcile` checks for kept items whose scope has ended, and the `init`
snippets mention it in one line. Keep it guidance; no new command or ledger.

## Acceptance criteria

- [x] tkt-work's close step covers promote, delete and keep-with-scope, including build artifacts outside `.scratch`
- [x] The Resolution records what was promoted, deleted or kept, with the reason for anything kept
- [x] tkt-reconcile flags kept items whose lifecycle scope has ended
- [x] init snippets carry a one-line version consistent with the skill
- [x] Repository checks (`mise run check`) pass

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:ready-for-agent`; stream `guidance`.

Close-artifact guidance remains portable standalone tkt scope. Sync work/reconcile/init wording; do not introduce a foreign-skill dependency or delete artifacts of unknown ownership.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

## Resolution (2026-11-03)

Portable work/reconcile/init guidance now promotes durable outputs, trashes owned disposable artifacts and records scoped retention. Evidence promoted; fixtures self-clean, disposable logs to OS trash, shared target retained through queue. Independent critic accepted. docs/evidence/parent-cli-labels-debug-196-198-237.md.

### Verification
1. ✓ tkt-work's close step tells the agent to promote durable outputs, delete the ticket's scratch and build artifacts, and keep only items with a stated lifecycle scope — "Work close guidance covers promote/trash/keep with scope including outside scratch and repo."
2. ✓ The Resolution names what was promoted, deleted or kept, and why anything kept is still needed — "Resolution guidance records disposition and retention reason; this Resolution applies it."
3. ✓ init snippets and tkt-reconcile carry the same rule without contradicting it — "Reconcile removes expired owned retained items; all six init output targets share consistent rule; full gate passed."

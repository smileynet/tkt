---
id: "144"
title: "Context silently narrows query (full corpus claim violated)"
status: done
blocked_by: []
priority: low
validation_criteria:
  - "tkt query ignores context or documents the filtering"
tags: ["cli", "kind:bug", "triage:ready-for-agent", "stream:quality"]
assignee: "codex"
---

# Context silently narrows query (full corpus claim violated)

## What to build

`tkt query` applies the active tag context (`TKT_CONTEXT`, else `.tickets/.context`), but
its help says "Dump all tickets as JSON Lines". Observed on 0.5.0 (`a827bb6`, 2026-10-29):
with context `ink` set in a scratch repo, `query` returned the `ink` ticket and every
untagged ticket and would silently drop tickets tagged otherwise; `query --help` never
mentions context.

tkt's own workflow skills (`tkt-reconcile`) rely on context-scoped queries, so the default
must keep honouring context. Make the filter visible and bypassable instead: say so in
the help, note on stderr when a context narrows the output, and add a flag that returns
the whole corpus.

Consumer: grey-matter's delivery-dashboard `snapshot-tickets.mjs` refuses to run while a
context is set; it would pass the bypass flag instead.

## Acceptance criteria

- [x] `tkt query --help` says the active tag context filters results and names the bypass flag
- [x] With a context set, the bypass flag returns every ticket; without it, output is unchanged
- [x] When a context narrows the output, a notice goes to stderr only, never into JSON stdout
- [x] An integration test covers context plus bypass, and the tkt-reconcile context flow still passes

## Triage — 2026-11-02

Disposition: `kind:bug`, `triage:ready-for-agent`; stream `cli`.

Confirmed: TKT_CONTEXT=other query hides an ink-tagged ticket and retains an untagged ticket, with no stderr notice. Keep existing scoped default; add/document bypass.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

## Resolution (2026-11-03)

Query preserves scoped default, reports actual narrowing, and offers all-context bypass without changing context. Guidance synchronized and independent critic accepted. Evidence promoted; shared build cache retained and temporary helpers expire at dashboard publication.

### Verification
1. ✓ tkt query ignores context or documents the filtering — "Context/bypass integration tests passed with JSONL stderr notice and clean stdout; final gate and installed all-context query cover all 241 tickets. See docs/evidence/quiet-json-context-143-144.md."

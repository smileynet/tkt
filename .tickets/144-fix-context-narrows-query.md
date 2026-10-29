---
id: "144"
title: "Context silently narrows query (full corpus claim violated)"
status: backlog
blocked_by: []
priority: low
validation_criteria:
  - "tkt query ignores context or documents the filtering"
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

- [ ] `tkt query --help` says the active tag context filters results and names the bypass flag
- [ ] With a context set, the bypass flag returns every ticket; without it, output is unchanged
- [ ] When a context narrows the output, a notice goes to stderr only, never into JSON stdout
- [ ] An integration test covers context plus bypass, and the tkt-reconcile context flow still passes

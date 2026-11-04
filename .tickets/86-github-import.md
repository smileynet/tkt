---
id: "86"
title: "tkt import --github: pull open issues into .tickets/"
status: open
blocked_by: ["125"]
priority: low
validation_criteria:
  - "`tkt import --github` imports open issues from current repo"
tags: ["integrations", "kind:enhancement", "triage:ready-for-agent", "stream:workflow"]
---

# tkt import --github: pull open issues into .tickets/

## Current executable contract — 2026-11-03

One-way GitHub import is blocked by ticket 125. Follow docs/github-adapter-contract.md: canonical identity is host plus numeric issue ID, with repository numeric ID/name, number and URL as checked provenance. Extract bounded state=all pages but create open issues only; closed/PR counts describe received objects. Preserve labels in raw snapshots without automatic tags/priority/ownership mapping; fence source Markdown, preserve null body as JSON null and render an empty source section. Use create-only replay deduplication and journaled atomic ticket/snapshot ownership. The historical source: github#NN idea alone is insufficient across repositories. Preserve unsupported raw source data without claiming it is supported typed frontmatter; do not write to GitHub. Implement only after the mapping contract is verified.

This contract supersedes conflicting historical proposals below.

## Context

GitHub/GitLab bridges create switching-cost stickiness. git-bug and git-issue both offer bidirectional sync. A one-way import (GitHub → .tickets/) is the minimum viable bridge — lets teams try tkt without abandoning their existing issue tracker.

## What to build

```bash
tkt import --github
# → Imported 12 open issues from smileynet/tkt
# →   79  undo-command (from #42)
# →   80  context-system (from #43)
# →   ...

tkt import --github --label "ready-for-agent"
# → Imported 3 issues matching label 'ready-for-agent'
```

Behavior:
- Requires `gh` CLI authenticated
- Imports open issues (configurable: `--state all`)
- Maps: title → title, body → ticket body, labels → tags (if context system exists)
- Assigns next available IDs
- Adds `source: github#NN` to frontmatter for traceability
- Does NOT sync back (one-way import, not bidirectional)
- Handles ID conflicts via standard allocation

## Acceptance criteria

- [ ] `tkt import --github` imports open issues from current repo
- [ ] Requires `gh` CLI on PATH and authenticated
- [ ] Maps issue title, body, and labels into ticket format
- [ ] Adds `source: github#NN` frontmatter field
- [ ] `--label` flag filters which issues to import
- [ ] Allocates IDs without conflicts
- [ ] Commits and pushes imported tickets
- [ ] Graceful error if `gh` not available or not authenticated


## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:ready-for-agent`; stream `integrations`.

Deferred one-way import. src/cli.rs has migrate but no import command; preserve source issue identity and bounded allocation. No external service changes are authorized by triage.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.


## Reactivation — 2026-11-03

The owner requested review of all parked work and execution of actionable tickets. This supersedes earlier blanket deferral. See [the parked-work review](../docs/parked-review-2026-11-03.md) for current scope and sequencing. Independent critic review is required before closure.

## Cross-project evidence review — 2026-11-03

Grey Matter tickets 10 and 11 retain a concrete downstream tracker/triage use case for an issue import bridge. This supports the already-open implementation scope, not reopening the completed adapter design (#125) or claiming the importer exists. Keep the create-only identity/replay contract and explicit opt-in boundaries. See [the evidence review](../docs/evidence/cross-project-review-2026-11-03.md).

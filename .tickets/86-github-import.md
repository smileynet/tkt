---
id: "86"
title: "tkt import --github: pull open issues into .tickets/"
status: backlog
blocked_by: []
priority: low
validation_criteria:
  - "`tkt import --github` imports open issues from current repo"
tags: ["integrations", "kind:enhancement", "triage:ready-for-agent", "stream:workflow"]
---

# tkt import --github: pull open issues into .tickets/

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

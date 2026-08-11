---
id: "86"
title: "tkt import --github: pull open issues into .tickets/"
status: open
blocked_by: []
priority: low
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

# tkt import --github: pull open issues into .tickets/

## What to build

TBD

## Acceptance criteria

- [ ] TBD

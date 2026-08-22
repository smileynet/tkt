---
id: "80"
title: "Context system: auto-scope reads and writes by tag"
status: in_progress
blocked_by: []
priority: medium
validation_criteria: 
  - "tkt context +tag sets and filters ready output"
  - "tkt new auto-tags from active context"
---

# Context system: auto-scope reads and writes by tag

## Context

dstask's context system is the best UX innovation in the file-based task space. Setting a context auto-filters ALL operations (reads AND writes). tkt's `CREW_ENV` is a primitive version limited to the `env` field. A generalized context system would let agents and humans work in a focused scope without explicitly filtering every command.

## What to build

A context that filters `ready`, `query`, `blocked` AND auto-tags `new`/`batch` tickets:

```bash
tkt context +backend           # set context
tkt ready                      # only shows tickets tagged 'backend'
tkt new auth --title "..."     # auto-tagged with 'backend'
tkt context                    # show current context
tkt context --clear            # remove context
```

Implementation options:
- Store context in `.tickets/config.toml` (project-level, committed)
- Store in user config `~/.config/tkt/config.toml` (personal, not committed)
- Support `TKT_CONTEXT` env var (like dstask's direnv integration)
- Filter on a new `tags` frontmatter field (array of strings)

## Acceptance criteria

- [ ] `tkt context +tag` sets active context
- [ ] `tkt context --clear` removes it
- [ ] `tkt context` (no args) shows current context
- [ ] `tkt ready` filters by context tags
- [ ] `tkt new` auto-applies context tags to new tickets
- [ ] `TKT_CONTEXT` env var overrides stored context
- [ ] Works alongside existing `CREW_ENV` filtering (additive)
- [ ] Integration test for context read/write scoping

# Context system: auto-scope reads and writes by tag

## What to build

TBD

## Acceptance criteria

- [ ] TBD

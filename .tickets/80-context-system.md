---
id: "80"
title: "Context system: auto-scope reads and writes by tag"
status: done
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

- [x] `tkt context +tag` sets active context
- [x] `tkt context --clear` removes it
- [x] `tkt context` (no args) shows current context
- [x] `tkt ready` filters by context tags
- [x] `tkt new` auto-applies context tags to new tickets
- [x] `TKT_CONTEXT` env var overrides stored context
- [x] Works alongside existing `CREW_ENV` filtering (additive)
- [x] Integration test for context read/write scoping

# Context system: auto-scope reads and writes by tag

## What to build

TBD

## Acceptance criteria

- [ ] TBD

## Resolution (2026-08-22)

Implemented full context system: tags field on tickets, .tickets/.context storage, TKT_CONTEXT env override, filtering on ready/query/blocked, auto-tagging on new/batch. 9 unit tests + e2e verified.

### Verification
1. ✓ tkt context +tag sets and filters ready output — "tkt context +backend → ready shows only backend-tagged + untagged; tkt new creates ticket with tags:[backend]"
2. ✓ tkt new auto-tags from active context — "TKT_CONTEXT env override works; --clear removes context; -tag exclude works"

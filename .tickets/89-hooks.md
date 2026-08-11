---
id: "89"
title: "Lifecycle hooks: on-new, on-claim, on-close scripts"
status: open
blocked_by: []
priority: low
---

# Lifecycle hooks: on-new, on-claim, on-close scripts

## Context

Taskwarrior's hook system creates a platform effect — time tracking, email notifications, mobile sync, and gamification all plug in. Hooks let users extend tkt without modifying the tool itself.

## What to build

Scripts in `.tickets/hooks/` run at lifecycle points:

```
.tickets/hooks/
├── on-new.sh        # runs after tkt new (receives ID, slug, title)
├── on-claim.sh      # runs after tkt claim (receives ID)
├── on-close.sh      # runs after tkt close (receives ID, resolution)
└── on-edit.sh       # runs after tkt edit (receives ID, changed fields)
```

Hook contract:
- Executable files in `.tickets/hooks/`
- Named `on-{event}.{ext}` (any extension, must be executable)
- Receive ticket data as environment variables: `TKT_ID`, `TKT_SLUG`, `TKT_TITLE`, `TKT_STATUS`
- Exit code 0 = success (tkt continues), non-zero = warning (tkt still continues, prints warning)
- Hooks run AFTER the mutation succeeds (post-hooks only, no pre-hooks for v1)
- Timeout: 10s max per hook

## Acceptance criteria

- [ ] Post-hooks fire for new, claim, close, edit
- [ ] Scripts receive ticket data via environment variables
- [ ] Non-zero exit prints a warning but doesn't fail the operation
- [ ] Hooks directory is optional (no hooks = no overhead)
- [ ] 10-second timeout per hook
- [ ] Hook execution doesn't block push operations
- [ ] Integration test with a simple hook script

# Lifecycle hooks: on-new, on-claim, on-close scripts

## What to build

TBD

## Acceptance criteria

- [ ] TBD

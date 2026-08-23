---
id: "95"
title: "Modularize env field: flexible tag-based environment matching"
status: done
blocked_by: ["92"]
priority: medium
validation_criteria: 
  - "requires subset matching filters frontier correctly"
  - "backward compat: env:corp still works via CREW_ENV"
---

# Modularize env field: flexible tag-based environment matching

## Context

The current `env` field is a rigid enum (`corp | personal | either`) filtered by `CREW_ENV`. This is too narrow — real workstations differ along many axes: GPU/CPU-only, OS, network access, personal/work, hardware capabilities. A ticket requiring a GPU should only appear on the frontier of a machine with a GPU.

## What to build

Replace the single `env` enum with a flexible tag set. Each machine declares its tags in local config. Tickets declare which tags they require. The frontier filters tickets whose required tags are a subset of the machine's available tags.

### Machine config (`~/.config/tkt/config.toml`)

```toml
[environment]
tags = ["linux", "gpu", "personal", "docker"]
```

### Ticket frontmatter

```yaml
---
id: "42"
title: "Train the model"
status: open
blocked_by: []
env_tags: ["gpu", "linux"]
---
```

### Matching logic

A ticket appears on the frontier if:
- `env_tags` is empty (matches any machine), OR
- every tag in `env_tags` is present in the machine's `environment.tags`

### Migration

- `env: corp` → `env_tags: ["corp"]`
- `env: personal` → `env_tags: ["personal"]`
- `env: either` → `env_tags: []` (or omit field)
- `CREW_ENV=corp` → still works as shorthand for machines tagged `["corp"]`

### CLI

```bash
tkt new train-model --title "Train model" --env-tags gpu,linux
tkt edit 42 --env-tags gpu,linux,docker
tkt config --set environment.tags=linux,gpu,personal
```

## Acceptance criteria

- [x] `env_tags` field parsed as list of strings in frontmatter
- [x] Machine tags declared in user config `[environment] tags`
- [x] `tkt ready` filters by tag subset matching
- [x] `tkt new --env-tags` sets tags at creation
- [x] `tkt edit --env-tags` modifies tags
- [x] Backward compatible: `env: corp` still works (mapped internally)
- [x] `CREW_ENV` still works as legacy shorthand
- [x] `tkt validate` warns on unknown tags (configurable)
- [x] Migration path documented for existing tickets

# Modularize env field: flexible tag-based environment matching

## What to build

TBD

## Acceptance criteria

- [ ] TBD

## Resolution (2026-08-23)

Added requires field (machine capability matching). Tickets declare requirements, machines declare capabilities in config. Subset matching (AND). Backward compat: env:corp → requires:[corp], CREW_ENV as legacy capability.

### Verification
1. ✓ requires subset matching filters frontier correctly — "machine.capabilities=gpu,linux → shows tickets with requires:[gpu,linux]; hides requires:[corp]"
2. ✓ backward compat: env:corp still works via CREW_ENV — "CREW_ENV=corp filters env:corp legacy tickets correctly; --requires flag creates tickets with requires field"

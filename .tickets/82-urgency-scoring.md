---
id: "82"
title: "Urgency scoring: computed numeric sort for frontier"
status: open
blocked_by: []
priority: medium
validation_criteria: 
  - "deliberate ordering reviewed"
---

# Urgency scoring: computed numeric sort for frontier

## Context

Taskwarrior's urgency coefficient system removes decision fatigue by computing a single numeric score. tkt already sorts frontier by priority bucket then ID — but this is coarse. A visible urgency score (combining priority × age × how-many-things-this-blocks) would make frontier ordering transparent and tunable.

## What to build

Compute an urgency score per frontier ticket, show it in `ready` output:

```bash
tkt ready
# Ready (3):
#   03 [8.2]  Deploy pipeline          (high, blocks 2)
#   05 [5.0]  Write tests              (medium, 12d old)
#   07 [3.1]  Update docs              (low)
```

Scoring formula (configurable via config.toml):
- Priority: urgent=8, high=4, medium=2, low=1
- Blocks count: +1 per ticket this unblocks
- Age: +0.1 per day since creation (caps at 3.0)
- Total: sum of components

Config override in `.tickets/config.toml`:
```toml
[urgency]
priority_urgent = 8.0
priority_high = 4.0
priority_medium = 2.0
priority_low = 1.0
blocks_weight = 1.0
age_weight = 0.1
age_cap = 3.0
```

## Acceptance criteria

- [ ] `tkt ready` shows urgency score next to each ticket
- [ ] Score computed from priority + blocks-count + age
- [ ] Frontier sorted by score descending (replaces current priority+ID sort)
- [ ] `tkt ready --json` includes urgency field
- [ ] Weights configurable via `[urgency]` section in config.toml
- [ ] Sensible defaults work without any config
- [ ] Score visible but not overwhelming in human output

# Urgency scoring: computed numeric sort for frontier

## What to build

TBD

## Acceptance criteria

- [ ] TBD

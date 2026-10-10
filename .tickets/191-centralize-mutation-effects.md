---
id: "191"
title: "Centralize command mutation effects and dry-run policy"
status: backlog
blocked_by: ["136"]
priority: medium
validation_criteria:
  - "Every mutating command uses one plan-and-apply effect path"
  - "Dry run performs no persistent filesystem or Git mutation"
  - "The shared executor preserves atomic writes and push gating"
  - "Table-driven tests cover no-write dry-run behavior"
tags: ["architecture", "integrity"]
---

# Centralize command mutation effects and dry-run policy

## What to build

TBD

## Acceptance criteria

- [ ] TBD

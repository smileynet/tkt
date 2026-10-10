---
id: "193"
title: "[labels.*] config: parse single-select + closed-value namespaces"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "config.rs parses [labels.<ns>] tables with single_select bool and values list"
  - "Unknown/malformed [labels.*] entries produce a clear config error"
  - "Existing configs without [labels.*] continue to load unchanged (backward compatible)"
---

# [labels.*] config: parse single-select + closed-value namespaces

## What to build

TBD

## Acceptance criteria

- [ ] TBD

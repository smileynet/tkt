---
id: "211"
title: "config tests race on TKT_NO_USER_CONFIG (set_var/remove_var across parallel tests)"
status: open
blocked_by: []
priority: medium
validation_criteria:
  - "cargo test passes 20 consecutive runs of 'cargo test --bin tkt config::tests' with default parallelism"
tags: ["type:bug"]
---

# config tests race on TKT_NO_USER_CONFIG (set_var/remove_var across parallel tests)

## What to build

TBD

## Acceptance criteria

- [ ] TBD

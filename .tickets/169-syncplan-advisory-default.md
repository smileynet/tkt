---
id: "169"
title: "sync-plan: advisory-by-default (demote plan-status-drift to warning, repurpose --check as CI gate)"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "default sync-plan run with drift exits 0 (test: integration::sync_plan_advisory_default)"
  - "sync-plan --check exits 1 on drift (test: integration::sync_plan_check_gate)"
  - "sync-plan --fix --dry-run previews without writing (test: integration::sync_plan_fix_dryrun)"
---

# sync-plan: advisory-by-default (demote plan-status-drift to warning, repurpose --check as CI gate)

## What to build

TBD

## Acceptance criteria

- [ ] TBD

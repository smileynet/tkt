---
id: "251"
title: "Preserve migration sources and foreign destinations"
status: in_progress
blocked_by: []
priority: high
validation_criteria:
  - "Observed collision and failed-publication regressions preserve source and foreign destination bytes"
  - "Prepared complete-file publication preserves backups and source data without a false batch-atomic or locking guarantee"
  - "Shared no-clobber publication retains hooks/projection and migration contracts, dry-run behavior, full gate and critic acceptance"
tags: ["stream:quality", "kind:bug", "triage:ready-for-agent"]
parent: "191"
assignee: "codex"
---
# Preserve migration sources and foreign destinations

Reporter: Codex
Confirmation status: confirmed
Parent: #191

## Evidence

The current migration apply loop removes a source before writing its destination
and uses a truncating write. Two focused Rust regressions now fail: an unlocked
file arriving after planning is overwritten, and a Windows-locked destination
causes failure after the original source has been removed. The tests use a nested
private .tickets directory and private backup root.

## What to build

Prepare readable original/converted text before persistent effects. Publish new
migration tickets atomically without replacing a foreign arrival, and remove the
original only after successful publication. Keep backups complete and preserve
conflicting old backup data. Reuse the existing no-clobber publication behavior
from hooks/projection through one small shared primitive, retaining executable
mode, error categories and recovery limits. Preserve the separate Git transaction
lifecycles and the public migration interface. Do not claim batch-transactional
recovery or external-editor locking.

## Acceptance criteria

- [ ] Both demonstrated collision/failure regressions preserve original sources and foreign target bytes.
- [ ] Migration output and backups use safe complete-file publication; failures do not delete the last original.
- [ ] Hooks/projection ownership tests and migration compatibility pass after sharing the primitive.
- [ ] Dry-run remains no-write, full verification passes and independent critic accepts before closure.

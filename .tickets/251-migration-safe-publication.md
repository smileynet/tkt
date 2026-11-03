---
id: "251"
title: "Preserve migration sources and foreign destinations"
status: done
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

- [x] Both demonstrated collision/failure regressions preserve original sources and foreign target bytes.
- [x] Migration output and backups use safe complete-file publication; failures do not delete the last original.
- [x] Hooks/projection ownership tests and migration compatibility pass after sharing the primitive.
- [x] Dry-run remains no-write, full verification passes and independent critic accepts before closure.

## Resolution (2026-11-03)

Prepared safe migration publication and backups, preserved sources/foreign destinations, and shared the no-clobber writer; final full gate and independent critic accepted. Parent 191 remains incomplete.

### Verification
1. ✓ Observed collision and failed-publication regressions preserve source and foreign destination bytes — "Both original collision/failure regressions went red before the fix and now pass, preserving original and foreign bytes. New source/backup and public preview/apply controls pass. See docs/evidence/migration-publication-251.md."
2. ✓ Prepared complete-file publication preserves backups and source data without a false batch-atomic or locking guarantee — "Readable complete source/output snapshots precede effects. Backups are retained without overwrite; originals are removed only after their output succeeds. Partial-batch and external-writer limits are explicit; no transactional guarantee is claimed."
3. ✓ Shared no-clobber publication retains hooks/projection and migration contracts, dry-run behavior, full gate and critic acceptance — "Final mise run check: 428 Rust tests and 42 Node fixtures passed; one existing Windows skip, clean fmt and zero clippy warnings. Hook/projection contracts and structured I/O boundary passed. Independent critic accepted final source; hashes recorded in companion JSON."

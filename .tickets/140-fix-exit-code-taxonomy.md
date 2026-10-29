---
id: "140"
title: "Fix exit code taxonomy leaks: config errors exit 2 instead of 1"
status: backlog
blocked_by: []
priority: low
validation_criteria:
  - "malformed --set exits 1 (Validation) not 2 (crash)"
---

# Fix exit code taxonomy leaks: config errors exit 2 instead of 1

## What to build

Re-triage 2026-10-29: not reproduced as titled. On 0.5.0, `tkt config --set nonsense.key=1`
exits 0 and accepts the unknown key, rather than exiting 2. Accepting unknown or invalid
keys silently is the real gap, and #190 (strict typed config) covers it. Record the
original malformed input if it differs; otherwise this folds into #190.

## Acceptance criteria

- [ ] The original malformed `--set` input and its exit code are recorded, or this ticket records that #190 supersedes it

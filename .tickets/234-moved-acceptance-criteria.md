---
id: "234"
title: "Acceptance criteria can be marked moved to another ticket"
status: open
blocked_by: []
priority: medium
validation_criteria:
  - "Moved state recognised by close and validate"
  - "Receiving ticket shows the moved criterion"
tags: ["cli"]
---

# Acceptance criteria can be marked moved to another ticket

## Origin

Field report from philippines-trip, 2026-11-01 (prioritization review, location planning and a Codex worktree takeover). Sam asked for these to be filed.

## What happened

Sam moved one unticked criterion from philippines-trip #78 to #99 and closed #78. The only way to close was to tick the criterion, which reads as if it were done.

## Acceptance criteria

- [ ] A criterion can be marked moved (for example `- [>] … → #99`), via `tkt close --move-ac N --to 99` or `tkt edit`.
- [ ] `close`, `validate` and `audit` treat moved criteria as resolved without claiming they were done, and validate checks the target exists.
- [ ] The receiving ticket gains the criterion with a back-reference.

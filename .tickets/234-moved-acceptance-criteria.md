---
id: "234"
title: "Acceptance criteria can be marked moved to another ticket"
status: open
blocked_by: []
priority: medium
validation_criteria:
  - "Moved state recognised by close and validate"
  - "Receiving ticket shows the moved criterion"
tags: ["cli", "kind:enhancement", "triage:needs-info"]
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

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:needs-info`; stream `cli`.

Moved-AC intent is current; decide marker grammar, multiple source/destination criteria, evidence mapping, idempotence and atomic multi-ticket recovery before implementation. Do not mark moved work done.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

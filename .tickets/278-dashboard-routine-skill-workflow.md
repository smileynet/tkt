---
id: "278"
title: "Keep routine dashboard verification in the delivery-dashboard workflow"
status: done
blocked_by: []
priority: medium
validation_criteria:
  - "AGENTS.md assigns routine docs/status.html generation, updates and verification scope to the delivery-dashboard skill without a separate standing validation ticket"
  - "Existing dashboard snapshot and strict checker requirements remain unchanged"
tags: ["guidance", "kind:enhancement", "triage:ready-for-agent", "stream:quality"]
spec: "AGENTS.md"
---
## Intent source

2026-11-08 retrospective. The session consolidated routine laptop/phone dashboard checks into ticket 277, then Sam rejected that standalone task and directed that docs/status.html simply follow the Grey Matter delivery-dashboard skill.

## What to change

Add a concise repo-local clarification to the existing AGENTS.md dashboard section. Routine dashboard maintenance follows the delivery-dashboard skill as its complete workflow; unavailable visual checks are evidence limits within that workflow rather than a reason to manufacture a standing validation ticket.

Preserve the existing generated gm-tickets ownership, canonical renderer alignment and strict checker requirements. This is guidance only; do not change the dashboard renderer or checker.

## Acceptance criteria

- [x] AGENTS.md assigns routine docs/status.html generation, updates and verification scope to the delivery-dashboard skill without a separate standing validation ticket
- [x] Existing dashboard snapshot and strict checker requirements remain unchanged

## Resolution (2026-11-08)

Clarified that routine dashboard work follows the delivery-dashboard skill without creating a standing validation ticket. No scratch artifacts retained for this guidance-only change.

### Verification
1. ✓ AGENTS.md assigns routine docs/status.html generation, updates and verification scope to the delivery-dashboard skill without a separate standing validation ticket — "AGENTS.md lines 15-18 now assign routine generation, updates and verification scope to delivery-dashboard and keep unavailable visual checks as in-workflow evidence limits; git diff --check passed."
2. ✓ Existing dashboard snapshot and strict checker requirements remain unchanged — "The published guidance diff retains snapshot-tickets.mjs ownership, tag-context refresh, delivery-dashboard checker and canonical renderer requirements verbatim; docs/status.html and checker code were unchanged."

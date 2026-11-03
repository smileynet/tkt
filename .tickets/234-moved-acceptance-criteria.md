---
id: "234"
title: "Acceptance criteria can be marked moved to another ticket"
status: done
blocked_by: []
priority: medium
validation_criteria:
  - "Moved criteria are distinct from done, resolve correctly in close/validate/audit, and require an existing destination"
  - "The receiving criterion has a source back-reference and source/destination evidence mapping is explicit"
  - "Retries are idempotent and interrupted multi-file transfers have tested recovery without losing or duplicating criteria"
tags: ["cli", "kind:enhancement", "stream:workflow", "triage:ready-for-agent"]
assignee: "codex"
---

# Acceptance criteria can be marked moved to another ticket

## Owner decision — 2026-11-03

Confirmed in chat: **Use an explicit moved marker and linked transfer**. This supersedes conflicting
requirements and the earlier dated decision gate; it does not claim implementation.
See [the authoritative record](../docs/decision-record-2026-11-03.md).

## Origin

Field report from philippines-trip, 2026-11-01 (prioritization review, location planning and a Codex worktree takeover). Sam asked for these to be filed.

## What happened

Sam moved one unticked criterion from philippines-trip #78 to #99 and closed #78. The only way to close was to tick the criterion, which reads as if it were done.

## Acceptance criteria

- [x] Moved criteria are distinct from done, resolve correctly in close/validate/audit, and require an existing destination
- [x] The receiving criterion has a source back-reference and source/destination evidence mapping is explicit
- [x] Retries are idempotent and interrupted multi-file transfers have tested recovery without losing or duplicating criteria

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:needs-info`; stream `cli`.

Moved-AC intent is current; decide marker grammar, multiple source/destination criteria, evidence mapping, idempotence and atomic multi-ticket recovery before implementation. Do not mark moved work done.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

## Planning gate

Resolved by the owner's 2026-11-03 chat submission: **Use an explicit moved marker and linked transfer**.
See [the decision record](../docs/decision-record-2026-11-03.md). Implementation remains open; the owner-policy gate is cleared.

## What to build

Represent transferred criteria with a distinct moved marker and destination link. Add the receiving criterion with a source back-reference. Close, validate and audit recognize transfer without claiming implementation. Check the destination and evidence mapping, define deterministic retry identity, and prove recovery across interrupted multi-file writes. Final marker/flag spelling is an implementation design detail to document and test, not another pending owner-policy decision.

## Resolution (2026-11-03)

Added move-ac linked transfers, distinct moved markers and strict link validation without claiming implementation. Independent critic accepted final code/test identities; evidence promoted to docs/evidence/ac-transfer-234.md. Disposable logs/helpers released to trash; shared target retained for ongoing CLI work.

### Verification
1. ✓ Moved criteria are distinct from done, resolve correctly in close/validate/audit, and require an existing destination — "Explicit moved state and symmetric links checked by validate/audit/close; eight integration cases; docs/evidence/ac-transfer-234.md"
2. ✓ The receiving criterion has a source back-reference and source/destination evidence mapping is explicit — "Receiving back-reference and unchanged independent VC/evidence scope documented and tested; critic accepted final hashes"
3. ✓ Retries are idempotent and interrupted multi-file transfers have tested recovery without losing or duplicating criteria — "Destination-first recovery, repeat no-op, remote push and no-push verified; final mise run check passes 395 Rust tests plus 42 Node fixtures, one existing skip"

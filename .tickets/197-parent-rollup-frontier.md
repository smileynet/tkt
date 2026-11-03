---
id: "197"
title: "Show computed parent progress with explicit closure"
status: done
blocked_by: ["192", "196"]
priority: high
validation_criteria:
  - "Child progress is reported without changing the stored parent status or bypassing close policy"
  - "Parent containers are excluded from the frontier and child views report done/total plus distinct cancelled counts"
  - "Integration tests cover mixed child states, explicit parent closure and unchanged priority/ID ordering"
tags: ["schema", "kind:enhancement", "stream:relations", "triage:ready-for-agent"]
assignee: "codex"
---

# Show computed parent progress with explicit closure

## Owner decision — 2026-11-03

Confirmed in chat: **Show computed progress; close parents explicitly**. This supersedes conflicting
requirements and the earlier dated decision gate; it does not claim implementation.
See [the authoritative record](../docs/decision-record-2026-11-03.md).

## What to build

Parent progress remains computed and separate from stored lifecycle status. A parent is never auto-closed by child status; explicit close, AC and evidence policy still apply. Show done/total child counts and a separate cancelled count without treating cancellation as done. Retain the container/frontier and child-view scope, without changing priority/ID ordering.

## Acceptance criteria

- [x] Child progress is reported without changing the stored parent status or bypassing close policy
- [x] Parent containers are excluded from the frontier and child views report done/total plus distinct cancelled counts
- [x] Integration tests cover mixed child states, explicit parent closure and unchanged priority/ID ordering

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:needs-info`; stream `schema`.

Automatic done contradicts the close-only evidence contract. Decide computed progress versus stored status and cancellation semantics before implementation; do not auto-close parent files.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

## Planning gate

Resolved by the owner's 2026-11-03 chat submission: **Show computed progress; close parents explicitly**.
See [the decision record](../docs/decision-record-2026-11-03.md). Implementation remains open; the owner-policy gate is cleared.

## Resolution (2026-11-03)

Added direct-child progress with distinct cancellation counts and container frontier exclusion. Explicit parent close policy preserved. Independent critic accepted final code/test hashes; evidence promoted to docs/evidence/parent-progress-197.md. Disposable helper/logs released to trash; shared target cache retained for ongoing CLI tickets.

### Verification
1. ✓ Child progress is reported without changing the stored parent status or bypassing close policy — "Integration parent_cli: mixed child states and no implicit status changes; docs/evidence/parent-progress-197.md"
2. ✓ Parent containers are excluded from the frontier and child views report done/total plus distinct cancelled counts — "Full-corpus children view and central frontier exclusion verified by integration tests; independent critic accepted"
3. ✓ Integration tests cover mixed child states, explicit parent closure and unchanged priority/ID ordering — "Final mise run check passed fmt, clippy and all Rust suites; priority plus ID tie-break and explicit close tested"

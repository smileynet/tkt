---
id: "239"
title: "Review historical closure warnings without fabricating evidence"
status: open
blocked_by: []
priority: low
validation_criteria:
  - "Every historical closure warning has a documented evidence-backed disposition; unavailable original proof remains explicitly unknown"
tags: ["quality", "kind:enhancement", "triage:ready-for-agent", "stream:quality"]
---

# Review historical closure warnings without fabricating evidence

## What to build

Disposition the historical warnings captured by the 2026-11-02 board audit. Separate missing original proof from template noise and preserve historical truth. This is follow-up quality work, not permission to reopen completed product behavior or invent successful checks.

## Context

Intent source: owner-requested full-board triage, 2026-11-02. Read docs/agents/ticket-triage-2026-11-02.md, src/audit.rs, and docs/adr/0003-audit-debt-grandfather-cutoff.md. Missing evidence: #154, #155, #156, #174, #206. Template warnings: #48, #80, #81, #82, #84, #91, #92, #95, #96, #97, #102, #129, #149, #152, #174.

## Acceptance criteria

- [ ] Reproduce the audit output and classify each warning confirmed, false positive, or obsolete with file evidence.
- [ ] Document unavailable original evidence honestly; distinguish any new check from the historical closure event.
- [ ] Correct only demonstrably stale placeholder text or audit false positives; do not fabricate or silently waive proof.
- [ ] Run applicable checks and record the remaining historical limitations.

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:ready-for-agent`; stream `quality`.

Audit --deep reports 20 historical warnings: five missing-evidence closures and fifteen template closures. Review and disposition without invented historical results; unrelated informational high-priority notices are not failures.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

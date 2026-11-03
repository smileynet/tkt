---
id: "195"
title: "parent validation: dangling + cycle checks"
status: done
blocked_by: ["192"]
priority: high
validation_criteria:
  - "check_dangling_parent flags a parent id with no matching ticket"
  - "check_parent_cycles detects a parent chain cycle; a ticket cannot be its own parent"
  - "Both checks wired into collect_findings and covered by tests mirroring check_cycles"
tags: ["schema", "kind:bug", "triage:ready-for-agent", "stream:relations"]
---

# parent validation: dangling + cycle checks

## What to build

Validate the `parent` field (ticket 192) with the same machinery that already
checks `blocked_by`.

- `src/findings.rs`: `check_dangling_parent` (a `parent` id with no matching
  ticket, mirrors `check_dangling_deps`) and `check_parent_cycles` (the parent
  chain is a DAG, mirrors `check_cycles`); a ticket cannot be its own parent.
- `src/core/validate.rs`: a `validate_parent_ref` helper (mirrors
  `validate_no_self_dep`).
- `src/commands/validate.rs`: wire both into `collect_findings`.

## Acceptance criteria

- [x] `check_dangling_parent` flags a `parent` id with no matching ticket
- [x] `check_parent_cycles` detects a parent-chain cycle; a ticket cannot be its own parent
- [x] Both checks wired into `collect_findings` and covered by tests mirroring `check_cycles`

## Triage — 2026-11-02

Disposition: `kind:bug`, `triage:ready-for-agent`; stream `schema`.

Confirmed: parent: "999" on the sole valid ticket passes validate with zero findings. Typed parent storage exists from #192; add dangling/self/cycle findings in src/findings.rs.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

## Resolution (2026-11-03)

Parent integrity checks wired into findings; independent critic accepted. docs/evidence/parent-label-config-195-193-236.md; fixture scratch self-clean, shared target retained for queue.

### Verification
1. ✓ check_dangling_parent flags a parent id with no matching ticket — "Public validate_flags_dangling_parent passes."
2. ✓ check_parent_cycles detects a parent chain cycle; a ticket cannot be its own parent — "Public self-parent and parent-chain cycle fixtures pass."
3. ✓ Both checks wired into collect_findings and covered by tests mirroring check_cycles — "Five parent fixtures pass including independent blocked_by graphs; full gate332Rust42Nodepass one existing skip."

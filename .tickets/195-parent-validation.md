---
id: "195"
title: "parent validation: dangling + cycle checks"
status: backlog
blocked_by: ["192"]
priority: high
validation_criteria:
  - "check_dangling_parent flags a parent id with no matching ticket"
  - "check_parent_cycles detects a parent chain cycle; a ticket cannot be its own parent"
  - "Both checks wired into collect_findings and covered by tests mirroring check_cycles"
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

- [ ] `check_dangling_parent` flags a `parent` id with no matching ticket
- [ ] `check_parent_cycles` detects a parent-chain cycle; a ticket cannot be its own parent
- [ ] Both checks wired into `collect_findings` and covered by tests mirroring `check_cycles`

---
id: "248"
title: "Compare plan lifecycle labels precisely without narrative parsing"
status: in_progress
blocked_by: []
priority: high
validation_criteria:
  - "Integration fixtures verify exact lifecycle drift and compatible fix/advisory behavior"
  - "Independent critic accepts final code and full local gate passes"
tags: ["kind:bug", "triage:ready-for-agent", "stream:quality"]
assignee: "codex"
---
## Intent source

Observed during the optional consumer enforcement probe for ticket 159 on 2026-11-03. Owner decision for ticket 184 keeps narrative parsing deferred and explicitly separates any reproduced PlanState correctness repair.

## What to build

Fix current three-column plan lifecycle comparisons without activating narrative parsing. An open ticket and plan row `| 01 | Policy fixture | done |` currently produce `pass (0 finding(s))` under `sync-plan --check --strict --brief`; source `src/commands/sync_plan.rs` tests only the presence of the done emoji/cancelled prefix, so nonterminal lifecycle mismatches are also missed. Compare recognized lifecycle labels precisely, preserve the canonical done display and cancellation reason detail, and make fix regenerate the correct cell. Define/report unrecognized cells rather than silently treating them as open. Preserve the advisory plain sync-plan versus explicit check/strict behavior and user-owned plan prose.

## Acceptance criteria

- [ ] Integration fixtures reproduce and repair plain-done and open/in-progress/backlog mismatches with correct check versus advisory exits.
- [ ] Canonical done, cancellation detail and unchanged matching rows remain compatible; fix/dry-run and unknown-cell handling are tested.
- [ ] Independent critic accepts the final code/guidance and full local gate passes without adding narrative parsing.

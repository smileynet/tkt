---
id: "184"
title: "Implement sync-plan narrative PLAN.md support (PlanFormat/PlanState) per ADR 0002"
status: open
blocked_by: []
priority: medium
validation_criteria:
  - "sync-plan --check passes against a prose/phased PLAN.md with sigil-anchored IDs (- [ ] #NN ...), via AST + anchored-ID extraction; table docs/plan.md path still works (tests/integration.rs:312-430 green)"
  - "Done predicate returns a PlanState sum type (Done|NotDone|Unknown); TKT_ASCII=1 plan reads done correctly and an unparseable cell surfaces as Unknown not silent not-done (defect 1, with regression test)"
  - "Dep-budget sub-decision recorded (CommonMark crate vs hand-rolled heading+list-item scanner), lean scanner benchmarked first per ADR 0002"
  - "New format/flag + default path documented in --help and all guidance surfaces per .memory/agent-guidance-surfaces.md"
---

# Implement sync-plan narrative PLAN.md support (PlanFormat/PlanState) per ADR 0002

## What to build

TBD

## Acceptance criteria

- [ ] TBD

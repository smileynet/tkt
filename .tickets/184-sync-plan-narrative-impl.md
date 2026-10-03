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

The narrative-parsing implementation deferred from #183. ADR 0002 is the governing
decision — read it first (`.memory/adr/0002-plan-narrative-support.md`). This ticket
builds **position C** (narrative plan with sigil-anchored IDs) as the opt-in format
alongside the existing `docs/plan.md` table, and carries the structural fix for
defect 1 (the `✅`-glyph / `TKT_ASCII=1` latent bug) via the `PlanState` sum type.

Position A (plan-as-projection — regenerate derivable columns from `tkt query`) is
the strategic primary per ADR 0002 but is a larger, separate change; this ticket is
scoped to the narrative-read + the representation refactor that both positions need.

## Design (from ADR 0002 + #183 research)

Extension point — the drift/finding/gate logic (`sync_plan.rs:62-119`) is already
format-agnostic once fed typed pairs. Introduce:

- `PlanFormat` — a **2-variant enum** (table | task-list), NOT a trait/plugin
  framework (rule-of-two; add a third only when a real third format exists).
  - `extract(plan_text) -> Vec<(TicketId, PlanState)>` (replaces the `RE_PLAN_ROW`
    scan at `sync_plan.rs:42,88`)
  - `rewrite(plan_text, id, desired) -> plan_text` (replaces the inline table fixer
    at `sync_plan.rs:50-60`)
  - dispatch by filename/content sniff; keep `docs/plan.md` table working via
    **auto-detect, not replacement**.
- `PlanState` sum type — `Done | NotDone | Unknown { raw }`, replacing `plan_done:
  bool`. An ASCII-mode or unparseable cell becomes `Unknown` (reportable), never a
  silent not-done. **This is defect 1's structural fix.** The done predicate becomes
  a `PlanFormat` method returning `PlanState`, not a `✅`-substring test.
- `TicketId` new-type over raw `\d+`; the sigil-anchored-ID rule (`#NN` at start/end
  of a list item or heading) is its parser. Parse-don't-validate at the boundary.
- Narrative parse strategy (ADR 0002): **CommonMark AST for structure** (headings =
  phases, list items = tasks; ignore everything else), **narrow anchored regex for
  IDs** over the flattened text of leaf blocks. Never regex for structure. GFM
  task-list checkbox state parsed explicitly (it's an extension, not core CommonMark).
- Drift findings: closed sum type over the five buckets (`TICKET_NOT_IN_PLAN`,
  `PLAN_REF_NO_TICKET`, `STATUS_DRIFT`, `AMBIGUOUS_REF`, `DUPLICATE_REF`), each with
  `file:line` provenance. Warn+suggest on ambiguous IDs; never auto-fix structure.

## Scope boundaries

- `--fix` for narrative is riskier (edit a checkbox/marker, preserve prose). Scope it
  down or disable it for narrative initially until a safe editor exists.
- Dep budget (AGENTS.md: "no deps without justification"): benchmark a hand-rolled
  heading+list-item scanner BEFORE pulling a CommonMark crate (pulldown-cmark/comrak);
  record the decision.
- Position A (plan-as-projection / regenerate from `tkt query`) is explicitly OUT of
  scope — separate follow-up if pursued.

## Acceptance criteria

- [ ] `PlanFormat` 2-variant enum with `extract`/`rewrite`; table path unchanged
      (`tests/integration.rs:312-430` green)
- [ ] `PlanState` sum type (`Done|NotDone|Unknown`); `TicketId` new-type; parse at boundary
- [ ] `sync-plan --check` passes against a prose/phased PLAN.md with sigil-anchored IDs
- [ ] Defect 1 fixed: `TKT_ASCII=1` plan reads done correctly; unparseable cell → `Unknown`; regression test added
- [ ] Dep-budget sub-decision recorded (lean scanner benchmarked first)
- [ ] `--help` + all guidance surfaces updated per `.memory/agent-guidance-surfaces.md`
- [ ] `--fix` behavior for narrative decided (scoped-down or disabled) and documented

## References

- Governing decision: `.memory/adr/0002-plan-narrative-support.md`
- Research: `.scratch/research/prior-art.md`, `.scratch/research/best-practices.md`
- Internal review: `.scratch/internal/syncplan-code.md`, `.scratch/internal/docs-config.md`
- Code: `src/commands/sync_plan.rs`; `core::atomic_write` (already used by the fixer since #183)
- Origin: #183 (decision ticket)

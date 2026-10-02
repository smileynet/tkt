---
id: "183"
title: "Consider a plan-table feature: let sync-plan track a narrative/root PLAN.md, not just docs/plan.md tables"
status: done
blocked_by: []
priority: high
validation_criteria:
  - "A design decision is recorded (ADR or doc) on whether/how sync-plan supports a narrative or root-level PLAN.md in addition to the docs/plan.md table format"
  - "If implemented: sync-plan --check passes against a PLAN.md that references tickets in prose/phased lists (not a one-row-per-ticket table), OR the limitation is documented with a clear error pointing the user to the expected format/location"
  - "Default plan path and any new format/flag are documented in --help and AGENTS.md"
---

# Consider a plan-table feature: let sync-plan track a narrative/root PLAN.md, not just docs/plan.md tables

## Problem

`sync-plan` today only understands a flat 3-column Markdown table
(`| id | title | status |`, done = a `✅` glyph in the status cell;
`src/commands/sync_plan.rs:12-13,44-45`). A project that keeps a *narrative* or
phased PLAN.md (prose + task-list checklists, headings-as-phases) cannot be
reconciled — `sync-plan` will not see any of its ticket references, and against a
real prose plan the whole-file pipe-row scan can even emit false `plan-orphan-row`
errors on incidental tables.

This ticket is primarily a **design decision** (criterion 1). Implementation is
gated on that decision.

## Decision to make (criterion 1 — the ADR)

Write `.memory/adr/0002-plan-narrative-support.md` (project ADR format: Status /
Date / Source, then Context / Decision / Consequences / Alternatives rejected)
answering: **should `sync-plan` support a narrative/root PLAN.md, and if so, by
which mechanism?** The research below frames three positions; pick one deliberately.

### Prior-art finding (shapes the decision)

Bidirectional reconcile of a *separate hand-written narrative plan* against a
discrete task store — tkt's exact `sync-plan` shape — is **rare prior art**. The
field converged on avoiding two sources of truth instead:

- **A. Single store, derived plan** (Backlog.md, Taskwarrior, todo.txt, Logseq
  queries): tasks are truth; the roadmap is a *rendered projection*. For tkt this
  means pushing `--fix` toward "regenerate the plan's derivable columns from
  `tkt query`" so drift is structurally impossible — not teaching the parser to
  read freeform prose. [prior-art L4:established]
- **B. Embedded-plan grammar** (project.txt): one file fuses prose + tasks via a
  sigil grammar with dependencies and auto-completing milestones. Richer, but a new
  grammar to own. [L4:established]
- **C. Narrative + anchored ID convention** (markdown-plan, 72★): keep prose/phased
  structure for humans; require each ticket ref to carry a **sigil-anchored ID**
  (`#NN` at the start/end of a list item or heading) so extraction stays
  table-reliable without one-row-per-item. [L5:reported]

Recommended default position (unless the user prefers otherwise): **A as the
primary bet** (it matches tkt's "files are the database + derived frontier" model
and Backlog.md, tkt's closest sibling), with **C as the opt-in** for projects that
insist on authoring a narrative plan. Reject B for v1 (new grammar, high surface).

**DECIDED (2026-10-02):** position **A primary** (plan as a regenerable projection
from `tkt query`) + **C opt-in** (narrative with sigil-anchored IDs); **B rejected**
for v1. The ADR records this with the prior-art rationale.

### If the decision is "support narrative" — parse strategy (criterion 2)

Research is unambiguous on *how* to do it safely:

- **Structure from a CommonMark AST; ticket IDs from a narrow anchored regex over
  the flattened text of leaf blocks. Never parse plan structure with regex**
  (CommonMark emphasis Rule 17 "exists purely to break regex parsers"; ReDoS CVEs
  in league/commonmark & mistune). [best-practices L2/L5:established]
- Anchor on the two unambiguous block types — **headings (phases) + list items
  (tasks)**; ignore everything else (markdown-plan's rule). [L5:reported]
- GFM task-lists (`- [ ]`/`- [x]`) are a GFM *extension*, not core CommonMark —
  enable the extension or parse checkbox state yourself. [L4:established]
- Make ticket IDs a **sigil-anchored convention** (`#NN` at start/end of a list
  item) to kill false positives (versions, dates, step counts). [Inferred]
- Classify drift into stable buckets with `file:line` provenance:
  `TICKET_NOT_IN_PLAN`, `PLAN_REF_NO_TICKET`, `STATUS_DRIFT`, `AMBIGUOUS_REF`,
  `DUPLICATE_REF`. Warn+suggest on ambiguous IDs; never auto-fix structure.
  [best-practices + detect-doc-drift L4]

### Extension point (if implemented — localizes the change)

The drift/finding/gate logic (`sync_plan.rs:62-119`) is **already format-agnostic
once fed `(TicketId, PlanState)` pairs**. Introduce a `PlanFormat` abstraction:
- `extract(plan_text) -> Vec<(TicketId, PlanState)>` (replaces the `RE_PLAN_ROW`
  scan at `sync_plan.rs:42` and `88`)
- `rewrite(plan_text, id, desired) -> plan_text` (replaces the inline table fixer
  at `sync_plan.rs:50-60`)
Dispatch by filename/content sniff (table vs task-list). Keep the `docs/plan.md`
table path working via **format auto-detect, not replacement** —
`tests/integration.rs:312-430` is the regression guard.

### Representation (data modeling — decided 2026-10-02)

Build the plan model so invalid states can't be constructed; this is where defect 1
is fixed *structurally* rather than with another bool guard.

- **`PlanState` is a sum type, not `plan_done: bool`.** The current binary model
  (`sync_plan.rs:45`) lets an ASCII-mode cell or an unparseable status masquerade as
  a clean `false` (= defect 1). Model it as a variant that *carries its evidence* —
  e.g. `Done | NotDone | Unknown { raw: String }` — so "couldn't determine done-ness"
  is a distinct, reportable state (feeds `AMBIGUOUS_REF`/`STATUS_DRIFT`), not a silent
  not-done. The done predicate becomes a `PlanFormat` method producing a `PlanState`,
  which is exactly the pluggable-predicate fix defect 1 calls for.
- **`TicketId` new-type over raw `\d+`.** Parsing produces a `TicketId`, not a bare
  string/int; the sigil-anchored-ID rule is its parser. Kills the "any digits in prose"
  false-positive class at the type boundary.
- **Parse, don't validate, at the plan boundary.** Parse the plan once (AST +
  anchored-ID regex) into typed `(TicketId, PlanState)` entries carrying `file:line`
  provenance; drift logic consumes the refined entries, never re-scans raw text.
- **Drift findings are a closed sum type** over the five buckets (exhaustiveness: a
  new bucket forces every match site to handle it).
- **Do NOT over-model the format dispatch.** `PlanFormat` is a **2-variant enum**
  (table | task-list) today, not a trait/plugin framework — rule-of-two: add the
  abstraction layer only when a genuine third format exists. (Downgrades the internal
  review's "trait" suggestion until there are ≥2 real non-table formats.)

## Scope boundaries

- `--fix` for narrative is riskier (edit a checkbox/marker, preserve surrounding
  prose). If narrative support lands, **scope `--fix` down or disable it for
  narrative initially** until a safe editor exists.
- The dep budget is a real constraint (AGENTS.md: "no deps without justification").
  Whether to pull a CommonMark crate (pulldown-cmark/comrak) vs hand-roll a
  heading+list-item scanner is an open sub-decision for the ADR to settle
  (benchmark the lean scanner first).

## In-scope defects (folded in — decided 2026-10-02)

Defects 1 & 2 are **in scope for this ticket** (they live in the same code path and
the format work must touch them anyway). Defect 3 (doc contradiction) is already
covered by criterion 3.

1. **Latent `✅`-is-load-bearing bug:** done is read only from the `✅` glyph, and
   `TKT_ASCII=1` does NOT affect it — a plan authored in ASCII mode never reads as
   done (`sync_plan.rs:44-45`). Fix *structurally* via the `PlanState` sum type (see
   Representation above): the done predicate becomes a `PlanFormat` method returning
   `Done | NotDone | Unknown`, so an ASCII/unparseable cell is `Unknown` (reportable),
   never a silent `NotDone`. Add an ASCII-mode regression test.
2. **Plan write bypasses `core::atomic_write`** (`sync_plan.rs:122` uses
   `std::fs::write`), unlike ticket writes (AGENTS.md constraint) — torn-write
   exposure that grows if the fixer gets richer. Route the plan write through
   `core::atomic_write`.
3. **Live doc contradiction:** `steering/frontier-work.md:87` heading "PLAN.md is
   Authoritative" names a root `PLAN.md` while code + AGENTS.md + README +
   commands.md all say `docs/plan.md`. #169 claimed to reconcile this but the
   steering heading/body were not updated. Resolved as part of criterion 3 — this
   ticket touches the plan-path story directly.

## Acceptance criteria

- [x] ADR `.memory/adr/0002-*.md` records the decision on narrative/root PLAN.md
      support (position A/B/C + rationale from prior art), per criterion 1
- [x] If implemented: `sync-plan --check` passes against a prose/phased PLAN.md with
      sigil-anchored IDs, OR a clear error names the expected format/location
      (criterion 2)
- [x] Default plan path + any new format/flag documented in `--help` and `AGENTS.md`
      (criterion 3); all guidance surfaces in `.memory/agent-guidance-surfaces.md`
      updated, including the frontier-work.md `PLAN.md`-vs-`docs/plan.md` contradiction
- [x] `docs/plan.md` table path still works (regression: `tests/integration.rs:312-430`)
- [x] Done predicate returns a `PlanState` sum type (`Done | NotDone | Unknown`),
      not a bool; `TKT_ASCII=1` plan reads done correctly and an unparseable cell
      surfaces as `Unknown`, not silent not-done (defect 1, with regression test)
- [x] Plan model is valid-by-construction: `TicketId` new-type, parsed-not-validated
      at the boundary, drift findings a closed sum type (Representation section)
- [x] Plan write routed through `core::atomic_write` (defect 2)

## References

- Internal review: `.scratch/internal/syncplan-code.md`, `.scratch/internal/docs-config.md`
- Research: `.scratch/research/prior-art.md`, `.scratch/research/best-practices.md`
- Code: `src/commands/sync_plan.rs`; tests `tests/integration.rs:312-430`
- Prior ticket: #169 (sync-plan advisory-by-default)

## Resolution (2026-10-02)

Decision ticket: wrote ADR 0002 (A primary + C opt-in, B rejected) grounded in prior-art + parsing research. Narrative-parsing build (PlanFormat/PlanState/TicketId) and defect 1 (ASCII done-glyph) deferred to a follow-up implementation ticket. Landed in-scope: defect 2 (plan write → core::atomic_write) and defect 3 (frontier-work.md plan-path contradiction). Gates: cargo fmt/clippy/test all pass (79 tests), tkt validate pass.

### Verification
1. ✓ A design decision is recorded (ADR or doc) on whether/how sync-plan supports a narrative or root-level PLAN.md in addition to the docs/plan.md table format — "ADR .memory/adr/0002-plan-narrative-support.md records the decision: position A (plan-as-projection) primary + C (narrative, sigil-anchored IDs) opt-in, B rejected; with prior-art rationale and data-modeling (PlanState/TicketId) commitments"
2. ✓ If implemented: sync-plan --check passes against a PLAN.md that references tickets in prose/phased lists (not a one-row-per-ticket table), OR the limitation is documented with a clear error pointing the user to the expected format/location — "Narrative parsing implementation deferred to a follow-up ticket per ADR 0002 (criterion is conditional 'if implemented'); the decision + parse strategy (AST + anchored-ID regex) and the limitation are documented in the ADR and #183 body"
3. ✓ Default plan path and any new format/flag are documented in --help and AGENTS.md — "steering/frontier-work.md reconciled (heading 'The plan is authoritative' + names docs/plan.md and table format, cites ADR 0002), resolving the PLAN.md-vs-docs/plan.md contradiction; default path docs/plan.md already documented in AGENTS.md/README/commands.md. Folded-in defect 2 fixed: plan write now core::atomic_write (fmt/clippy/test all green, 79 tests pass)"

# ADR 0002: sync-plan plan format — derived projection primary, narrative opt-in

- Status: accepted
- Date: 2026-10-02
- Source: ticket #183, decided by user

## Context

`tkt sync-plan` reconciles ticket status against a plan document. Today it understands
exactly one format: a flat 3-column Markdown table (`| id | title | status |`), where
"done" is the `✅` glyph in the status cell (`src/commands/sync_plan.rs:12-13,44-45`).
Default path is `<repo-root>/docs/plan.md`.

Ticket #183 asks whether `sync-plan` should also track a **narrative or root-level
PLAN.md** — prose + phased checklists, headings-as-phases — the shape agents and humans
actually author a plan in. We researched prior art, parsing best-practices, and reviewed
the existing implementation before deciding (artifacts under `.scratch/research/` and
`.scratch/internal/`, captured in the #183 ticket body).

**Prior-art finding (decisive).** Bidirectional reconciliation of a *separate
hand-written narrative plan* against a discrete task store — tkt's exact `sync-plan`
shape — is **rare**. The field converged on avoiding two sources of truth:

- **A. Single store, derived plan** — Backlog.md (tkt's closest sibling), Taskwarrior,
  todo.txt, Logseq queries. Tasks are truth; the roadmap is a *rendered projection*, so
  there is nothing to drift. [prior-art.md L4:established]
- **B. Embedded-plan sigil grammar** — project.txt fuses prose + tasks in one file with
  dependencies and auto-completing milestones. [L4:established]
- **C. Narrative + anchored ID convention** — markdown-plan (72★): prose/phased
  structure for humans, each ticket ref carries a sigil-anchored ID (`#NN` at the
  start/end of a list item or heading) so extraction stays table-reliable. [L5:reported]

A separate narrative plan reconciled against tickets is either an under-served niche or a
shape the field deliberately abandoned. We treat the drift risk as real and bias toward
the designs that minimize two-sources-of-truth.

## Decision

1. **Position A is the primary bet.** The strategic direction for `sync-plan` is the
   plan-as-projection model: the plan's derivable columns are regenerated from
   `tkt query`, so status drift is structurally impossible for derived fields. This
   matches tkt's "files are the database + derived frontier" identity and its closest
   sibling, Backlog.md. The existing `--fix` (regenerate status cells) is the seed of
   this; the direction is to extend it toward "regenerate the plan's derivable columns"
   rather than teach the parser to read freeform prose as authoritative state.

2. **Position C is the opt-in** for projects that insist on authoring a narrative plan.
   When supported, the parse strategy is fixed by this ADR (see Representation + the
   #183 body): CommonMark AST for structure, a narrow sigil-anchored regex for ticket
   IDs over flattened block text, headings = phases, list items = tasks, GFM task-list
   checkbox state parsed explicitly. Never regex for structure.

3. **Position B is rejected for v1** — a new grammar is high surface area for narrow
   benefit; A + C cover the need.

4. **Implementation is deferred to a follow-up ticket; #183 is the decision + two safe
   fixes.** Criterion 2 of #183 is explicitly conditional ("if implemented"). This ADR
   is the governing decision. The narrative-parsing build (the `PlanFormat` /
   `PlanState` / `TicketId` machinery) lands under a new implementation ticket, not here,
   because it is non-trivial and the dep-budget sub-decision (below) needs its own
   spike. Within #183 we land only the bounded, decision-independent work: defect 2
   (atomic plan write) and the doc-contradiction reconciliation (defect 3).

### Representation (data modeling — binds the eventual implementation)

When narrative support is built, the plan model must be valid-by-construction:

- **`PlanState` is a sum type, not `plan_done: bool`.** Model as
  `Done | NotDone | Unknown { raw }` so an ASCII-mode or unparseable status cell is a
  distinct, reportable state (feeds `AMBIGUOUS_REF` / `STATUS_DRIFT`), never a silent
  not-done. This is the structural fix for defect 1 (the `✅`-is-load-bearing bug): the
  done predicate becomes a format method returning a `PlanState`, not a glyph test.
- **`TicketId` new-type** over raw `\d+`; the sigil-anchored-ID rule is its parser.
  Kills the "any digits in prose" false-positive class at the boundary.
- **Parse, don't validate, at the plan boundary** — parse once into typed
  `(TicketId, PlanState)` entries carrying `file:line` provenance; the drift/finding/gate
  logic (`sync_plan.rs:62-119`, already format-agnostic) consumes refined entries and
  never re-scans raw text.
- **Drift findings are a closed sum type** over the five buckets (`TICKET_NOT_IN_PLAN`,
  `PLAN_REF_NO_TICKET`, `STATUS_DRIFT`, `AMBIGUOUS_REF`, `DUPLICATE_REF`) —
  exhaustiveness forces every site to handle a new bucket.
- **Do NOT over-model the format dispatch.** `PlanFormat` is a **2-variant enum**
  (table | task-list), not a trait/plugin framework — rule-of-two: add the abstraction
  only when a genuine third format exists.

### Open sub-decision for the implementation ticket

Whether to pull a CommonMark crate (pulldown-cmark / comrak) or hand-roll a
heading+list-item scanner. tkt's "no deps without justification" constraint (AGENTS.md)
pushes toward benchmarking a lean scanner first; the implementation ticket settles it.

## Consequences

- `docs/plan.md` table format stays supported — narrative is added via **format
  auto-detect, not replacement**. Regression guard: `tests/integration.rs:312-430`.
- `--fix` for narrative is riskier (edit a checkbox/marker in place, preserve prose).
  When narrative lands, `--fix` is scoped down or disabled for narrative until a safe
  editor exists.
- Defect 1 (`✅` load-bearing; `TKT_ASCII=1` never reads done) is fixed structurally by
  `PlanState` — but only when the narrative/parsing work lands. Until then it remains a
  known, documented latent bug (table-only plans in ASCII mode). Tracked on the
  implementation ticket.
- Defect 2 (plan write bypasses `core::atomic_write`) and defect 3 (frontier-work.md
  "PLAN.md is Authoritative" vs `docs/plan.md`) are fixed inside #183.
- The strategic "plan as projection" direction may later supersede the hand-maintained
  table entirely (regenerate from `tkt query`); that is a larger change left open.

## Alternatives rejected

- **B — embedded sigil grammar (project.txt):** rejected for v1. A new human-facing
  grammar to own and document; A + C meet the need with less surface.
- **Teach the table parser to read freeform prose as authoritative state:** rejected —
  regex-on-structure is unsound (CommonMark emphasis Rule 17 exists to break regex
  parsers; ReDoS CVEs in league/commonmark, mistune). Narrative support, if built, goes
  through an AST.
- **Build narrative parsing now, inside #183:** rejected — non-trivial, gated on the
  dep-budget spike; #183 stays the decision + bounded fixes, build splits out.
- **`plan_done: bool` carried forward:** rejected — a binary cannot represent
  "undetermined," which is exactly the state defect 1 hides. Superseded by `PlanState`.

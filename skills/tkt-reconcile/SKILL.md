---
name: tkt-reconcile
description: "End-of-session ticket + plan pass: make every ticket current, capture all open work as tickets, and re-prioritize/re-sequence the plan to match current priorities and work streams. Use when wrapping up a session, auditing ticket health, after planning, when work feels untracked, or asking 'is everything captured?' / 'update the plan'. Trigger: sync tickets, update the plan, re-prioritize, end of session ticket pass, ticket review, are tickets current, is everything tracked, ticket audit, ticket hygiene, wrap up tickets, tag scope, work stream."
---

# Tkt Reconcile

Review all tickets against session context, then re-organize the plan to match current priorities. Three outcomes: every planned item is tracked, every ticket is current, and the plan's ordering reflects the agreed priorities and work streams.

## Process

### 1. Gather context

Read all open tickets and the current plan (`docs/plan.md`, session decisions, stated priorities). If the session is scoped to one or more **work streams** (tags), note them. Build a mental map of: what's planned, what's tracked, what's stale, which stream each ticket belongs to.

```bash
tkt context          # show the active tag context (if any)
tkt query --tag <stream>   # tickets in one stream
```

### 2. Audit each open ticket

For every open/in_progress ticket, check for real drift:

- **Stale blocked_by** — `tkt ready` resolves dynamically (done blockers are ignored), so this is cosmetic, not functional. Do NOT manually clear `blocked_by` when a dependency closes — the history is useful context. Only flag if a ticket genuinely isn't appearing on `tkt ready` when it should be.
- **Missing blocked_by** — work can't actually start without another ticket, but no edge declared
- **Vague ACs** — criteria use "properly", "correctly", "as expected" instead of testable conditions
- **Dead context** — file paths that moved, ADRs that were superseded, specs that changed
- **Superseded work** — the plan changed and this ticket no longer reflects what's needed (→ close or rewrite)
- **Scope creep** — ticket grew during discussion but wasn't split
- **Missing/wrong tag** — in a multi-stream project, the ticket has no work-stream tag or the wrong one (see step 5)

For each issue: fix in place (edit frontmatter/body), or if the ticket is no longer relevant, close it with a note or move to `status: backlog`.

### 3. Identify untracked work

Scan session context for work that has no ticket:
- Decisions made that require implementation
- Issues discovered during the session
- Follow-up work from completed tickets
- Planned items with no corresponding ticket

For each gap, create a ticket following the quality standard below. **Default new tickets to `open`** (frontier-eligible) — reserve `--status backlog` for work you are deliberately deferring out of this cycle. Tag it with its work stream at creation (step 5).

After creating tickets, add them to the plan — every open ticket needs a plan row so `tkt sync-plan --check` passes clean. Place new rows in the appropriate plan section with correct status, then re-organize per step 6.

### 4. New ticket quality standard

Apply the tkt-core skill's ticket-standards (if the tkt-core skill is loaded, read `references/ticket-standards.md`; otherwise apply the checklist below).

Every new ticket MUST have:

- **Intent source** — link to what spawned it (session decision, plan section, ADR, spec, or prior ticket)
- **Key context** — relevant files, decisions, domain terms the implementer needs
- **Desired outcome** — behavioral "What to build" (what the system does, not how to build it)
- **Validation** — concrete, testable acceptance criteria + `validation_criteria` frontmatter
- **Ordering** — correct `blocked_by` and priority (see step 5)
- **Work-stream tag** — `--tags <stream>` in a multi-stream project (see step 5)

```bash
tkt new <slug> --title "..." [--tags STREAM] [--blocked-by NN,NN] [--priority P]
```

Then fill the body with the full template (What to build, Context, ACs, Out of scope). A ticket that fails the quality bar wastes the implementer's context window.

### 5. Set ordering axes + tag scoping

Three independent axes — don't conflate them:

**Dependencies** (`blocked_by`): structural — A must finish before B can start.
- Only add blocked_by when there's a real technical or logical dependency
- "We want to do A first" is NOT a blocked_by — that's a plan-ordering/priority concern (step 6)

**Priority** (`--priority urgent|high|medium|low`): contextual weight, NOT a hard ordering override.
- Priority adds weight to the frontier; it does not reorder the plan around a late ticket (see step 6 rubric)
- Set via `tkt edit <id> --priority high|medium|low`

**Work-stream tag** (`--tags <stream>`): which parallel thread the ticket belongs to.
- Tag each ticket with its stream (`ink`, `platform`, `config`, …) using the project's established tag vocabulary (`tkt query --tag <name>` to see it)
- One tag per ticket is the norm; add a second only when work genuinely spans streams
- Scope a session to a stream with `tkt context <stream>` so `tkt ready` and new-ticket tagging focus on that thread; clear with `tkt context --clear`
- Omit tags in single-stream projects or for cross-cutting work

```bash
tkt validate --brief   # catches cycles and broken blocked_by refs
```

### 6. Re-organize the plan against current priorities

This is the judgment step the tool can't do: `tkt sync-plan --fix` only reconciles *derivable* columns (status). Re-sequencing and re-grouping is yours.

**Plan-ordering rubric** (apply in order — each rule only breaks ties the one above leaves open):

1. **Finish current threads first.** An in-progress ticket, and the open tickets that continue the same thread/stream, come before starting a new thread. Don't scatter a half-done stream to chase a shinier one.
2. **Prefer earlier (lower-ID) tickets.** Within a thread, and between threads of equal standing, the earlier ticket comes first. Lower ID = earlier commitment = default order.
3. **Blockers override position.** A genuine `blocked_by` (or a hard external dependency) moves a ticket later regardless of its ID — you cannot start what's blocked.
4. **Explicit user instruction overrides all of the above.** If the user has said "do X before Y" this session, that ordering wins; record it as a note so the next session knows it was deliberate, not drift.
5. **Priority is contextual weight, not order.** `high`/`urgent` raises a ticket's standing among otherwise-equal candidates — it breaks ties and signals importance. It does NOT entitle a later, higher-priority ticket to jump ahead of an earlier in-progress thread. Priority informs; position and blockers decide.

Apply the rubric to each plan section:
- **Re-sequence rows** within sections so top-to-bottom reflects the rubric order.
- **Move tickets between release sections** (e.g. v0.5 gate vs. intentionally deferred) when session priorities changed what belongs in this cycle.
- **Flip ticket priorities** (`tkt edit --priority`) so the weight matches the plan — then note *why* in the ticket or plan when a priority changed.
- Keep streams legible: group or label rows by work-stream tag where the plan has parallel threads.

Then reconcile derivable columns:

```bash
tkt sync-plan --fix --brief   # regenerate status cells; never invents ordering
```

### 7. Verify

```bash
tkt sync-plan --check --brief   # plan↔ticket status agreement
tkt validate --brief            # structural health (cycles, refs)
tkt ready                       # frontier matches the rubric's expected first picks
```

Present: tickets created, tickets updated, plan re-organization (what moved and why), current frontier.

## Does NOT

- Orient at the start of a session (that's `tkt-orient`, the session-start counterpart that reads the state this skill leaves behind)
- Pick and execute the next ticket (that's `tkt-work`) — this skill makes the plan's frontier *pickable*, it doesn't pick
- Decompose a spec from scratch into a ticket set (that's the `tkt-core` skill's ticket-creation workflow, or your harness's planning flow)
- Create specs or features
- Rewrite the plan's narrative prose or change its format (status/ordering only)

---
id: "182"
title: "Decide whether tkt needs a delete/cancel operation — and what it should do"
status: backlog
blocked_by: []
priority: medium
validation_criteria:
  - "A decision is recorded (ADR or ticket resolution): delete command, cancelled status, or neither — with rationale grounded in the IDs-as-contracts and files-are-the-database constraints"
  - "If a mechanism is chosen, it is specified (semantics, git/push behavior, effect on blocked_by refs and frontier) before any implementation ticket is opened"
tags: ["cli"]
---

# Decide whether tkt needs a delete/cancel operation — and what it should do

## The gap

`tkt` has no way to retire a ticket that shouldn't be worked and shouldn't be `done`. The status
lifecycle is `backlog → open → in_progress → done`, and `done` is reachable only via `tkt close`
(with its AC/resolution/evidence gates). Today a mistaken, duplicate, or abandoned ticket has no
clean exit: you either leave it in `backlog` forever, hand-edit/`rm` the file (bypassing git-tracked
history and any dep integrity), or force it through `close` with a fake resolution.

This ticket is to **decide the right mechanism**, not to assume a `tkt delete` command is it.

## Why this is a real design question, not an obvious feature

tkt's contract pushes back on naive deletion:

- **IDs are contracts.** `renumber` is birth-window-only precisely because cited IDs are referenced
  by `blocked_by` and by external prose/commits. Deleting ID 42 orphans every `blocked_by: [42]` and
  frees the number for reuse → a future ticket 42 silently inherits stale references. (`tkt validate`
  would need to treat dangling `blocked_by` refs correctly.)
- **Files are the database, git is the log.** A "delete" that just `rm`s the file loses the audit
  trail — but git retains it, so hard delete isn't unrecoverable. Still, the frontier/corpus scan
  reads the working tree, so a removed file simply vanishes from all views.
- **The close gates exist for a reason.** Whatever this mechanism is, it must NOT become a backdoor
  around the evidence-first close workflow (the #178/#179 folklore lesson).

## Options to evaluate

1. **A `cancelled` (or `wontfix`/`obsolete`) status.** Add a terminal state distinct from `done`.
   - Pros: keeps the file + history; preserves the ID as a contract; `blocked_by` refs stay valid
     (and a dependent blocked only by a cancelled ticket could be treated as unblocked or flagged);
     no destructive fs op; symmetric with the existing status model; `tkt query`/`validate` can see it.
   - Cons: expands the status enum (touches frontier logic, validate, sync-plan, every status match);
     needs a rule for "is a ticket blocked by a cancelled ticket ready?"; needs its own command
     (`tkt cancel <id> --reason …`) or an `edit --status cancelled` path (but hand-editing status is
     otherwise forbidden — see #154).
2. **A `tkt delete <id>` command (soft or hard).**
   - *Soft* (move to `.tickets/archive/` or set a `deleted:` marker): recoverable, keeps history,
     but adds a second location the corpus scan must ignore.
   - *Hard* (`git rm` + commit + push, gated): clean removal, git keeps history, but reuses the ID
     and orphans `blocked_by` refs unless guarded. Would need the same push-gated transaction path
     as other mutations (GitTransaction / push.enabled).
   - Pros: matches user mental model ("delete the mistake"); explicit.
   - Cons: ID reuse hazard; dangling-ref handling; a destructive op needs strong guards (birth-window
     only? refuse if any ticket depends on it? `--force`?).
3. **Do nothing — document the workaround.** `--status backlog` for "not now"; for genuine mistakes,
   `git rm` the file manually during the birth window before anyone references the ID. Cheapest, but
   leaves the sharp edge unaddressed.

## Open questions to resolve

- What does a dependent ticket do when its only blocker is cancelled/deleted — auto-unblock, or flag?
- Is deletion birth-window-only (like renumber) or allowed anytime with a dependency guard?
- Does the mechanism reuse an ID (delete) or retire it permanently (cancel keeps the number)?
- How does `tkt validate` treat dangling `blocked_by` after the operation?
- Does this need push-gating (yes, if it mutates + commits — per the mutation-push constraint)?

## Recommendation to start from (not a decision)

Lean toward **option 1 (a `cancelled` status)** — it preserves the ID-as-contract invariant and the
git-tracked file, fits the existing status model, and avoids the ID-reuse/dangling-ref hazards that
make hard delete risky. But this ticket's job is to run the decision (ideally an ADR) with the
tradeoffs above, then spawn a scoped implementation ticket. Parked in `backlog` until prioritized.

## Acceptance criteria

- [ ] A decision is recorded (ADR or this ticket's resolution): `cancelled` status, `delete` command, or neither — with rationale grounded in IDs-as-contracts and files-are-the-database
- [ ] If a mechanism is chosen, its semantics are specified (command/status, git+push behavior, effect on `blocked_by` refs and frontier readiness, `validate` handling) before an implementation ticket opens
- [ ] Whatever is chosen must not become a backdoor around the close gates (evidence-first workflow)

## Notes

Discovered 2026-09-07. Related lifecycle constraints: #154 (forbid hand-editing `status: done`),
renumber birth-window rule (IDs are contracts).

## Waiting on this decision (2026-10-29 scope review)

With `allow_force = false`, these tickets can't be closed honestly today because their
criteria will never be met:

- #208: duplicate, folded into #138.
- #87: `new --template`, merged into #176.
- #115: reviews recall telemetry; recall was retired in the 2026-10-26 cutover.
- #161: changes crew-research's frontier-work; crew-research was retired in the same cutover.
- #94: a test ticket with no content.
- #69: integration exploration, partly delivered (0.5.0 plugins and SessionStart hook) but
  with no recorded decision.

grey-matter ticket 33 (check ACs on a superseded ticket) was migrated here and closed in
grey-matter.

## Proposed decision (2026-10-29, pending owner approval)

**Add a terminal `cancelled` status, reached only through `tkt cancel`. Reject hard delete,
an archive directory and doing nothing.** If approved, record this as ADR 0004 and open
one implementation ticket.

### Command and record

`tkt cancel <id> --reason duplicate|superseded|obsolete|wontfix --note "<why>" [--by <id>]`

- Allowed from `backlog`, `open` or `in_progress`; never from `done`.
- `--by` is required for `duplicate` and `superseded` and must name an existing ticket
  that is not itself cancelled.
- Writes a `## Cancellation` section (reason, by, note, date). It writes no Resolution
  and checks no acceptance criteria, so a cancelled ticket can never pass as done.
- One commit (`chore(tickets): cancel NN`) through the same push-gated transaction as
  `close`. Not birth-window-limited, because nothing is destroyed.
- `tkt reopen <id>` returns it to `open` and keeps the Cancellation section as history.

### Dependents (answers the open questions)

- The frontier rule stays as it is: a blocker must be `done`. Cancelling never silently
  unblocks anything.
- `cancel` refuses while a non-terminal ticket lists it in `blocked_by`, unless the
  caller picks one: `--repoint` rewrites those edges to the `--by` ticket, or
  `--drop-edges` removes them. Each rewritten ticket is named in the output and the commit.
- `validate` reports an error for any `blocked_by` that points at a cancelled ticket, which
  catches hand edits.

### IDs, views and checks

- The file stays and the ID is retired permanently: no reuse, no dangling references.
- `ready` and `blocked` exclude cancelled tickets. `query` includes them with
  `status: cancelled`, and `--status cancelled` filters them.
- `validate` requires a Cancellation section on every cancelled ticket, as `done` requires a
  Resolution (the #154 analogue), and checks that `--by` resolves.
- `audit` exempts cancelled tickets from the closure-quality rules and adds one rule: a
  cancellation needs a reason and a note.
- `sync-plan` shows cancelled rows with their reason; `capabilities` declares the command
  and the status (with #138).
- `close.allow_force` is untouched; existing force-closed "superseded" tickets are not
  rewritten.

### Why not the others

- **Hard delete:** reuses IDs and orphans `blocked_by` and external references. It also
  removes the ticket from every view, and git history is no substitute for the corpus.
- **Archive directory:** adds a second location that every scan must know to skip, and
  still needs a rule for dependents.
- **Do nothing:** six tickets (#208, #87, #115, #161, #94, #69) are stuck today with no
  honest exit, and backlog grows by sediment.

### Consumers to update in the same release

- tkt: `tkt-core` and the `init` snippets document `cancel`.
- grey-matter: the delivery-dashboard snapshot, the checker's status list and a
  collapsed "Cancelled" group in the Plan tab; `tkt-conventions.md` for superseded work.
  Without this, the dashboard checker would reject a snapshot containing the new status.

### First use

Cancel #208 and #87 (duplicates, `--by` #138 and #176), #115 and #161 (obsolete), #94
(wontfix), and decide #69, #140 and #142.

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

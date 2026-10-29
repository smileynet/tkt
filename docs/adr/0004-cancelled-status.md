# ADR 0004: Retire tickets with a terminal `cancelled` status

- Status: accepted
- Date: 2026-10-29
- Source: ticket #182 (decision), approved by user; implementation ticket #227

## Context

The lifecycle is `backlog → open → in_progress → done`, and `done` is reachable only
through `tkt close`, with its acceptance, resolution and evidence gates. This repo sets
`close.allow_force = false`. A duplicate, obsolete or mistaken ticket therefore has no
honest exit: it stays in backlog forever, gets hand-deleted outside the tool, or is forced
through `close` with a resolution that claims work nobody did.

On 2026-10-29 six tickets were stuck this way: #208 (duplicate of #138), #87 (merged into
#176), #115 and #161 (their tools were retired in the 2026-10-26 cutover), #94 (an empty
test ticket) and #69 (partly delivered, no recorded decision).

Two constraints shape any fix:

- **IDs are contracts.** `blocked_by` edges, commits and outside prose cite ticket ids;
  `renumber` is birth-window-only for this reason.
- **Files are the database, git is the log.** Views read the working tree, so a removed
  file vanishes from every view even though git keeps it.

The fix must also never become a way around the close gates.

## Decision

1. **Add a terminal `cancelled` status, reachable only through `tkt cancel`:**
   `tkt cancel <id> --reason duplicate|superseded|obsolete|wontfix --note "<why>" [--by <id>]`.
   - Allowed from `backlog`, `open` or `in_progress`; refused for `done`.
   - `--by` is required for `duplicate` and `superseded` and must name an existing ticket
     that is not cancelled.
   - Writes a `## Cancellation` section (reason, by, note, date). It writes no Resolution
     and checks no acceptance criteria.
   - Commits as `chore(tickets): cancel NN` through the same push-gated transaction as
     `close`. It is not birth-window-limited, because nothing is destroyed.
2. **`tkt reopen <id>` returns a cancelled ticket to `open`.** The Cancellation section
   stays as history.
3. **Dependents are handled explicitly.** The frontier rule is unchanged: a blocker must be
   `done`. `cancel` refuses while a non-terminal ticket lists it in `blocked_by`, unless
   the caller chooses `--repoint` (rewrite those edges to the `--by` ticket) or
   `--drop-edges` (remove them). Every rewritten ticket is named in the output and the
   commit.
4. **The file and ID stay.** A cancelled ID is retired permanently and never reused.
5. **Views and checks:**
   - `ready` and `blocked` exclude cancelled tickets; `query` includes them with
     `status: cancelled` and `--status cancelled` filters them.
   - `validate` errors on a cancelled ticket without a Cancellation section (the #154
     analogue for hand edits), on a `--by` that does not resolve, and on any `blocked_by`
     that points at a cancelled ticket.
   - `audit` exempts cancelled tickets from closure-quality rules and adds one rule: a
     cancellation must carry a reason and a note.
   - `sync-plan` shows cancelled rows with their reason; `capabilities` declares the
     command and the status.
6. **Nothing else changes.** `close.allow_force` keeps its meaning, and tickets already
   force-closed as superseded are not rewritten.

## Consequences

- Duplicate and obsolete work leaves the frontier honestly, and reports can tell
  finished work from abandoned work.
- Every consumer that enumerates statuses must learn the new one in the same release:
  tkt's own skills and `init` snippets, and grey-matter's delivery-dashboard snapshot,
  checker and Plan tab, whose checker otherwise rejects the snapshot.
- The `Status` enum gains a variant, so every exhaustive match changes (21 `Status::Done`
  sites in 8 files on 2026-10-29). The compiler finds the matches; tests must cover the
  behavior of each.
- Once #227 ships, the six stuck tickets above are cancelled with their reasons, and #69,
  #140 and #142 each get a decision.

## Alternatives rejected

- **Hard delete (`git rm`):** frees the ID for reuse, orphans `blocked_by` and outside
  references, and removes the ticket from every view; git history is no substitute for the
  corpus.
- **Soft delete into an archive directory:** creates a second location every scan must
  know to skip, and still needs a rule for dependents.
- **Do nothing and document a workaround:** leaves the stuck tickets with no honest exit
  and lets the backlog grow by sediment.
- **Allow `close --force` for these cases:** would record abandoned work as `done`, which
  is the backdoor the close gates exist to prevent.

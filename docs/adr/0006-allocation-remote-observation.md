# ADR 0006: Allocation observes the remote even when publication is disabled

- Status: accepted
- Date: 2026-11-08
- Source: ticket #281

## Context

With `push.enabled = false` (or `--no-push`), `tkt new` and `tkt batch`
resolved no upstream and scanned only local ticket filenames, so id
allocation was local-only. Two sessions — or one local session and one
remote — could allocate the same id. The collision surfaced only after a
later fetch/rebase, as two `.tickets/NN-*.md` files sharing one id, after
which `tkt close <id>` could match the wrong ticket. Observed 2026-11-08:
a push-disabled session allocated 278 while the remote independently
allocated and pushed its own 278.

Race detection already had three pieces: push-rejection triggers
re-allocation (push-enabled path), `tkt validate` reports `duplicate-id`
files as an error, and `tkt rebase` renumbers local collisions
(upstream wins). None of them observed the remote at allocation time when
push was disabled — even though the remote was reachable and reading it
requires no push. `tkt rebase` already established that a read-only fetch
is not a publication.

## Decision

Collision prevention lives at both ends:

1. **Allocation is remote-aware regardless of publication.** Upstream
   resolution and fetch are read-only observation and happen whenever a
   remote exists; `push.enabled = false` and `--no-push` suppress only
   the publish step. If the remote cannot be observed (fetch or upstream
   resolution fails) while publication is disabled, allocation falls
   back to local state **with a loud warning** — local-only work stays
   possible offline, but the degraded collision guarantee is visible.
   With publication enabled, an unobservable remote stays fatal: pushing
   requires fresh remote state.
2. **Validate-time detection stays** the safety net: `duplicate-id` is an
   error finding, and mutations (`close`, `claim`, `cancel`, `edit`,
   `note`, `move-ac`, `reopen`) refuse an id that matches more than one
   ticket instead of silently targeting the first file.

## Consequences

Id allocation in push-disabled mode performs network I/O when a remote
exists; a previously silent local-only path now reports when it cannot
see the remote. Two sessions that allocate at the same instant can still
pick the same id — no local lock spans checkouts, and an unpushed local
ticket is invisible to other sessions by definition. That residual is
bounded by the existing defenses: the push-enabled session re-allocates
on push rejection, `tkt validate` flags persisted duplicates as errors,
and `tkt rebase` recovers them. `MutationContext` (existing-ticket
mutations) keeps its current contract: it observes the remote only when
publication is enabled.

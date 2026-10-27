---
id: "223"
title: "close/claim leave docs/plan.md behind, so sync-plan --check fails after every close"
status: open
blocked_by: []
validation_criteria:
  - "Each observed point marked confirmed / rejected / re-scoped with evidence"
  - "After tkt close <id> in a repo whose docs/plan.md has a row for <id>, tkt sync-plan --check exits 0 with no further command (integration test)"
  - "After tkt claim <id> (and tkt edit --status), sync-plan --check reports no plan-status-drift for <id> (integration test)"
  - "A repo with no docs/plan.md is unaffected: close/claim write no plan file and their commits are unchanged (integration test)"
  - "The behaviour, and any opt-out, is documented in --help, AGENTS.md and the tkt-core skill"
tags: ["cli"]
---

# close/claim leave docs/plan.md behind, so sync-plan --check fails after every close

> **Filed from a downstream adopter (jumpdeck, tkt 0.4.0), 2026-10-27. Confirmation status: unconfirmed.**
> Reproduce each point and mark it confirmed / rejected / re-scoped before changing anything.

## Observed

1. jumpdeck's `mise run check` runs `tkt sync-plan --check` as its gate (#169 made `--check` the CI
   gate). Twice in one session (`tkt close 76`, then `tkt close 78`), the close commit was pushed
   and the very next check failed:
   `{"status":"fail","findings":[{"file":"76-intro-in-export.md","rule":"plan-status-drift","message":"plan says not done, ticket is done (run: tkt sync-plan --fix)","severity":"warning"}]}`.
   Each time it took a separate `tkt sync-plan --fix` plus a hand-made "chore(plan): sync" commit
   and push.
2. `tkt claim 78` on a freshly created ticket gave `missing-plan-row` ("in_progress ticket has no
   plan row") and `--check` exited 1; `--fix` reported `"fixed":1` but did not add the row, so
   the row had to be written by hand.
3. Source (at 723589c): `src/commands/close.rs`, `claim.rs` and `new.rs` never mention the plan;
   only `sync_plan.rs` reads or writes `docs/plan.md`.

So every state change tkt makes atomically (pull, commit, push) leaves a derivable plan cell
stale, and the derivable drift #169 meant to make cosmetic still blocks any repo that gates on
`--check`.

## What to build

Suggestion only, to be decided:

- When `docs/plan.md` (or the configured plan path) has a row for the ticket, `close`, `claim`
  and status-changing `edit` rewrite that row's status cell in the same commit (the same
  regeneration `sync-plan --fix` already does), so the change and the plan land together.
- Decide whether `new` (and `claim` with no row) appends a row, or whether `sync-plan --check`
  stops treating a missing row for an open/in-progress ticket as a failure. Today `--fix` does
  neither, which is point 2.
- Alternative if writing the plan from close is unwanted: `--check` passes derivable status drift
  (warns) and fails only on non-derivable conflicts, as #169's item 3 proposed.
- A config opt-out (`[close] update_plan = false`) if any adopter needs the old behaviour.

## Acceptance criteria

- [ ] Each observed point marked confirmed / rejected / re-scoped
- [ ] close keeps the plan row current in its own commit (or --check stops failing on derivable drift)
- [ ] claim / status edits likewise; missing-row behaviour decided and consistent with --fix
- [ ] No plan file means no change in behaviour
- [ ] Documented in --help, AGENTS.md and tkt-core

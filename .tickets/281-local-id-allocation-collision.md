---
id: "281"
title: "Ticket id collision when push.enabled=false: allocation is local-only"
status: done
blocked_by: []
validation_criteria:
  - "decision recorded on where collision prevention lives"
  - "concurrent allocation cannot persist duplicate ids, or fails loudly before push"
  - "tkt validate reports duplicate-id ticket files as an error"
tags: ["type:chore", "kind:bug"]
---
## What to build

Investigate and fix the ticket-ID collision path when `push.enabled = false`:
`tkt new` allocates the next id from local state only, so two sessions with
unpushed local tickets (or one local + one remote) can allocate the same id.
The collision surfaces only at push/rebase, leaving two `.tickets/NN-*.md`
files with the same id — after which `tkt close <id>` and friends can target
the wrong ticket's validation criteria.

**Observed 2026-11-08 (tkt repo):** session A pushed `278-dashboard-routine-skill-workflow`;
session B (push.enabled=false) allocated 278 locally for `local-install-script`,
pushed after rebase, and `close 278` matched the *other* ticket's 2 validation
criteria. Fixed forward by renumbering to 280.

**Questions to settle:** should allocation fetch-remote-first even when push is
disabled (read-only fetch is not a push)? Should `validate` or `new` detect
same-id files and fail loudly? Should push rejection due to non-fast-forward
trigger re-allocation instead of pushing a duplicate?

## Acceptance criteria

- [x] Decision recorded on where collision prevention lives (allocation-time fetch, validate-time detection, or both)
- [x] Two sessions allocating concurrently cannot persist two files with the same id (or the second fails loudly before push)
- [x] `tkt validate` reports duplicate-id ticket files as an error

## Resolution (2026-11-09)

Decision recorded in docs/adr/0006-allocation-remote-observation.md: collision prevention lives at both ends. (1) Allocation (new/batch) now resolves the upstream and fetches even under --no-push / push.enabled=false — observation is read-only, only publication is suppressed — so allocated ids skip remote tickets; when publication is disabled and the remote is unobservable, allocation falls back to local state with a loud warning (JSON notice envelope under -o json), and stays fatal when publishing. (2) validate duplicate-id detection retained and now evidenced by a regression test (predates this ticket, from #16/#150). (3) Push-rejection re-allocation retained. Added: mutations (claim/close/cancel/edit/note/move-ac/reopen) refuse an id shared by multiple files instead of silently targeting the first match — the close-278 mis-target from the incident. Known limit (ADR): two sessions allocating at the same instant can still pick one id; bounded by push-rejection renumber, validate duplicate-id error and tkt rebase recovery. Evidence: mise run check green post-rebase (533 passed, 0 failed, fmt/clippy clean). Two-axis code review run; all findings fixed in-scope (cli --no-push help text, find_ticket single pass, JSON notice envelope). Guidance surfaces synced: cli help, 5 init snippets, commands.md, tkt-contract.md, README; tkt-work kept under its 24 KB body limit with a corrected one-word claim. Deployed: cargo install (binary hash 501a9ee == HEAD) + tools/deploy.mjs (4 skills). Complements the config stopgap from ticket 284.

### Verification
1. ✓ decision recorded on where collision prevention lives — "docs/adr/0006-allocation-remote-observation.md records the decision: allocation-time remote observation + validate-time detection (both)"
2. ✓ concurrent allocation cannot persist duplicate ids, or fails loudly before push — "tests/integration.rs: test_new_with_push_disabled_allocates_remote_aware_ids (asserts created 04 skipping remote 02/03 without pulling), test_new_with_push_disabled_tolerates_unreachable_remote (loud local-only fallback), pre-existing test_competing_allocation_no_collision (push-enabled race); residual simultaneity limit recorded in ADR 0006"
3. ✓ tkt validate reports duplicate-id ticket files as an error — "tests/integration.rs: test_validate_reports_duplicate_id_files_as_error (exit 1, rule duplicate-id, both files named; check_duplicate_ids in src/findings.rs)"

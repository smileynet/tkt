---
id: "152"
title: "Review 2026-08-27: fmt gate red at HEAD, indented-comment corruption, #149 dispositions pending"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "cargo fmt --check passes at HEAD on current stable toolchain (or toolchain pinned)"
  - "indented comment lines in frontmatter do not corrupt preceding field values"
  - "each finding dispositioned: fixed, wontfix with rationale, or deferred to existing ticket"
  - "tickets 133-146 backfilled with #128 finding references, no TBD bodies on release blockers"
tags: ["review"]
---

# Review 2026-08-27: fmt gate red at HEAD, indented-comment corruption, #149 dispositions pending

## What to build

A project review (2026-08-27) checked the working tree against intent
(AGENTS.md contract, README hand-edit promise) and the open ticket set.
#132 was verified fixed and closed during the review; its two promised
follow-ups were filed as #150/#151. The findings below remain and need
disposition: fix directly, defer to an existing ticket (annotate), or
wontfix with rationale.

### Review provenance

- Reviewer: opencode session, 2026-08-27, target HEAD `cfbfcb9` (post-#132 close, post-#150/#151 filing)
- Method: cross-check contracts (AGENTS.md, README.md:183 hand-edit promise) vs open tickets vs current source; full gate run (clippy clean, 185/185 tests pass, fmt FAILED — see F1)
- This ticket supersedes the remaining #149 items; close #149 once its findings are dispositioned here or annotated there

### F1 — high: fmt gate is red at HEAD (toolchain drift)

rustfmt 1.8.0-stable wants `use anyhow::{bail, Context, Result};` at
src/core/ticket.rs:4 (case-insensitive brace-sort); the committed
`{Context, Result, bail}` was canonical under older rustfmt. CI is disabled
(.github/workflows/ci.yml:3) so the local gate is the only check — and it
fails. Separately, the `cargo-fmt.exe` shim on this machine is broken
(`%1 is not a valid Win32 application`, same class as the mise shim gotcha);
rustfmt works via direct path.
Fix: one-shot `cargo fmt` commit; consider pinning the rust toolchain in
mise.toml (currently pins only cargo-release/git-cliff) and noting the
direct rustfmt path in AGENTS.md.

### F2 — medium: indented comments corrupt the preceding field (#132 residual)

src/core/ticket.rs:205-213: the continuation branch runs before the comment
branch, so `  # note` after `title: "X"` merges into the raw value — the
typed title reads back as `X"\n  # note` (stray quote + comment text).
#132's unit test `parse_tolerates_comment_lines` does not assert title,
hiding this. YAML semantics: comments are non-data at any indentation.
Fix: in the continuation branch, skip lines whose `trim_start()` starts with
`#` (block-list `- "01"` lines are unaffected); add a test asserting the
preceding field value is clean, and one for indented comments inside a
block-style `blocked_by`.

### F3 — high: #149 F-R1 still open — release blockers have TBD bodies

Tickets 133-145 all still contain `What to build: TBD` / `- [ ] TBD` and
none reference #128, where evidence and fix sketches live (verified:
`rg -l "#128" .tickets` matches only #149). #128 is done, so a fresh agent
working any release blocker gets only a title. This is the critical path for
a healthy v0.3.1.
Fix: backfill 133-145 (and 146) with the #128 finding reference and real
acceptance criteria.

### F4 — low: lint inline-array branch keeps empty items (#149 F-R3)

src/commands/lint.rs:157-163 maps without filtering, so `blocked_by: ["01", ]`
normalizes to `["01", ""]`; the bare-scalar branch filters, this one doesn't.
Fix: add `.filter(|s| !s.is_empty())` after the map + regression test.

### F5 — medium: fold #149 F-R4 into #139 (error kind + dead severity branches)

close.rs:87 uses bare `domain_bail!` (defaults to Validation) where
GateFailed is correct — but the branch is unreachable: `parse_evidence`
(close.rs:331-341) bails unconditionally on missing evidence, so the config
severity (`warn`/`false`) is ignored for partial evidence and the severity
branches at close.rs:83-105 are dead. This is #139's core defect, confirmed
against current source.
Fix: correct the error kind inside #139's restructure, not as a separate change.

### F6 — decision: #146 blocked by feature work (#149 F-R5)

#146 (v0.3.1, bugfix release) is blocked by 10 tickets including #127
(agent-onboarding feature) and #138 (blocked by #127). A bugfix release
shouldn't wait on a feature.
Recommendation: descope #127/#138 from #146 (ship fixes in v0.3.1, feature
in the next release), record the decision in #146, and re-block #138
accordingly. #146's body/AC are still TBD — fill from #128's P0/P1/P2
summary once decided.

### F7 — low: normalize_blocked_by missing multi-line pass-through test (#149 F-R2 gap)

Commit 77ce6f4 added inline/bare/empty/unquoted cases to lint.rs tests, but
no multi-line (block-style) pass-through case; lint.rs:130 returns
multi-line values as-is — worth pinning with a test.

### F8 — info (no action)

- #132's promised follow-ups were filed as #150 (doctor detection gap) and #151 (load_corpus diagnostics) at `cfbfcb9`.
- #149 F-R6 (update_check ignores DO_NOT_TRACK) unchanged; tracked by #145 (backlog). Coordinate with #135 since the notice lands after the JSON error envelope (runs after `cli::run`).
- Frontier after this review: high = 127, 133, 134, 135, 136, 149; medium = 137, 139, 141 + open feature backlog 79/83/86-91/93.

## Acceptance criteria

- [ ] F1: `cargo fmt --check` green at HEAD on current stable toolchain (or toolchain pinned and gate re-run)
- [ ] F2: indented comment lines no longer corrupt preceding field values; regression test asserts the field value
- [ ] F3: tickets 133-145 and 146 backfilled with #128 finding references; no TBD bodies on release blockers
- [ ] F4: lint filters empty inline-array items; regression test added
- [ ] F5: error-kind fix folded into #139 spec (annotate both tickets)
- [ ] F6: #146 blocker decision recorded in the ticket; #138 re-blocked if descoped
- [ ] F7: multi-line pass-through test added for normalize_blocked_by
- [ ] #149 closed or annotated as superseded by this ticket
- [ ] cargo fmt && cargo clippy --all-targets && cargo test pass clean

## Disposition (verified 2026-08-30, HEAD)

Research + code/test review dispatched 2026-08-30 (raw in `.scratch/t152-review/`).
**7 of 8 findings already resolved by committed work (#167 + #149); only F2 remains.**

| Finding | Verified state at HEAD | Disposition |
|---------|------------------------|-------------|
| F1 (fmt gate red) | RESOLVED — `cargo fmt --check` exit 0 (fixed by #167's `cargo fmt`) | Fixed |
| F2 (indented comment corrupts preceding field) | CONFIRMED — continuation branch (ticket.rs:204) runs before comment branch (:208); indented `# note` merges into prior field's raw value; `parse_tolerates_comment_lines` (:1122) has the fixture but only asserts id/status | **FIX HERE** — the only remaining work |
| F3 (blockers TBD, no #128 ref) | DONE — backfilled during #149 (commit 629ee1a); grep confirms no TBD, all reference #128 | Done |
| F4 (lint inline-array keeps empties) | DONE — fixed as #149 F-R3 (commit 09fc517) + regression test | Done |
| F5 (fold error-kind into #139) | DONE — #139 spec now folds in the F-R4 error-kind fix | Done |
| F6 (#146 blocked by feature) | DONE — #127/#138 descoped to v0.4.0, recorded in #146 | Done |
| F7 (multi-line pass-through test) | DONE — `normalize_value_passes_through_multiline` added (commit 09fc517) | Done |
| F8 (info) | No action | — |

### F2 fix (research + review grounded)

Reorder the parse loop so the comment branch (`line.trim_start().starts_with('#')`)
is evaluated **before** the continuation branch (`(starts_with(' ')||'\t') && !fm.is_empty()`)
in `TicketFile::parse` (ticket.rs ~204-208). Bodies unchanged; only the relative order
of the two else-if blocks swaps.

Guards (from research-yaml.md, YAML 1.2 §6.6 / §3.2.3.3, Verified):
- Keep the comment check a **whole-line** predicate — do NOT strip trailing mid-line `#`
  (a title like `"fix #152"` contains a literal `#` inside a quoted scalar).
- Comments are valid at any indentation and may interleave a block sequence without
  terminating it — so an indented comment inside `blocked_by` must leave the list intact.
- Block-list items (`  - "01"`) start with `-` not `#` → unaffected (review-parser.md).

Tests to add (review-tests.md; minimum T1+T2+T4):
- T1: indented comment after `title` → assert `title == "Commented"` (fails at HEAD, passes after)
- T2: indented comment inside block-style `blocked_by` → assert list == `["01","03"]`
- T4: parse→serialize→parse round-trip — comment preserved verbatim (empty-key passthrough),
  dropped by lint's canonical rewrite, preceding field not merged

Close-readiness: VC1 (fmt) MET; VC3 (dispositioned) MET except F2; VC4 (blockers backfilled)
MET for actual release blockers (138/140/142-145 are descoped/backlog, outside scope). Only
VC2 (F2) blocks the close. #149 already closed on merits (commit 6d91ab2), satisfying the
"#149 closed or annotated" AC.

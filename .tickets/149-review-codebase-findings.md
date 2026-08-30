---
id: "149"
title: "Review 2026-08-26 codebase review findings and action valid ones"
status: done
blocked_by: []
priority: high
validation_criteria:
  - "each finding dispositioned: fixed, wontfix with rationale, or deferred to existing ticket"
  - "lint normalize_blocked_by regression tests added and passing"
  - "tickets 132-141 and 146 reference ticket 128 findings (no TBD bodies on release blockers)"
  - "146 blocker list re-decided: 127/138 in or out of v0.3.1"
  - "cargo fmt && cargo clippy --all-targets && cargo test all pass clean"
---

# Review 2026-08-26 codebase review findings and action valid ones

## What to build

A static codebase review (2026-08-26) verified all open fix tickets against
current source and found them accurate — no new defects beyond the ticketed
set. It also surfaced process/quality findings that need disposition. Review
each finding below and action the valid ones: fix directly, defer to an
existing ticket (annotate), or wontfix with rationale.

### F-R1 — high: release-blocking tickets have TBD bodies, no link to spec

Tickets 132-137, 139, 141, and 146 contain `## What to build / TBD` and
`- [ ] TBD`, with no reference to ticket 128 (the 2026-08-23 audit, F1-F19)
where evidence, locations, and fix sketches live. 128 is done and off the
frontier, so a fresh agent working #132 gets only a title.
Fix: backfill each ticket 132-141 with the 128 finding reference (e.g. "See
#128 F2") in the body and fill in real acceptance criteria. #139 is the worst
case — its body cannot explain "unreachable warn branch" without F10.

### F-R2 — medium: #131 lint half shipped without regression tests

Commit 9875714 rewrote `normalize_blocked_by` (src/commands/lint.rs:147-178)
with no tests; only the parser side got them. 128's AC required regression
tests per confirmed defect class.
Fix: unit-test normalize_blocked_by — bare scalar, inline array, empty,
multi-line pass-through.

### F-R3 — low: lint inline-array branch keeps empty items

lint.rs:157-163 maps without filtering, so `blocked_by: ["01", ]` normalizes
to `["01", ""]`. The new bare-scalar branch filters; this one doesn't.
Fix: add `.filter(|s| !s.is_empty())` after the map.

### F-R4 — low: close.rs partial-evidence bail uses wrong error kind

src/commands/close.rs:87 `domain_bail!` defaults to Validation for a gate
failure; the no-evidence gate at :65 correctly uses GateFailed.
Fix: `domain_bail!(GateFailed, ...)`.

### F-R5 — needs decision: #146 blocker list may be over-scoped

v0.3.1 is blocked by 10 tickets including #127 (agent-onboarding feature) and
#138 (manifest regen, blocked by #127). #127 is not a data-loss/contract fix.
Decide: descope #127/#138 from #146, or explicitly accept the delay.

### F-R6 — medium: update_check ignores DO_NOT_TRACK and runs after output

src/main.rs:115 runs `check_for_update()` after `cli::run()`, so its stderr
notice lands after the JSON error envelope; update_check.rs:20-26 honors
TKT_UPDATE_CHECK/CI but not DO_NOT_TRACK or JSON_OUTPUT. Overlaps #135/#145 —
coordinate the fix with those tickets rather than duplicating.

## Acceptance criteria

- [x] Each finding F-R1 through F-R6 dispositioned (fixed / deferred / wontfix)
- [x] F-R2: normalize_blocked_by regression tests added and passing
- [x] F-R1: tickets 132-141 and 146 reference #128 findings; no TBD bodies on release blockers
- [x] F-R5: 146 blocker list decision recorded in the ticket
- [x] cargo fmt && cargo clippy --all-targets && cargo test pass clean

## Disposition (verified 2026-08-30, HEAD)

Research + code/ticket review dispatched 2026-08-30 (raw in `.scratch/t149-review/`).
#149 is **superseded by #152** (per #152 provenance: "close #149 once its findings are
dispositioned here or annotated there"). #156 defers #149/#152 items back to #152, it does
not supersede it.

| Finding | Verified state at HEAD | Disposition |
|---------|------------------------|-------------|
| F-R1 | CONFIRMED — only #149/#152/#156 reference #128; 10/11 blockers (132-141,146) still TBD (except #132, done) | **Defer to #152 F3** (identical, tracked there) |
| F-R2 | PARTIAL — 8 regression tests exist (lint.rs:239-294); only the multi-line pass-through case is missing | **Fix here** — one small test on the `raw.contains('\n')` short-circuit |
| F-R3 | CONFIRMED — inline-array branch (lint.rs:171-181) doesn't filter empties; bare-scalar branch (lint.rs:188) does | **Fix here** — `["01", ]`→`["01", ""]` fabricates a value; YAML 1.2 treats a trailing comma as a one-element list, so dropping the empty is spec-aligned [research-lists.md, Verified]. Add `.filter` + regression test |
| F-R4 | CONFIRMED + **DEAD CODE** — close.rs partial-evidence gate uses bare `domain_bail!` (Validation not GateFailed), but `parse_evidence` bails via `?` before the gate is reached → zero runtime impact | **Defer to #139** (fix inside its restructure per #152 F5; do not touch separately) |
| F-R5 | CONFIRMED over-scoped — #146 `blocked_by` still includes #127 (feature); #138 chains via `blocked_by: [127]` | **Decision: descope** #127/#138 from #146. SemVer clause 6 + GitLab patch-release policy: a bugfix release contains fixes only and must not be gated on feature work; feature ships in v0.4.0 [research-release.md, Established]. Record in #146 |
| F-R6 | CONFIRMED — main.rs runs update_check after `cli::run()`; honors QUIET/CI but not DO_NOT_TRACK | **Defer to #135/#145** (coordinate). Small guard `if DO_NOT_TRACK { return }`; post-output ordering is defensible |

**Plan:** fix F-R2 + F-R3 here (both genuinely #149-specific, small, isolated); annotate
F-R1/F-R4/F-R6 as deferred; record the F-R5 descope decision in #146; then close #149 as
superseded-by-#152. Note: F-R1 and the "no TBD bodies" AC are owned by #152, so those ACs
close via `--force` with the superseded rationale rather than merits-based completion.

## Resolution (2026-08-30)

All six findings dispositioned and actioned. F-R2 (multi-line pass-through test) and F-R3 (inline-array empty-item filter) fixed in lint.rs with regression tests. F-R5 actioned: #127/#138 descoped from v0.3.1 (#146), decision recorded per SemVer/GitLab patch-release policy. F-R1 completed (Option A): all v0.3.1 blockers (133-137,139,141) and #146 backfilled with their #128 finding references and real acceptance criteria, replacing TBD bodies. F-R4 folded into #139's spec (confirmed dead code); F-R6 deferred to #135/#145 (annotated). #149 originally framed as superseded-by-#152, but closed on merits since all five validation criteria are genuinely satisfied.


### Verification
1. ✓ each finding dispositioned: fixed, wontfix with rationale, or deferred to existing ticket — "All 6 findings (F-R1..F-R6) dispositioned in the ticket's Disposition table with rationale + file:line"
2. ✓ lint normalize_blocked_by regression tests added and passing — "F-R2/F-R3 regression tests pass: normalize_inline_array_drops_trailing_comma_empty + normalize_value_passes_through_multiline; cargo test 206 passed (144 unit + 62 integration) 0 failed"
3. ✓ tickets 132-141 and 146 reference ticket 128 findings (no TBD bodies on release blockers) — "grep -rl #128 .tickets shows blockers 133,134,135,136,137,139,141,146 all reference #128; grep TBD across 132-141+146 returns empty (no TBD bodies on release blockers)"
4. ✓ 146 blocker list re-decided: 127/138 in or out of v0.3.1 — "#146 blocker list re-decided: #127/#138 descoped to v0.4.0, recorded in #146 body; blocked_by now 131,132,133,134,135,136,137,139,141"
5. ✓ cargo fmt && cargo clippy --all-targets && cargo test all pass clean — "Gate green: cargo fmt --check exit 0; cargo clippy --all-targets 0 warnings; cargo test 206 passed; tkt validate --brief passes"

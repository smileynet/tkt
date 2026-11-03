# Historical closure warning review (#239)

## Reproduction and limits

Ticket line references below refer to the pre-cleanup worktree baseline
`b7c71d382d3bffee4ad28431363507c04b8d0eb8`.

On worktree `b7c71d3`, `tkt audit --deep --brief` printed 20 warning records:
five `low-evidence-closure` warnings and fifteen `template-only-closure`
warnings. Ticket #174 appears in both sets. The command also printed five
informational `high-priority-open` findings and ended `pass (25 finding(s))`.
That status and exit 0 do not mean the audit is warning-free.

The installed CLI identified itself as `tkt 0.4.0 (4130a85)`. The intervening
commits to the worktree were ticket/dashboard updates, and `src/audit.rs` and
`src/commands/audit.rs` have no diff from that binary's commit. The observed
warning set therefore matches the current audit implementation.

`check_validation_evidence` in `src/audit.rs:140-160` requires a body containing
both `### Verification` and `✓`; checked AC boxes and a resolution paragraph do
not satisfy that mechanical test. `check_template_only` in
`src/audit.rs:219-247` searches the whole ticket body for literal template
strings; it does not decide whether the rest of the ticket contains a real
specification or resolution.

For all five low-evidence records below, the current ticket has checked criteria
but no standalone `### Verification` section. The close commits changed statuses
and marked criteria complete (some also revised criteria); none added a
standalone verification section. That proves the closure record lacks this
evidence; it does **not** prove no checks were run elsewhere. Original execution
proof remains unknown. Any source or tests visible today are new corroboration,
not retroactive evidence for the historical close.

## Five `low-evidence-closure` warnings

| Ticket | Disposition and file evidence |
|---|---|
| #154 | **Confirmed omission; original proof unknown.** `.tickets/154-detect-hand-flipped-done.md:56-63` has checked criteria for validate, fix, guidance, and tests, but no `### Verification`. Close commit `e8593af` sets status and marks/revises criteria complete; it adds no verification section. Current implementation/tests cannot establish what was run at that close. |
| #155 | **Confirmed omission; original proof unknown.** `.tickets/155-explore-enforcement-hooks.md:70-76` records checked research/decision deliverables but no verification section. Close commit `a1b4242` sets status and marks criteria complete; it adds no verification section. The decision text is preserved in the ticket; no separate historical test/run proof is recorded. |
| #156 | **Confirmed omission; original proof unknown.** `.tickets/156-review-delta-findings-2026-08-27.md:78-84` marks D1-D5 and a full gate as complete but has no `### Verification`. Close commit `6c43cff` sets status and marks/revises criteria complete; it adds no verification section. Its review conclusions remain in the body; the claimed gate result is not independently evidenced in the closure record. |
| #174 | **Confirmed omission; original proof unknown.** `.tickets/174-validate-stub-body.md:51-57` checks the implementation, strict-mode, false-positive, and gate criteria without a verification section. Close commit `f72ed58` sets status and marks criteria complete; it adds no verification section. Current `src/findings.rs` and tests show the feature exists now, but do not establish its historical test run. |
| #206 | **Confirmed omission; original proof unknown.** `.tickets/206-audit-grandfather-cutoff.md:40-50` checks the cutoff behavior, exceptions, regression test, and config errors without `### Verification`. Close commit `2fd3690` sets status and marks/revises criteria complete; it adds no verification section. Current `src/config.rs`, `src/commands/audit.rs`, and `src/audit.rs` contain the cutoff implementation and tests; this is current-source corroboration, not historical execution proof. |

## Fifteen `template-only-closure` warnings

“Confirmed” below means the literal placeholder remains in the record. It does
not establish that the work was unfinished. “False positive” means the detector
matched a quoted example in substantive prose, not a template block. “Obsolete”
means the ticket records a duplicate or rejected/superseded request.

| Ticket | Disposition and file evidence |
|---|---|
| #102 | **Obsolete duplicate closure.** `.tickets/102-unified-config-cascade.md:13-19` is a placeholder, but `:21-27` says it duplicated #103 and came from a cancelled command. The duplicate was explicitly closed; no implementation proof is claimed for #102 itself. |
| #129 | **Obsolete duplicate closure.** `.tickets/129-debug-log-only.md:14-20` is a placeholder; `:22-29` says it duplicates already-implemented #130 and records that disposition. This warning is not evidence of missing #130 behavior. |
| #149 | **False positive.** The detector's `- [ ] TBD` substring occurs inside the F-R1 discussion at `.tickets/149-review-codebase-findings.md:25-30`; there is no template-only body. The ticket contains detailed findings and a resolution at `:83-105`. |
| #152 | **False positive.** The marker is quoted in the F3 discussion at `.tickets/152-review-findings-2026-08-27.md:57-62`, not used as the ticket's own placeholder body. The record has substantive findings and a resolution/verification at `:123-171`. |
| #174 | **False positive for the template-only classification.** `.tickets/174-validate-stub-body.md:18`, `:28-30`, and `:45-48` quote the template sentinel while specifying its detection. The ticket has a concrete feature spec and checked criteria at `:32-57`. Its separate missing-evidence warning remains confirmed above. |
| #48 | **Confirmed template-only record.** `.tickets/48-dotfile-config.md:11-17` leaves both the What-to-build body and AC as `TBD`. The resolution at `:19-21` claims an implementation and 84 tests, but does not reconstruct the missing original spec or criteria. Preserve the warning as a ticket-record gap; do not infer the test run beyond that existing note. |
| #80 | **Confirmed stale placeholder block; not template-only overall.** `.tickets/80-context-system.md:18-46` contains a substantive context design and checked criteria; `:49-55` then repeats an empty template block. Resolution and verification follow at `:57-68`. The literal warning was real, but the “template-only” interpretation is false. The duplicate block was removed in the narrow cleanup below. |
| #81 | **Confirmed stale placeholder block; not template-only overall.** `.tickets/81-dry-run-mutations.md:23-55` has detailed behavior and checked criteria; `:58-63` is a repeated empty template block. Resolution and seven verification entries are at `:65-76`. The close record is not template-only; the duplicate block was removed in the narrow cleanup below. |
| #82 | **Obsolete rejected request with stale placeholder.** `.tickets/82-urgency-scoring.md:17-55` contains a proposed computed sort and checked criteria; `:59-65` is a trailing blank template. The resolution at `:67-72` explicitly says “Won't implement” because priority+ID order is intentional. Current `AGENTS.md:265` independently says not to implement computed frontier sort overrides. The checked feature criteria conflict with that resolution; both remain unchanged here, and the placeholder is not evidence that urgency scoring should be implemented. |
| #84 | **Confirmed stale placeholder block; not template-only overall.** `.tickets/84-capabilities-schema.md:19-55` contains the schema proposal and checked criteria; `:59-65` is a repeated blank template. Resolution and verification are at `:67-74`. The warning identified leftover text, not an empty work record; the duplicate block was removed below. |
| #91 | **Confirmed stale placeholder block; not template-only overall.** `.tickets/91-agent-close-validation.md:15-133` contains a detailed design and checked criteria; `:135-142` appended another title and blank template. The apparent Resolution/Verification at `:54-65` is inside a fenced example, not a ticket closure section. Thus the template-only inference is false, but the historical closure also lacks a standalone verification record; do not treat the example as original proof. Only the trailing duplicate template block was removed below. |
| #92 | **Confirmed stale placeholder block; not template-only overall.** `.tickets/92-implement-validation-criteria.md:28-66` has concrete design and checked ACs; `:70-76` repeats a blank template. The actual resolution and verification are at `:77-93`. Only the duplicated block was removed below. |
| #95 | **Obsolete/superseded design with stale placeholder.** `.tickets/95-env-tags.md:18-72` records the earlier `env_tags` proposal and checked criteria; `:76-82` is a repeated empty template. Resolution at `:84-90` says the delivered representation became `requires` with legacy `env` compatibility. Current ticket schema likewise uses `requires`/`env`, not `env_tags`. The old proposal was superseded; do not claim the old flag/field was implemented. |
| #96 | **Confirmed stale placeholder block; not template-only overall.** `.tickets/96-cut-v020.md:15-31` specifies the release and has checked criteria; `:35-41` repeats an empty template. Resolution and verification at `:43-50` record the release. Only the duplicated block was removed below. |
| #97 | **Confirmed stale placeholder block; not template-only overall.** `.tickets/97-crates-io-page-research.md:16-36` has substantive research criteria; `:40-47` repeats an empty template. Resolution and four verification entries are at `:47-55`. Only the duplicated block was removed below. |

## Disposition boundary

The audit reproduces, but historical proof cannot be recovered from the closure
records. No checks were rerun for this review, and no `status`, AC, resolution,
or evidence field was changed. Current source can corroborate that some features
exist; it cannot transform an absent contemporaneous verification record into
historical evidence.

The clearly quoted-marker false positives (#149, #152, #174) should be handled
by a separate audit-rule change, not by deleting meaningful discussion from
completed tickets. Duplicate/rejected tickets (#102, #129, #82, #95) are
non-actionable for product implementation. The seven duplicate empty blocks
listed below were removed; no statuses, checked ACs, resolutions, or evidence
were changed. #48's only spec/AC body is itself a placeholder, so reconstructing
it from the resolution would risk inventing original intent.

## Narrow cleanup and fresh reproduction

At the pre-cleanup baseline above, the seven completed tickets contained these
demonstrably duplicate empty template blocks, each after substantive material
or completed criteria:

- #80: repeated title, What-to-build `TBD`, and unchecked `TBD` AC block
- #81: repeated title, What-to-build `TBD`, and empty AC block
- #84: repeated title, What-to-build `TBD`, and unchecked `TBD` AC block
- #91: repeated title, What-to-build `TBD`, and empty AC block
- #92: repeated title, What-to-build `TBD`, and empty AC block
- #96: repeated title, What-to-build `TBD`, and unchecked `TBD` AC block
- #97: repeated title, What-to-build `TBD`, and empty AC block

Removed only those blocks. Preserved the original prose/specifications,
acceptance checkboxes, resolutions, and verification text. Left #48 untouched
because its original specification is unavailable. Left #82 and #95 unchanged
because their historical checkboxes and changed decisions conflict.

After cleanup, a fresh worktree build (`cargo run --quiet -- --version` reported
`tkt 0.4.0 (b7c71d3)`) reproduced with environment variables `DO_NOT_TRACK=1`,
`TKT_NO_USER_CONFIG=1`, and `TKT_UPDATE_CHECK=0` set for
`cargo run --quiet -- audit --deep --brief`.
It reported **13 warnings**: the same five low-evidence records and eight
template records (#102, #129, #149, #152, #174, #48, #82, #95), plus the same
five informational findings: `pass (18 finding(s))`. Audit source files were
unchanged between the initial and fresh-build probes; the seven placeholder
removals account for the seven warning records that disappeared. Other parallel
worktree changes are not attributed to this review. The false positives and
unresolved historical proof remain visible and explicitly classified.

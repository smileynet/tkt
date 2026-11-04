# Cross-project evidence review — 2026-11-03

The owner requested review of production evidence in other `D:/code` projects and
reopening recommended tickets. This pass found **no evidenced reason to reopen a
completed/cancelled ticket or reactivate either remaining parked ticket**. Existing
open work already covers the demonstrated follow-ups. This is an inspection of
working project data and attributed operational reports, not new production
qualification of those applications.

## Observed scope and results

Current installed `tkt 0.4.0 (35c6753)` ran `validate --output json` and
`query --all-context` in 78 immediate project directories containing `.tickets`.
The tkt checkout and duplicate candidate/staging families (`tkt*`, `grey-windows*`,
`windows-*`, `sharktank*`, `release-*`) were excluded. The recorded
[per-project aggregate](cross-project-review-2026-11-03.json) identifies the exact
projects, command outcomes, findings and source-file hashes. Commands used
`DO_NOT_TRACK=1`, `TKT_UPDATE_CHECK=0`, `TKT_DEBUG=0`; no consent or other project's
ticket/configuration was changed.

- **Pass, read availability:** all 78 queries returned structured rows, with
  diagnosed skips in two projects; no operational crash or timeout was observed.
- **Validation findings:** 76 validations exited 0, but their warnings/information
  remain findings, not clean bills of health. Two exited 1 for three malformed
  files: Chunky's `agy-findings-8539e59.md` lacks `id`; Crew Research's tickets 100
  and 112 duplicate `validation_criteria`. These are observed corpus defects,
  correctly reported by the current parser, not evidence to reopen parser repairs.
- Parsed corpus: 3,482 tickets; 785 open/in-progress, of which 420 have no tags;
  258 backlog. These are current snapshots, not rates, repeated incidents, team
  intent or proof that an advisory would help. Empty/example and old project boards
  are included in scope; this is not 78 independently active production deployments.
- Advisory findings include 161 `no-source-reference` and 378
  `frontier-readiness` entries. These demonstrate that the shipped checks inspect
  real project data; they do not prove reference sufficiency or justify a hard gate.

## Ticket dispositions

| Ticket or report | Evidence | Disposition |
|---|---|---|
| #147 consent resets | No platform/config/enable/restart sequence showing consent reversal was found. Read-only `telemetry --show --all` contained 27 retained events, two Learning Library `new` validation errors, and no consent-transition proof. | Keep parked; no speculative consent change. Retained telemetry is a small sample, not complete historical usage. |
| #184 narrative plan parsing | Crew Research ticket 149 requests narrative-only planning and less duplicated status; root `PLAN.md` files are also in use. That establishes a planning preference, not a demonstrated need for tkt to parse freeform prose. ADR 0002 and completed #187 provide the derived-projection direction; owner explicitly deferred narrative parsing. | Keep parked. Revisit only with a bounded workflow that projection cannot serve and a parser/ownership proposal. |
| #168 advisory demand | 420 of 785 parsed active tickets lack tags; 258 are backlog. No before/after guidance baseline or attributed complaint establishes unwanted behavior. | Already open; attach the measurement, retain demand assessment. No nudge or default-status change. |
| #173 / #175 readiness/source checks | Crew Research ticket 148 explicitly separates mechanical slots from judgment of adequate context. Current checks produce advisory findings across real corpora. | Keep done; the bounded implementation matches that split. Judgment work is not a missing mechanical implementation. |
| #86 GitHub import / #125 contract | Grey Matter tickets 10/11 describe a tracker-adapter need and conditional use of `tkt import --github`. | #86 is already open, #125's design is done. This does not reopen the design or claim a runtime importer exists. |
| Historical close failures | Teach Me ticket 263 reports four successful `tkt close` operations. The old local checkout still has stale guidance, but freshly fetched origin/main at bc4bd35 already instructs contributors to use `tkt close`. | Keep tkt close repairs done; any remaining verification belongs to that project's existing ticket, with its superseded premise made explicit. |
| Historical dry-run writes | Grey Matter ticket 84 records that the old lint caveat was obsolete after verification. Current #250/#251 repair separate newly reproduced effects. | Keep the fixed tickets done; broader #191 remains in progress. |
| CLI/pack mismatch | Grey Matter ticket 98 records the exact version skew and completed compatibility audit with identity-bound reuse. | Keep that repair done. Compatibility remains a deployment check, not evidence of a current tkt regression. |
| Learning Library's two recent validation failures | Retained telemetry records two `new` validation errors followed by success. Its current uncommitted config diff changes `require_validation_evidence = "error"` to `true`. | Configuration mismatch is corroborated, but telemetry has no arguments/full error and cannot prove each failure's cause. No product ticket is reopened from this sequence. |

## Primary records inspected

Teach Me exposed a checkout-drift trap: local fc5335a and freshly fetched upstream bc4bd35 diverge by 1,170/1,193 commits. An ordinary fetch updated main but refused conflicting existing tags; a targeted no-tags fetch then passed. No tags, branches, staged changes or working files were overwritten. The current upstream guidance must govern the reactivation decision, rather than the stale local copy.

The aggregate receipt hashes these source records: Crew Research tickets 148/149;
Teach Me ticket 263; Grey Matter tickets 10/11/84/98; and Learning Library's current
project configuration. Their ticket numbers belong to their respective projects,
not the local tkt board. Other project's files and uncommitted work were preserved.

No tickets were closed during this review. Independent critic review checks the
reopening recommendation; the plan and dashboard record the resulting disposition.

## Final review and verification

Independent critic review accepted no reactivation and withdrew its initial Teach
Me recommendation after inspecting freshly fetched upstream. Historical ticket
263 can be reconciled separately on a current isolated checkout; its original
documentation premise does not justify reopening a tkt repair or another active
implementation thread.

PASS: local `tkt validate --strict --brief` and
`tkt sync-plan --check --strict --brief` each report zero findings. The dashboard
snapshot contains all 252 tickets and 12 frontier items; its checker reports zero
errors/warnings and zero pending asks. `git diff --check` is clean. This pass changes
only tickets, evidence, plan and dashboard content; Rust code and renderer source
are unchanged, so no Rust rebuild or rendered-UI acceptance is claimed.

The compact aggregate and interpretation are promoted evidence. Disposable review
helpers/raw output are recycled after verification. Preexisting scratch, managed
build/install outputs and other projects' staged/untracked work are preserved;
their earlier logical-size inventory is in
[the retained-artifact receipt](parked-retained-artifacts-2026-11-03.json), not a
new measurement of those shared caches.

Actual disposition: recycled 5 review helper/raw-output files (421847 logical bytes) to the OS Recycle Bin. No unknown-owned artifacts were removed.

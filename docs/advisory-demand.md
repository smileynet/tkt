# Missing-tag and backlog advisories: measured decision

Ticket #168 is a measurement and disposition task. **Do not add a runtime nudge
from this evidence.** Missing metadata and deferred work are observable, but this
sample does not establish that an additional per-command reminder would solve a
recurring post-guidance problem. No telemetry was enabled or consent changed.

## Frequency and scope

The [cross-project snapshot](evidence/cross-project-review-2026-11-03.json) queried
78 immediate `D:/code` project boards with `tkt 0.4.0 (35c6753)`, excluding duplicate
candidate/staging families. It parsed 3,482 tickets; three malformed files in two
projects were skipped with diagnostics. This mixes active, historical and example
corpora, not 78 qualified production deployments.

| Observation at that snapshot | Frequency | What it establishes |
|---|---:|---|
| Projects with open/in-progress tickets | 53 / 78 | Where an active-ticket denominator exists. |
| Projects with at least one untagged active ticket | 38 / 78 | Missing tags are distributed across boards, not one isolated file. |
| Untagged open/in-progress tickets | 420 / 785 (53.5%) | A metadata prevalence count; no judgment that each omission is harmful. |
| Projects with backlog tickets | 31 / 78 | Deferral is used across projects. |
| Backlog tickets across every status | 258 / 3,482 (7.4%) | Stored backlog prevalence, not the frequency of inappropriate parking. |

The largest untagged active cohorts were GDHelper Pipeline (50/54), KC2 Mods
(43/43), Lacrosse Bosse Helper (37/60), and Bigg Pi (31/31). Sampling high-count
boards exposes prevalence; it does not establish their intended stream taxonomy
or the number of navigation failures caused by absent tags.

## Local history and interpretation

Completed #167 reports a 2026-08-29 baseline of 20 backlog tickets out of 46 live
tickets and 151 untagged tickets out of 167. Those are attributed historical
measurements in that ticket, not freshly reproduced historical Git-tree counts.

A fresh local query before this measurement closed found 253 tickets, 136
untagged. All 136 were terminal: none of the 15 open/in-progress or two backlog
tickets lacked tags. Backlog was two of 17 nonterminal tickets. This is a current
snapshot, not a matched before/after cohort. Bulk triage, retagging, closures and
the changing corpus confound any claim that guidance alone caused improvement.

The [parked-work review](parked-review-2026-11-03.md) supplies one concrete workflow
episode: 23 parked tickets were reviewed and 21 reactivated after the owner asked
for actionable work. That demonstrates real over-parking in this repository. The
repeated requests within that same session are not counted as independent
incidents, and the result does not show that a creation-time reminder would have
prevented the original deferrals. The review and existing plan/frontier exposed
the work without adding another runtime message.

Representative counterexamples bound the proposed check:

- Learning Library ticket 16 is an untagged authored-guide task with explicit
  acceptance and closure evidence. It shows that absence of tags alone is not a
  readiness failure; this review does not establish whether that project needed
  multiple streams. At inspection its resolution heading says 2026-11-04: tkt's
  close date is computed from UTC epoch days, while this report uses the client's
  America/Los_Angeles date. The file's observed write time, 2026-11-04 00:38:03 UTC,
  is November 3 at 16:38:03 PST. This is not future local evidence. The aggregate
  counts remain the earlier frozen snapshot, not a live status claim.
- Lacrosse Bosse Helper ticket 107 explicitly defers advisory maintenance beyond a
  behavior-preserving release baseline. A generic backlog warning would include
  this deliberate planning choice; the ticket does not report a parking problem.
- Local #147 has an explicit missing-reproduction gate, and #184 an owner deferral.
  Both belong in backlog despite satisfied structural prerequisites.

The previously inspected retained telemetry contained only 27 events, with two
Learning Library `new` validation errors followed by success. It records no
missing-tag or inappropriate-backlog classification, no reminder exposure and no
causal result. It cannot supply a demand or success rate. Do not enable collection
or reinterpret those unrelated validation errors to fill that evidence gap.

## Decision and revisit condition

Keep existing creation guidance (actionable work defaults open, backlog requires a
reason, streams use tags) and use the existing backlog query/dashboard review.
No new runtime advisory, hard gate, required tag, default-status configuration,
telemetry event, automatic retagging or bulk status change is selected.

Revisit an opt-in advisory only after attributed, separate post-guidance incidents
show that actionable work was parked without an intended deferral, or missing
stream tags actually hid work. Record the creation command/context, intended
scope and subsequent correction; compare with intentional single-stream/deferred
controls. A bounded trial would need observed usefulness and noise, preserved
quiet/JSON behavior and a separate implementation ticket. That is a future
evidence threshold, not an untracked implementation commitment or scheduled study.

The [measurement receipt](evidence/advisory-demand-168.json) pins the aggregates,
source identities and limitations. Independent critic acceptance and arithmetic,
link and scope checks precede closure; this task adds no product code.

Independent critic accepted the final arithmetic, bounded decision and UTC/local chronology. Both disposable measurement helpers (2120 logical bytes across 2 files) were recycled after promotion; shared build/batch artifacts remain for ongoing work.

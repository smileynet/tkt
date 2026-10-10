# tkt delivery plan

This plan is the repository-level view of unfinished work. Ticket files remain the
source of truth for implementation detail and acceptance criteria; this table makes
release sequencing and intentionally deferred work visible to `tkt sync-plan`.

## v0.3.1 — correctness and release gate

`#146` is the patch-release gate. Do not tag or publish `v0.3.1` until its defect
blockers and the cross-platform atomic-write verification below are done, the canonical
gate passes, and the changelog records the release.

| ID | Work | Status |
|---:|---|---|
| 133 | Preserve unquoted YAML scalar bytes | open |
| 134 | Make renumber citation-safe and crash-consistent | open |
| 135 | Put JSON error envelope last on stderr | open |
| 136 | Apply global dry-run to every mutation | open |
| 137 | Resolve the real upstream branch for race detection | open |
| 139 | Repair evidence-gate severity and discarded-evidence behavior | open |
| 141 | Count uppercase checked acceptance boxes | open |
| 152 | Fix indented-comment parser corruption | open |
| 181 | Verify atomic writes across Windows and Linux | open |
| 146 | Cut, tag, and document v0.3.1 | open |

## Quality, integrity, and dogfooding

These items strengthen the ticket contract and the project’s ability to rely on its
own health signals. They should follow the patch-release blockers unless a defect
requires promotion.

| ID | Work | Status |
|---:|---|---|
| 153 | Prevent stale installed-binary validation | backlog |
| 156 | Disposition remaining delta-review findings | open |
| 160 | Decide legacy missing-resolution policy | backlog |
| 163 | Backfill missing IDs from filename prefixes | open |
| 164 | Support canonical slug IDs in validation | open |
| 165 | Write project-scoped configuration safely | open |
| 173 | Add advisory frontier Definition-of-Ready validation | open |
| 175 | Flag tickets with no source reference | open |
| 176 | Improve new-ticket body template and add body input | open |
| 177 | Finish ticket-content guidance after #176; evidence guidance is covered by #179 | open |
| 185 | Restore automatic push/PR CI verification (release engineering, not a #146 blocker) | open |
| 186 | Reconcile or retire the historical parity harness | open |

## v0.4.0 — agent experience and extensions

These are useful product improvements, deliberately not blockers for the corrective
patch release.

| ID | Work | Status |
|---:|---|---|
| 127 | Improve agent discovery and init guidance | open |
| 138 | Complete the machine-readable capabilities manifest | open |
| 161 | Support aggregate reporter provenance in steering | open |
| 168 | Consider advisory nudges for tag-less/backlog work | backlog |
| 79 | Safely undo the last tkt mutation | open |
| 83 | Edit a ticket body through the configured editor | open |
| 86 | One-way GitHub issue import | open |
| 87 | Ticket body templates | open |
| 88 | Recent-completions report | open |
| 89 | Lifecycle hooks | open |
| 90 | Open ticket-body URLs | open |
| 93 | Pluggable ticket schema and meta fields | open |

## Post-v0.3.1 lifecycle and plan design

These decisions change core contracts or introduce an optional planning surface. Keep
them out of the active frontier until the corrective release is complete.

| ID | Work | Status |
|---:|---|---|
| 182 | Decide a delete/cancel lifecycle mechanism | backlog |
| 184 | Implement opt-in narrative sync-plan support | deferred (ticket open) |

ADR 0002 makes plan-as-projection the strategic primary direction; narrative support
is the opt-in secondary path in #184. #187 owns the primary projection renderer; keep
the two efforts independently schedulable.

| ID | Work | Status |
|---:|---|---|
| 187 | Generate a derived sync-plan projection from the ticket corpus | backlog |

## Intentionally deferred backlog

These tickets remain visible for planning but are not commitments for the current
release cycle.

| ID | Work | Status |
|---:|---|---|
| 66 | Move cargo-dist workspace metadata | backlog |
| 69 | Explore agent/harness integrations | backlog |
| 94 | Validate the VC flag | backlog |
| 99 | Configure update-check interval | backlog |
| 115 | Review telemetry/debug patterns | backlog |
| 125 | Research additional import schemas | backlog |
| 140 | Correct exit-code taxonomy leaks | backlog |
| 142 | Repair legacy CREW_ENV edge cases | backlog |
| 143 | Reconcile quiet and JSON output | backlog |
| 144 | Keep query a full-corpus operation under context | backlog |
| 145 | Honor DO_NOT_TRACK in update checking | backlog |
| 147 | Investigate telemetry consent resets | backlog |
| 148 | Support a primary-agent role pattern | backlog |
| 150 | Have doctor detect ejected tickets | backlog |
| 151 | Route corpus-ejection diagnostics through normal output policy | backlog |
| 157 | Add optional hook installation | backlog |
| 158 | Add per-agent ticket-write nudges | backlog |
| 159 | Document CI enforcement usage | backlog |

# tkt delivery plan

This plan is the repository-level view of unfinished work. Ticket files remain the
source of truth for implementation detail and acceptance criteria; this table makes
release sequencing and intentionally deferred work visible to `tkt sync-plan`.

## v0.4.0 — shipped workflow changes and correctness gate

`#146` is the minor-release gate. v0.3.1 was never tagged and is intentionally skipped.
Do not tag or publish v0.4.0 until the defect and release-readiness blockers below are
done, the canonical gate passes, and the curated changelog records the release.

| ID | Work | Status |
|---:|---|---|
| 133 | Preserve unquoted YAML scalar bytes | ✅ done |
| 134 | Make renumber citation-safe and crash-consistent | ✅ done |
| 135 | Put JSON error envelope last on stderr | ✅ done |
| 136 | Apply global dry-run to every mutation | ✅ done |
| 137 | Resolve the real upstream branch for race detection | ✅ done |
| 139 | Repair evidence-gate severity and discarded-evidence behavior | ✅ done |
| 141 | Count uppercase checked acceptance boxes | ✅ done |
| 152 | Fix indented-comment parser corruption | open |
| 181 | Verify atomic writes across Windows and Linux | open |
| 188 | Reject ambiguous duplicate frontmatter keys | open |
| 189 | Unify env and requires into one eligibility model | open |
| 150 | Make doctor strict results agree with validate | ✅ done |
| 160 | Decide and apply the legacy missing-resolution validation baseline | open |
| 185 | Restore automatic push/PR CI verification | open |
| 201 | Remediate agent-facing help and release metadata drift | open |
| 204 | Parse single-item block-style dependencies correctly | ✅ done |
| 205 | Keep migration confirmation without writing during dry runs | ✅ done |
| 146 | Cut, tag, and document v0.4.0 | open |

## Quality, integrity, and dogfooding

These items strengthen the ticket contract and the project’s ability to rely on its
own health signals. They follow the v0.4.0 gate unless they are explicitly listed
above as release blockers.

| ID | Work | Status |
|---:|---|---|
| 153 | Prevent stale installed-binary validation | backlog |
| 156 | Disposition remaining delta-review findings | open |
| 163 | Backfill missing IDs from filename prefixes | open |
| 164 | Support canonical slug IDs in validation | open |
| 165 | Write project-scoped configuration safely | open |
| 173 | Add advisory frontier Definition-of-Ready validation | open |
| 175 | Flag tickets with no source reference | open |
| 176 | Improve new-ticket body template and add body input | open |
| 177 | Finish ticket-content guidance after #176; evidence guidance is covered by #179 | open |
| 186 | Reconcile or retire the historical parity harness | open |
| 190 | Parse known config keys into strict typed policies | open |
| 191 | Centralize mutation effects and dry-run policy after #136 | backlog |

## v0.5.0 — agent experience and extensions

These are planned product improvements, deliberately not blockers for the v0.4.0
release of already-merged behavior.

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
| 202 | Deploy tkt guidance and skill safely across supported harnesses | open |

## v0.5.0 — native relations and label namespaces

Close the structural gaps surfaced by the grey-matter skill-adaptation work by
adding native concepts (a `parent` relation, config-declared label namespaces, and
an `assignee` field) rather than emulating them with tag conventions and an external
lint script. All additive and backward compatible. Work in the order below: the
three roots first (`192`, `193`, `194`), their dependents, then the docs pass last.

| ID | Work | Status |
|---:|---|---|
| 192 | parent field: schema, parse, serialize | open |
| 193 | `[labels.*]` config: parse single-select + closed-value namespaces | open |
| 194 | assignee field + claim-by-assignment + frontier honors it | open |
| 195 | parent validation: dangling + cycle checks (after 192) | open |
| 196 | parent CLI + query/capabilities surface (after 192) | open |
| 197 | parent rollup + frontier semantics (after 192, 196) | open |
| 198 | label-namespace validation: single-select + closed-set (after 193) | open |
| 199 | docs, capabilities, CHANGELOG for parent/assignee/labels (after 194, 196-198) | open |
| 200 | parent-scoped ready frontier (after 196, 197) | backlog |

## Release scope decision record

| ID | Work | Status |
|---:|---|---|
| 203 | decide v0.4.0 release scope before moving planned work to v0.5.0 | ✅ done |

## Post-v0.4.0 lifecycle and plan design

These decisions change core contracts or introduce an optional planning surface. Keep
them out of the active frontier until the v0.4.0 release is complete.

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

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
| 152 | Fix indented-comment parser corruption | ✅ done |
| 181 | Verify atomic writes across Windows and Linux | ✅ done |
| 188 | Reject ambiguous duplicate frontmatter keys | ✅ done |
| 189 | Unify env and requires into one eligibility model | ✅ done |
| 150 | Make doctor strict results agree with validate | ✅ done |
| 160 | Decide and apply the legacy missing-resolution validation baseline | ✅ done |
| 185 | Restore automatic push/PR CI verification | ✅ done |
| 201 | Remediate agent-facing help and release metadata drift | ✅ done |
| 204 | Parse single-item block-style dependencies correctly | ✅ done |
| 205 | Keep migration confirmation without writing during dry runs | ✅ done |
| 146 | Cut, tag, and document v0.4.0 | ✅ done |

## Post-v0.4.0 quality, integrity, and dogfooding

These items strengthen the ticket contract and the project’s ability to rely on its
own health signals. They are explicitly deferred from the v0.4.0 release; the
release gate is limited to the completed blocker table above and #146.

| ID | Work | Status |
|---:|---|---|
| 153 | Prevent stale installed-binary validation | open |
| 156 | Disposition remaining delta-review findings | ✅ done |
| 163 | Backfill missing IDs from filename prefixes | backlog |
| 164 | Support canonical slug IDs in validation | backlog |
| 165 | Write project-scoped configuration safely | backlog |
| 173 | Add advisory frontier Definition-of-Ready validation | backlog |
| 175 | Flag tickets with no source reference | backlog |
| 176 | Improve new-ticket body template and add body input | open |
| 177 | Finish ticket-content guidance after #176; evidence guidance is covered by #179 | open |
| 186 | Reconcile or retire the historical parity harness | backlog |
| 190 | Parse known config keys into strict typed policies | open |
| 206 | Grandfather pre-protocol corpus from audit missing-resolution (ADR 0003) | ✅ done |
| 191 | Centralize mutation effects and dry-run policy after #136 | backlog |

## v0.5.0 — agent experience and extensions

These are planned product improvements, deliberately not blockers for the v0.4.0
release of already-merged behavior.

| ID | Work | Status |
|---:|---|---|
| 127 | Improve agent discovery and init guidance | open |
| 138 | Complete the machine-readable capabilities manifest | open |
| 208 | capabilities -o json incomplete (consolidated into #138) | backlog |
| 161 | Support aggregate reporter provenance in steering | backlog |
| 168 | Consider advisory nudges for tag-less/backlog work | backlog |
| 79 | Safely undo the last tkt mutation | backlog |
| 83 | Edit a ticket body through the configured editor | backlog |
| 86 | One-way GitHub issue import | backlog |
| 87 | Ticket body templates | backlog |
| 88 | Recent-completions report | backlog |
| 89 | Lifecycle hooks | backlog |
| 90 | Open ticket-body URLs | backlog |
| 93 | Pluggable ticket schema and meta fields | backlog |
| 202 | Deploy tkt guidance and skill safely across supported harnesses | backlog |
| 207 | Retire crew steering surface: fold frontier-work into a skill/AGENTS.md | ✅ done |

## v0.5.0 — native relations and label namespaces

Close the structural gaps surfaced by the grey-matter skill-adaptation work by
adding native concepts (a `parent` relation, config-declared label namespaces, and
an `assignee` field) rather than emulating them with tag conventions and an external
lint script. All additive and backward compatible. Work in the order below: the
relation and label roots first (`192`, `193`), then `194` (reusing the completed
agent identity contract from `224`), their dependents, and the docs pass last.

| ID | Work | Status |
|---:|---|---|
| 192 | parent field: schema, parse, serialize | ✅ done |
| 193 | `[labels.*]` config: parse single-select + closed-value namespaces | open |
| 224 | Agent eligibility policy and invocation-scoped identity | ✅ done |
| 194 | assignee field + claim-by-assignment + frontier honors it | open |
| 195 | parent validation: dangling + cycle checks (after 192) | open |
| 196 | parent CLI + query/capabilities surface (after 192) | open |
| 197 | parent rollup + frontier semantics (after 192, 196) | open |
| 198 | label-namespace validation: single-select + closed-set (after 193) | open |
| 199 | docs, capabilities, CHANGELOG for parent/assignee/labels (after 194, 196-198) | open |
| 200 | parent-scoped ready frontier (after 196, 197) | open |

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
| 184 | Implement opt-in narrative sync-plan support | backlog |

ADR 0002 makes plan-as-projection the strategic primary direction; narrative support
is the opt-in secondary path in #184. #187 owns the primary projection renderer; keep
the two efforts independently schedulable.

| ID | Work | Status |
|---:|---|---|
| 187 | Generate a derived sync-plan projection from the ticket corpus | backlog |

## Current maintenance frontier

Work in order: correct shipped guidance, migrate repository knowledge, then address
automatic plan synchronization. This ordering follows the current session decision.

| ID | Work | Status |
|---:|---|---|
| 221 | Make shipped knowledge-path guidance convention-neutral | ✅ done |
| 222 | Adopt the current grey-matter repository layout | ✅ done |
| 223 | Keep plan status current during ticket mutations | ✅ done |

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
| 151 | Route corpus-ejection diagnostics through normal output policy | open |
| 157 | Add optional hook installation | backlog |
| 158 | Add per-agent ticket-write nudges | backlog |
| 159 | Document CI enforcement usage | backlog |

## Unplanned tickets

| ID | Work | Status |
|---:|---|---|
| 225 | Make upstream deployment validation enforce strict plugin and catalog checks | ✅ done |

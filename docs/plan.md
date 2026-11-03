# tkt delivery plan

Ticket files remain the source of truth. This table records intended sequencing and
explicit deferral; statuses are projections checked by `tkt sync-plan`.

Reconciled 2026-11-02 against checkout `62c8226` and the deployed matching binary.
See [full-board triage](agents/ticket-triage-2026-11-02.md) for per-ticket evidence,
readiness and unknowns. There is no active tag context. Dashboard work (#228 and #240) is in progress; rendered verification remains unavailable.

v0.4.0 is historical completed work (#146, local tag `v0.4.0`); the old prepublication
warning is retired. Later release readiness is governed by [release maintenance](releasing.md):
current authorization stops at reviewed staged candidates. This plan does not authorize
a new release or imply that all remaining work belongs to a promised version.

## Graph reading

The dashboard groups every unfinished ticket into one of six `stream:` workstreams.
`gate:decision` names an unresolved contract or policy choice; `gate:evidence` names
a required observation. The ticket body explains each gate under Planning gate.
Neither changes status or invents a dependency. Backlog remains deferred even when
prerequisites are met. The graph shows gates on dependency-blocked tickets too,
including the #197 parent-status decision upstream of #199 and #200.

## Current maintenance frontier

Discovery and the first integrity/configuration repairs are complete, including typed policies #190, editable tags #233, structured corpus diagnostics #151 and dist configuration #66. The guidance proposal #226 is complete; restructuring remains unapplied pending approval. Current parallel work is #165 (project config writes), #194 (assignee ownership) and #176 (creation body/templates). Each implementation requires independent critic acceptance and the local gate before closure. Dashboard rendering remains unavailable outside this queue. Sequencing does not invent dependency edges; the mechanical frontier still sorts priority then ID. Parent rollup #197 remains gated on its recorded decision.

| ID | Work | Status |
|---:|---|---|
| 138 | Update capabilities manifest with all commands | ✅ done |
| 127 | Improve agent discoverability: help nudge to capabilities, init recommends agent setup | ✅ done |
| 153 | Guard against stale-binary validation (prefer integration tests, add freshness check) | ✅ done |

## Post-v0.4.0 configuration and correctness

All rows here remain open investigations or implementations. `triage:needs-info` means the report names missing reproduction/design evidence; it is not a verified implementation brief. #140 is restored to open after its distinct malformed-input crash was reproduced. #165 owns writing configuration; #190 owns interpreting it.

| ID | Work | Status |
|---:|---|---|
| 140 | Fix exit code taxonomy leaks: config errors exit 2 instead of 1 | ✅ done |
| 143 | Fix -q + -o json emits bare non-JSON output inconsistently | open |
| 144 | Context silently narrows query (full corpus claim violated) | open |
| 145 | Fix update_check ignores DO_NOT_TRACK | open |
| 151 | load_corpus ejection is stderr-only, bypasses JSON envelope and TKT_ASCII/sym_warn | ✅ done |
| 163 | lint: backfill missing id from filename prefix | ✅ done |
| 164 | validate: recognize slug ids as canonical (stop false id-filename-mismatch) | ✅ done |
| 165 | config --set: support project scope (.tickets/config.toml), not just user | ✅ done |
| 190 | Parse known config keys into strict typed policies | ✅ done |
| 236 | tkt config shows the wrong user-config path on macOS | ✅ done |
| 237 | debug = true in config prints to stderr instead of the log file | ✅ done |
| 238 | Renumbering leaves stale ID references in ticket bodies | open |

## Agent workflow and guidance

#176 includes the named-template scope absorbed from #87. #177 waits on #176 and #233, while #230 owns classified evidence guidance. #226 is proposal-only; #228 owns the dashboard renderer migration. #235 extends existing push-disable and failure-reporting behavior rather than replacing it. #239 tracks historical audit limitations without fabricating proof.

| ID | Work | Status |
|---:|---|---|
| 176 | new: clearer body template + --body flag to fill content at creation | ✅ done |
| 177 | guidance: state evidence↔validation_criteria rule, tag editing, fill-body-at-new, link-the-source | open |
| 226 | Propose AGENTS.md restructuring while preserving guidance coverage | ✅ done |
| 228 | Migrate status dashboard to the current Plan-tab contract | in_progress |
| 229 | tkt-work close step cleans up the ticket's scratch and build artifacts | ✅ done |
| 230 | Separate artifact, functionality and user-outcome evidence in ticket closure guidance | ✅ done |
| 231 | tkt query outputs every frontmatter field, including tags | open |
| 232 | Frontier output distinguishes unfinished blockers from finished ones | open |
| 233 | tkt edit can set and clear tags | ✅ done |
| 234 | Acceptance criteria can be marked moved to another ticket | open |
| 235 | Ticket mutations skip heavy commit hooks and can defer the push | in_progress |
| 239 | Review historical closure warnings without fabricating evidence | ✅ done |
| 240 | Show workstreams and decision/evidence gates in the ticket graph | in_progress |
| 241 | Reject done status outside the close lifecycle | ✅ done |

## Native relations and label namespaces

Work from roots to dependents: #193 → #198; #195 → #196 → #197/#200; #194 reuses completed #224. #197 requires a decision about computed progress versus close-gated stored status before implementation; automatic done is not approved here. #199 is the final combined documentation/example pass. ADR 0004 already decides cancellation; #227 implements it and then retires the five pending tickets below.

| ID | Work | Status |
|---:|---|---|
| 193 | [labels.*] config: parse single-select + closed-value namespaces | ✅ done |
| 194 | assignee field + claim-by-assignment + frontier honors it | ✅ done |
| 195 | parent validation: dangling + cycle checks | ✅ done |
| 196 | parent CLI + query/capabilities surface | ✅ done |
| 197 | parent rollup + frontier semantics | open |
| 198 | label-namespace validation: single-select + closed-set | ✅ done |
| 199 | docs, capabilities, CHANGELOG for parent/assignee/labels | open |
| 200 | tkt ready --parent <id>: frontier scoped to one epic/map | open |
| 227 | Add a cancelled status and tkt cancel/reopen per ADR 0004 | in_progress |

## Lower-priority actionable tooling

#66 remains open rather than being silently demoted; use the pinned dist command and generator. #159 needs its CI prescription reconciled with the standing verification policy before implementation. No new hosted workflow is authorized by this plan.

| ID | Work | Status |
|---:|---|---|
| 66 | Migrate [workspace.metadata.dist] to dist-workspace.toml | ✅ done |
| 159 | Document CI recipe: tkt validate --strict / lint --check as the real enforcement gate | open |

## Intentionally deferred backlog

These are deliberately deferred, not release commitments. Triage notes distinguish specified enhancements from requests awaiting design, reproduction or a pinned external reference. #158 now records its real #157 config dependency. #173 must reuse #175 source-reference logic; #184 is optional narrative support and #187 is the independent primary projection direction under ADR 0002. #202 is limited to remaining binary-only skill delivery.

| ID | Work | Status |
|---:|---|---|
| 69 | Explore agent/harness-specific integrations (hooks, MCP, skills) | backlog |
| 79 | tkt undo: revert last mutation | backlog |
| 83 | tkt note <id>: open ticket body in $EDITOR | backlog |
| 86 | tkt import --github: pull open issues into .tickets/ | backlog |
| 88 | tkt done --week: show-resolved report for standups | backlog |
| 89 | Lifecycle hooks: on-new, on-claim, on-close scripts | backlog |
| 90 | tkt open <id>: find URLs in ticket body and open in browser | backlog |
| 93 | Pluggable schema definitions with meta field + ai_context support | backlog |
| 99 | Make update check interval configurable | backlog |
| 125 | Research and build import schemas for Jira, Linear, Beads, GitLab, and other ticket systems | backlog |
| 142 | Fix CREW_ENV=either hides everything and ac edge cases | backlog |
| 147 | Investigate telemetry consent resetting to disabled between sessions | backlog |
| 148 | Support primary-agent role pattern via init snippets and documentation | backlog |
| 157 | tkt hooks subcommand + pre-commit installer (warn-default, config block/log) | backlog |
| 158 | tkt init installs per-agent write-nudge for .tickets/ (Claude, Codex) | backlog |
| 168 | Optional tooling nudge: advisory on tag-less/backlog tickets | backlog |
| 173 | validate: frontier-scoped readiness rule (Definition of Ready gate) | backlog |
| 175 | validate: advisory no-source-reference finding (union of spec/#NN/parent/link) | backlog |
| 184 | Implement sync-plan narrative PLAN.md support (PlanFormat/PlanState) per ADR 0002 | backlog |
| 186 | Reconcile or retire the historical Python parity harness | backlog |
| 187 | Generate a derived sync-plan projection from the ticket corpus | backlog |
| 191 | Centralize command mutation effects and dry-run policy | backlog |
| 202 | Deploy tkt guidance and skill safely across supported harnesses | backlog |

## Pending cancellation under ADR 0004

Approved dispositions remain parked until #227 exists: #87 superseded by #176; #208 duplicate of #138; #115/#161 obsolete; #94 wontfix. Their new #227 dependency records the actual lifecycle gate. No acceptance box was checked and no ticket was falsely marked done. #69/#140/#142 are not blanket cancellations; #140 is now reproduced and actionable.

| ID | Work | Status |
|---:|---|---|
| 87 | tkt new --template: predefined ticket body structures | backlog |
| 94 | Test VC flag | backlog |
| 115 | Review recall telemetry and debug functionality for useful patterns | backlog |
| 161 | frontier-work: match aggregate/multi-reviewer Reporter, not just Reporter: Codex | backlog |
| 208 | tkt capabilities -o json is incomplete: omits flags (--tags, --vc, --requires) and commands (context, blocked, audit, ...) | backlog |

## Completed release, maintenance and decision context

Historical completed rows are retained for continuity. Their closure evidence is not
re-certified by this reconciliation; #239 records audit follow-up.

| ID | Work | Status |
|---:|---|---|
| 133 | Preserve unquoted YAML scalar bytes | ✅ done |
| 134 | Make renumber citation-safe and crash-consistent | ✅ done |
| 135 | Put JSON error envelope last on stderr | ✅ done |
| 136 | Apply global dry-run to every mutation | ✅ done |
| 137 | Resolve the real upstream branch for race detection | ✅ done |
| 139 | Repair evidence-gate severity and discarded-evidence behavior | ✅ done |
| 141 | Count uppercase checked acceptance boxes | ✅ done |
| 146 | Cut, tag, and document v0.4.0 | ✅ done |
| 150 | Make doctor strict results agree with validate | ✅ done |
| 152 | Fix indented-comment parser corruption | ✅ done |
| 156 | Disposition remaining delta-review findings | ✅ done |
| 160 | Decide and apply the legacy missing-resolution validation baseline | ✅ done |
| 181 | Verify atomic writes across Windows and Linux | ✅ done |
| 182 | Decide a delete/cancel lifecycle mechanism | ✅ done |
| 185 | Restore automatic push/PR CI verification | ✅ done |
| 188 | Reject ambiguous duplicate frontmatter keys | ✅ done |
| 189 | Unify env and requires into one eligibility model | ✅ done |
| 192 | parent field: schema, parse, serialize | ✅ done |
| 201 | Remediate agent-facing help and release metadata drift | ✅ done |
| 203 | decide v0.4.0 release scope before moving planned work to v0.5.0 | ✅ done |
| 204 | Parse single-item block-style dependencies correctly | ✅ done |
| 205 | Keep migration confirmation without writing during dry runs | ✅ done |
| 206 | Grandfather pre-protocol corpus from audit missing-resolution (ADR 0003) | ✅ done |
| 207 | Retire crew steering surface: fold frontier-work into a skill/AGENTS.md | ✅ done |
| 221 | Make shipped knowledge-path guidance convention-neutral | ✅ done |
| 222 | Adopt the current grey-matter repository layout | ✅ done |
| 223 | Keep plan status current during ticket mutations | ✅ done |
| 224 | Agent eligibility policy and invocation-scoped identity | ✅ done |
| 225 | Make upstream deployment validation enforce strict plugin and catalog checks | ✅ done |

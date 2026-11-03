# tkt delivery plan

Ticket files remain the source of truth. This table records intended sequencing and
explicit deferral; statuses are projections checked by `tkt sync-plan`.

Initially triaged 2026-11-02 at `62c8226`; reconciled again 2026-11-03 after
the unblocked maintenance queue. See [current evidence](evidence/frontier-maintenance-2026-11-03.md).
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
The owner settled all 17 published decisions on 2026-11-03; see [the decision record](decision-record-2026-11-03.md). #199 and #200 still require the actual #197 implementation.

## Current maintenance frontier

The newly requested Decisions tab is implemented under #243, with recommended
radio choices, notes and a chat export. Its model/DOM checks pass; rendered
laptop/phone acceptance remains unverified under the existing browser restriction.
LAN hosting is active under completed #244; see [the hosting guide](dashboard-lan.md)
for its address, served-file scope and supervised-process lifecycle.

The earlier maintenance batch closed 32 tickets and explicitly cancelled six obsolete items. All 17 owner decisions are now confirmed. Implementation remains open for #197 (computed progress with explicit closure), #234 (linked moved criteria), and #159 (optional consumer CI guidance). Guidance approval is staged as #245 → #246 → #247, following the completed proposal #226. The three dashboard tickets retain unverified rendered acceptance; 22 backlog items remain deliberately deferred. No implementation is claimed by recording these decisions.

| ID | Work | Status |
|---:|---|---|
| 242 | Audit completed templates by actual section content | ✅ done |
| 138 | Update capabilities manifest with all commands | ✅ done |
| 127 | Improve agent discoverability: help nudge to capabilities, init recommends agent setup | ✅ done |
| 153 | Guard against stale-binary validation (prefer integration tests, add freshness check) | ✅ done |

## Post-v0.4.0 configuration and correctness

Rows record completed repairs alongside remaining investigations and implementations. `triage:needs-info` means the report names missing reproduction/design evidence; it is not a verified implementation brief. The reproduced malformed-config crash in #140 is repaired. #165 owns writing configuration; #190 owns interpreting it.

| ID | Work | Status |
|---:|---|---|
| 140 | Fix exit code taxonomy leaks: config errors exit 2 instead of 1 | ✅ done |
| 143 | Fix -q + -o json emits bare non-JSON output inconsistently | ✅ done |
| 144 | Context silently narrows query (full corpus claim violated) | ✅ done |
| 145 | Fix update_check ignores DO_NOT_TRACK | ✅ done |
| 151 | load_corpus ejection is stderr-only, bypasses JSON envelope and TKT_ASCII/sym_warn | ✅ done |
| 163 | lint: backfill missing id from filename prefix | ✅ done |
| 164 | validate: recognize slug ids as canonical (stop false id-filename-mismatch) | ✅ done |
| 165 | config --set: support project scope (.tickets/config.toml), not just user | ✅ done |
| 190 | Parse known config keys into strict typed policies | ✅ done |
| 236 | tkt config shows the wrong user-config path on macOS | ✅ done |
| 237 | debug = true in config prints to stderr instead of the log file | ✅ done |
| 238 | Renumbering leaves stale ID references in ticket bodies | ✅ done |

## Agent workflow and guidance

#176 includes the named-template scope absorbed from cancelled #87. #177 and #230 complete the content and classified evidence guidance. #226 is proposal-only; #228 owns the dashboard renderer migration. #235 extends existing push-disable and failure-reporting behavior. #239 records historical audit limitations without fabricating proof.

| ID | Work | Status |
|---:|---|---|
| 176 | new: clearer body template + --body flag to fill content at creation | ✅ done |
| 177 | guidance: state evidence↔validation_criteria rule, tag editing, fill-body-at-new, link-the-source | ✅ done |
| 226 | Propose AGENTS.md restructuring while preserving guidance coverage | ✅ done |
| 245 | Prepare approved guidance reference homes and current coverage map | open |
| 246 | Apply approved conditional guidance pointers and update coverage matrix | open |
| 247 | Verify and deploy the staged guidance restructuring | open |
| 228 | Migrate status dashboard to the current Plan-tab contract | in_progress |
| 229 | tkt-work close step cleans up the ticket's scratch and build artifacts | ✅ done |
| 230 | Separate artifact, functionality and user-outcome evidence in ticket closure guidance | ✅ done |
| 231 | tkt query outputs every frontmatter field, including tags | ✅ done |
| 232 | Frontier output distinguishes unfinished blockers from finished ones | ✅ done |
| 233 | tkt edit can set and clear tags | ✅ done |
| 234 | Acceptance criteria can be marked moved to another ticket | open |
| 235 | Ticket mutations skip heavy commit hooks and can defer the push | ✅ done |
| 239 | Review historical closure warnings without fabricating evidence | ✅ done |
| 240 | Show workstreams and decision/evidence gates in the ticket graph | in_progress |
| 241 | Reject done status outside the close lifecycle | ✅ done |

## Native relations and label namespaces

Work from roots to dependents: #193 → #198; #195 → #196 → #197/#200; #194 reuses completed #224. #197 now implements the approved computed-progress contract with explicit close-gated stored status; automatic done is excluded. #199 is the final combined documentation/example pass. ADR 0004 cancellation is implemented by #227; obsolete items below are explicitly cancelled.

| ID | Work | Status |
|---:|---|---|
| 193 | [labels.*] config: parse single-select + closed-value namespaces | ✅ done |
| 194 | assignee field + claim-by-assignment + frontier honors it | ✅ done |
| 195 | parent validation: dangling + cycle checks | ✅ done |
| 196 | parent CLI + query/capabilities surface | ✅ done |
| 197 | Show computed parent progress with explicit closure | in_progress |
| 198 | label-namespace validation: single-select + closed-set | ✅ done |
| 199 | docs, capabilities, CHANGELOG for parent/assignee/labels | open |
| 200 | tkt ready --parent <id>: frontier scoped to one epic/map | open |
| 227 | Add a cancelled status and tkt cancel/reopen per ADR 0004 | ✅ done |

## Tooling and verification decisions

#66 is complete with the pinned dist configuration migration verified. #159 has approval for optional consumer CI guidance with explicit limits and the standing local verification policy preserved. No new hosted workflow is authorized by this plan.

| ID | Work | Status |
|---:|---|---|
| 66 | Migrate [workspace.metadata.dist] to dist-workspace.toml | ✅ done |
| 159 | Document optional consumer CI with accurate enforcement limits | open |

## Intentionally deferred backlog

These are deliberately deferred, not release commitments. Triage notes distinguish specified enhancements from requests awaiting design, reproduction or a pinned external reference. #158 now records its real #157 config dependency. #173 must reuse #175 source-reference logic; #184 is optional narrative support and #187 is the independent primary projection direction under ADR 0002. #202 is limited to remaining binary-only skill delivery.

| ID | Work | Status |
|---:|---|---|
| 69 | Define the focused remaining harness-integration scope | backlog |
| 79 | tkt undo: revert last mutation | backlog |
| 83 | Add a noninteractive ticket-note append operation | backlog |
| 86 | tkt import --github: pull open issues into .tickets/ | backlog |
| 88 | tkt done --week: show-resolved report for standups | backlog |
| 89 | Lifecycle hooks: on-new, on-claim, on-close scripts | backlog |
| 90 | tkt open <id>: find URLs in ticket body and open in browser | backlog |
| 93 | Pluggable schema definitions with meta field + ai_context support | backlog |
| 99 | Make update check interval configurable | backlog |
| 125 | Design the first external-system import adapter contract | backlog |
| 147 | Investigate telemetry consent resetting to disabled between sessions | backlog |
| 148 | Support primary-agent role pattern via init snippets and documentation | backlog |
| 157 | tkt hooks subcommand + pre-commit installer (warn-default, config block/log) | backlog |
| 158 | tkt init installs per-agent write-nudge for .tickets/ (Claude, Codex) | backlog |
| 168 | Measure demand for missing-tag and backlog advisories | backlog |
| 173 | validate: frontier-scoped readiness rule (Definition of Ready gate) | backlog |
| 175 | validate: advisory no-source-reference finding (union of spec/#NN/parent/link) | backlog |
| 184 | Implement sync-plan narrative PLAN.md support (PlanFormat/PlanState) per ADR 0002 | backlog |
| 186 | Reconcile or retire the historical Python parity harness | backlog |
| 187 | Generate a derived sync-plan projection from the ticket corpus | backlog |
| 191 | Centralize command mutation effects and dry-run policy | backlog |
| 202 | Design versioned embedded skills for binary-only installs | backlog |

## Recorded cancellations under ADR 0004

The lifecycle is implemented and six dispositions are recorded: #87 superseded by #176; #208 duplicates #138; #115/#161 are obsolete; #94 and #142 are wontfix. Cancellation checked no acceptance boxes and claimed no completed implementation. #142's isolated capability probe did not reproduce the report; its historical unspecified edge cases remain unknown. #140 is fixed and complete. #69 stays deferred with an approved focused follow-up scope.

| ID | Work | Status |
|---:|---|---|
| 87 | tkt new --template: predefined ticket body structures | cancelled (superseded) |
| 94 | Test VC flag | cancelled (wontfix) |
| 115 | Review recall telemetry and debug functionality for useful patterns | cancelled (obsolete) |
| 142 | Fix CREW_ENV=either hides everything and ac edge cases | cancelled (wontfix) |
| 161 | frontier-work: match aggregate/multi-reviewer Reporter, not just Reporter: Codex | cancelled (obsolete) |
| 208 | tkt capabilities -o json is incomplete: omits flags (--tags, --vc, --requires) and commands (context, blocked, audit, ...) | cancelled (duplicate) |

## Completed release, maintenance and decision context

Historical completed rows are retained for continuity. Their closure evidence is not
re-certified by this reconciliation; #239 records audit follow-up.

| ID | Work | Status |
|---:|---|---|
| 133 | Fix yaml_scalar_unescape corrupting plain (unquoted) scalars | ✅ done |
| 134 | Fix renumber: enforce birth-window citation scan before apply | ✅ done |
| 135 | Fix JSON error envelope not last line of stderr | ✅ done |
| 136 | Fix global --dry-run ignored by 7 mutating commands | ✅ done |
| 137 | Fix origin/main hardcoded in race detection | ✅ done |
| 139 | Fix evidence gates partially dead: unreachable warn branch + silent discard | ✅ done |
| 141 | Fix uppercase X checkboxes invisible to AC stats | ✅ done |
| 146 | Release v0.4.0: ship post-v0.3.0 workflow changes safely | ✅ done |
| 150 | doctor must report all validate findings | ✅ done |
| 152 | Review 2026-08-27: fmt gate red at HEAD, indented-comment corruption, #149 dispositions pending | ✅ done |
| 156 | Delta review 2026-08-27: #154 legacy-baseline gap, cross-project evidence, #152 still pending | ✅ done |
| 160 | Decide handling for 27 historical done tickets flagged missing-resolution | ✅ done |
| 181 | Verify atomic_write on Windows + Linux (fs::rename overwrite, temp cleanup, worktrees) | ✅ done |
| 182 | Decide whether tkt needs a delete/cancel operation — and what it should do | ✅ done |
| 185 | Restore automated CI gate for pushes and pull requests | ✅ done |
| 188 | Reject ambiguous duplicate frontmatter keys | ✅ done |
| 189 | Unify env and requires into one ticket eligibility model | ✅ done |
| 192 | parent field: schema, parse, serialize | ✅ done |
| 201 | Remediate small agent-facing help and release metadata drift | ✅ done |
| 203 | Decide v0.4.0 release scope before moving planned work to v0.5.0 | ✅ done |
| 204 | Parse single-item block-style blocked_by lists correctly | ✅ done |
| 205 | Keep migration confirmation without writing during dry runs | ✅ done |
| 206 | audit: grandfather pre-protocol corpus from missing-resolution (ADR 0003) | ✅ done |
| 207 | Retire crew-research steering surface: fold frontier-work into a skill/AGENTS.md | ✅ done |
| 221 | Shipped tkt-core guidance prescribes docs/adr and docs/agents/specs, contradicting grey-matter's no-.memory convention | ✅ done |
| 222 | Adopt grey-matter repo layout: relocate .memory/ to GLOSSARY.md, docs/adr/, docs/agents/ | ✅ done |
| 223 | close/claim leave docs/plan.md behind, so sync-plan --check fails after every close | ✅ done |
| 224 | Let tickets declare which agents may pick them up | ✅ done |
| 225 | Make upstream deployment validation enforce strict plugin and catalog checks | ✅ done |

## Unplanned tickets

| ID | Work | Status |
|---:|---|---|
| 243 | Collect dashboard decisions with recommendations and chat export | in_progress |
| 244 | Serve the delivery dashboard on the local network | ✅ done |

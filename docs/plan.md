# tkt delivery plan

Ticket files remain the source of truth. This table records intended sequencing and
explicit deferral; statuses are projections checked by `tkt sync-plan`.

Initially triaged 2026-11-02 at `62c8226`; reconciled again 2026-11-03 after
the unblocked maintenance queue. See [current evidence](evidence/frontier-maintenance-2026-11-03.md).
See [the parked-work execution receipt](evidence/parked-execution-2026-11-03.md) for current results, deployment and limits. The earlier [guidance deployment receipt](evidence/guidance-deployment-247.md) is historical.
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
prerequisites are met. The graph shows gates on dependency-blocked tickets too.
The owner settled all 17 published decisions on 2026-11-03; see [the decision record](decision-record-2026-11-03.md). The completed #197 implementation unblocks #199 and #200.

The actionable implementation queue, including parent #191, is complete. Historical batch results are in [the execution receipt](evidence/unblocked-batch-2026-11-03.json); current results and limits are in [the final dispatcher receipt](evidence/git-effects-261.md) and the execution queue below.

## Cross-project production evidence

A subsequent [78-project review](evidence/cross-project-review-2026-11-03.md) found no verified tkt regression requiring terminal-ticket reopening. #147 still lacks a consent-reset reproduction; #184 remains deferred. Existing #168 now has a measured corpus baseline and #86 has corroborating downstream demand. Working-copy warnings are distinguished from product defects and stale local checkouts from current upstream guidance.

## Current maintenance frontier

The newly requested Decisions tab is implemented under #243, with recommended
radio choices, notes and a chat export. Its model/DOM checks pass; rendered
laptop/phone acceptance remains unverified under the existing browser restriction.
LAN hosting is active under completed #244; see [the hosting guide](dashboard-lan.md)
for its address, served-file scope and supervised-process lifecycle.

The earlier maintenance batch closed 32 tickets and cancelled six obsolete items. This execution batch completed #197, #234, #199, #200, #159, #248 and the approved guidance stages #245–#247, each after independent critic acceptance. The parked-work review reactivated 21 tickets; two remain parked for explicit deferral or missing evidence. Three dashboard tickets retain unavailable rendered acceptance. See the current execution queue below. The CLI and workflow pack are deployed; optional Kiro Crew activation is unavailable without its CLI. The wider inventory audit still reports unrelated unowned/legacy Kiro roots, which were preserved.

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
| 245 | Prepare approved guidance reference homes and current coverage map | ✅ done |
| 246 | Apply approved conditional guidance pointers and update coverage matrix | ✅ done |
| 247 | Verify and deploy the staged guidance restructuring | ✅ done |
| 228 | Migrate status dashboard to the current Plan-tab contract | in_progress |
| 229 | tkt-work close step cleans up the ticket's scratch and build artifacts | ✅ done |
| 230 | Separate artifact, functionality and user-outcome evidence in ticket closure guidance | ✅ done |
| 231 | tkt query outputs every frontmatter field, including tags | ✅ done |
| 232 | Frontier output distinguishes unfinished blockers from finished ones | ✅ done |
| 233 | tkt edit can set and clear tags | ✅ done |
| 234 | Acceptance criteria can be marked moved to another ticket | ✅ done |
| 235 | Ticket mutations skip heavy commit hooks and can defer the push | ✅ done |
| 239 | Review historical closure warnings without fabricating evidence | ✅ done |
| 240 | Show workstreams and decision/evidence gates in the ticket graph | in_progress |
| 241 | Reject done status outside the close lifecycle | ✅ done |

## Native relations and label namespaces

Work from roots to dependents: #193 → #198; #195 → #196 → #197/#200; #194 reuses completed #224. #197 now implements the approved computed-progress contract with explicit close-gated stored status; automatic done is excluded. #199 completed the combined documentation/example pass. ADR 0004 cancellation is implemented by #227; obsolete items below are explicitly cancelled.

| ID | Work | Status |
|---:|---|---|
| 193 | [labels.*] config: parse single-select + closed-value namespaces | ✅ done |
| 194 | assignee field + claim-by-assignment + frontier honors it | ✅ done |
| 195 | parent validation: dangling + cycle checks | ✅ done |
| 196 | parent CLI + query/capabilities surface | ✅ done |
| 197 | Show computed parent progress with explicit closure | ✅ done |
| 198 | label-namespace validation: single-select + closed-set | ✅ done |
| 199 | docs, capabilities, CHANGELOG for parent/assignee/labels | ✅ done |
| 200 | tkt ready --parent <id>: frontier scoped to one epic/map | ✅ done |
| 227 | Add a cancelled status and tkt cancel/reopen per ADR 0004 | ✅ done |

## Tooling and verification decisions

#66 is complete with the pinned dist configuration migration verified. #159 documents optional consumer CI guidance with explicit limits and the standing local verification policy preserved. The owner removed automatic CI; #249 tracks an optional local pre-push convenience, with no hosted-CI equivalence. No new hosted workflow is authorized by this plan.

| ID | Work | Status |
|---:|---|---|
| 66 | Migrate [workspace.metadata.dist] to dist-workspace.toml | ✅ done |
| 159 | Document optional consumer CI with accurate enforcement limits | ✅ done |

## Reactivated work and remaining deferrals

The owner requested execution of actionable parked work. The [per-ticket review](parked-review-2026-11-03.md) records the dispositions. The implementation slices and their prerequisite designs are now complete after independent critic acceptance, including #191 and children #259–#261. Optional hooks remain opt-in; #93 remains design-only and #168 added no runtime nudge. Actual prerequisites remain in the ticket graph; this plan adds no scheduling dependencies. The remaining policy/rendering gates and backlog dispositions are listed in the current execution queue below.

| ID | Work | Status |
|---:|---|---|
| 69 | Define the focused remaining harness-integration scope | ✅ done |
| 79 | tkt undo: revert last mutation | ✅ done |
| 83 | Add a noninteractive ticket-note append operation | ✅ done |
| 86 | tkt import --github: pull open issues into .tickets/ | ✅ done |
| 88 | tkt done --week: show-resolved report for standups | ✅ done |
| 89 | Lifecycle hooks: on-new, on-claim, on-close scripts | ✅ done |
| 90 | tkt open <id>: find URLs in ticket body and open in browser | in_progress |
| 93 | Pluggable schema definitions with meta field + ai_context support | ✅ done |
| 99 | Make update check interval configurable | ✅ done |
| 125 | Design the first external-system import adapter contract | ✅ done |
| 147 | Investigate telemetry consent resetting to disabled between sessions | backlog |
| 148 | Support primary-agent role pattern via init snippets and documentation | ✅ done |
| 157 | Optional advisory project pre-commit hooks | ✅ done |
| 158 | Generate optional advisory native write nudges | ✅ done |
| 168 | Measure demand for missing-tag and backlog advisories | ✅ done |
| 173 | validate: frontier-scoped readiness rule (Definition of Ready gate) | ✅ done |
| 175 | validate: advisory no-source-reference finding (union of spec/#NN/parent/link) | ✅ done |
| 184 | Implement sync-plan narrative PLAN.md support (PlanFormat/PlanState) per ADR 0002 | backlog |
| 186 | Reconcile or retire the historical Python parity harness | ✅ done |
| 187 | Generate a derived sync-plan projection from the ticket corpus | ✅ done |
| 191 | Centralize command mutation effects and dry-run policy | ✅ done |
| 202 | Design versioned embedded skills for binary-only installs | ✅ done |

### Current execution queue

Fresh review of all six remaining tickets: [reproduction results and rendering checklist](evidence/remaining-six-review-2026-11-04.md). Private current-reader controls did not reproduce a persistent consent reset; session overrides and alternate roots explain disabled state without file changes. No gate was removed on inference. Browser inventory still denies local browser use; no fallback was attempted. Narrative support remains owner-deferred; its separate status-cell repair was delivered in #248.

The remaining implementation queue is complete: #261 and parent #191 closed after the fresh 523-Rust/42-Node gate and independent critic acceptance ([receipt](evidence/git-effects-261.md)). The installed CLI is `e755765`; workflow deployment is recorded in the receipt. The earlier #259 test-isolation incident remains documented with prior consent/data state unknown ([receipt](evidence/telemetry-preview-259.md)). No open ready ticket remains. #90 is gated by automatic approval review rejecting the launcher implementation as blocked by policy. Three dashboard tickets (#228, #240, #243) retain unavailable rendered acceptance. Two backlog tickets remain: #147 needs its exact cross-session reproduction; #184 is explicitly deferred. This queue adds no dependency edges.

## Recorded cancellations under ADR 0004

The lifecycle is implemented and six dispositions are recorded: #87 superseded by #176; #208 duplicates #138; #115/#161 are obsolete; #94 and #142 are wontfix. Cancellation checked no acceptance boxes and claimed no completed implementation. #142's isolated capability probe did not reproduce the report; its historical unspecified edge cases remain unknown. #140 is fixed and complete. #69 completed its delivered-integration inventory and focused follow-up scope.

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
| 248 | Compare plan lifecycle labels precisely without narrative parsing | ✅ done |
| 249 | Add optional local pre-push verification | ✅ done |
| 250 | Prevent debug logging from writing during dry-run | ✅ done |
| 251 | Preserve migration sources and foreign destinations | ✅ done |
| 252 | Remove unsupported migration-map guidance | ✅ done |
| 253 | Plan and apply local configuration context and init effects | ✅ done |
| 254 | Restrict undo close checkbox recognition to the acceptance section | ✅ done |
| 255 | Keep import guidance within native skill loading limits | ✅ done |

Guidance delivery repair #255 is complete after critic acceptance: the core entry routes detailed import instructions to a standalone reference within the observed native body limit, and all six active/staged targets match ([receipt](evidence/import-guidance-255.json)).

Maintenance integration: upstream #256–#258 fixture repairs are preserved. The Windows follow-up to #256 was reproduced, repaired and accepted by the critic after a fresh full gate; the newly integrated undo fixture passed nine focused tests. The reporting CLI was installed and verified fresh at `c15919f`.

Latest 2026-11-04 delivery: #261 and parent #191 are complete after critic acceptance and the fresh full gate. CLI source `e755765` is installed; workflow refresh passed (KiroCrew built only). The plan and dashboard retain four gated WIP tickets and two backlog items. See [the final execution receipt](evidence/git-effects-261.md) for verification, deployment, artifact disposition and limits.
| 259 | Prevent telemetry commands from mutating during dry-run | ✅ done |
| 260 | Route remaining filesystem mutation families through typed effect phases | ✅ done |
| 261 | Route Git publication and process phases through the common effect dispatcher | ✅ done |
| 264 | tkt new -o json returns id and path; edit --ac help explains its argument | open |
| 268 | Fix the history newline comparison that blocks Clippy | ✅ done |
| 269 | Measure tkt Kimi plugin update behavior | open |
| 270 | Validate tkt Kimi installation beyond macOS | open |
| 271 | tkt edit --body to replace a ticket body | open |
| 274 | Fix pre-push entry-point detection across canonicalized paths | open |
| 275 | Prevent done reports from triggering Git maintenance | cancelled (duplicate) |
| 276 | Normalize macOS temporary paths in import preview test | cancelled (duplicate) |

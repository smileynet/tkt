# Full-board triage — 2026-11-02

## Scope and evidence boundary

Owner request: “Deep dive, reconcile plan, make sure all tickets are triaged and current.”
Reviewed all 65 initially unfinished ticket bodies, their current code/CLI surfaces,
ADRs 0002–0004 and the existing plan; added #239 for historical audit follow-up.
The resulting corpus has 239 tickets: 173 done, 38 open, 28 backlog, no in-progress.
All 66 unfinished tickets have a work stream, exactly one `kind:` tag, exactly one
`triage:` tag, a dated assessment and validation criteria. Completed tickets were
checked by strict validation and the deep closure audit; their historical product
behavior was not re-tested individually and their closure records were not rewritten.

Baseline: clean main `62c8226`, fetched origin/main with 0/0 divergence. Installed
`tkt 0.4.0 (62c8226)` matched that baseline during the behavioral probes. Later
documentation commits change HEAD; no product code changed in this reconciliation.
No context was active. No external issues, pull requests, third-party accounts,
macOS runtime or current harness API documentation were audited. Historical claims
about 0.5.0 are dated snapshots, not the version of this checkout.

`docs/agents/tkt-conventions.md` was absent, so this reconciliation records the
triage vocabulary there instead of assuming it existed. No skill or AGENTS.md
was changed. Negative/terminal dispositions below were already approved in ADR 0004
and prior ticket notes; this session does not close or cancel them.

## Reproduction observations

Probes used small newly created local Git repositories with no remote, explicit
`[push] enabled=false`, isolated `XDG_CONFIG_HOME`/`XDG_STATE_HOME`,
`DO_NOT_TRACK=1`, `TKT_UPDATE_CHECK=0` and `TKT_NO_USER_CONFIG=1` except the deliberate
config-only debug case. These were install/CLI contract probes, not a Rust test-suite run.
Two initial probe groups lacked Git initialization and returned not-a-git-repository;
those runs are **infrastructure errors**, excluded from the findings below. After
initializing the fixtures, the commands produced the observations recorded here.

| Ticket | Command / input | Observed output | Classification |
|---:|---|---|---|
| 138 | `tkt capabilities` | exit 0; commands: batch, capabilities, claim, close, config, edit, lint, new, query, ready, validate | Confirmed incomplete inventory; config is already included |
| 140 | `tkt config --set malformed` | exit 2; `crash: expected key=value format, got "malformed"` | Confirmed input error misclassified as crash |
| 143 | `tkt new quiet-json --title "Quiet JSON" -q -o json` | exit 0; stdout `02` | Confirmed absent structured object; not every bare numeric ID is invalid JSON syntax |
| 144 | `TKT_CONTEXT=other tkt query` with ink-tagged 01 and untagged 02 | exit 0; only 02 emitted; stderr empty | Confirmed silent filtering |
| 151 | `TKT_ASCII=1 tkt -o json query` on ticket lacking id | exit 0; empty stdout; stderr starts `⚠ skipping 01-broken.md` | Confirmed unstructured Unicode diagnostic |
| 163 | bare `tkt lint` on 01-no-id.md | exit 0; `1 file(s) skipped (parse errors)`; missing id remains absent | Confirmed missing backfill, not a passing normalization |
| 163 | `tkt lint --fix` | exit 2; unexpected argument --fix | Current interface is bare lint; do not assume the proposed flag exists |
| 164 | `tkt validate --brief` on slug-key.md with id slug-key | exit 1; id-filename-mismatch | Confirmed reported mismatch |
| 165 | `tkt config --project --set close.require_resolution=true` | exit 2; unexpected --project | Confirmed missing requested interface |
| 190 | `push.enabled="treu"`; `tkt config --show` | exit 0; `push.enabled = "false" (project)` | Confirmed silently weakened policy |
| 195 | sole ticket has `parent: "999"`; `tkt validate --brief` | exit 0; pass (0 findings) | Confirmed missing dangling-parent finding |
| 231 | `tkt query` on stored tags, requires and custom fields | exit 0; only id/title/status/blocked_by/agents emitted | Confirmed omitted fields; agents already works |
| 232 | done 01, open 02 blocked_by 01; `tkt ready --json` | exit 0; 02 retains blocked_by [01], lacks pending_blockers | Confirmed JSON gap; historical edges must remain |
| 232 | same corpus; `tkt ready` | `Ready (1):` then `02 Open`, no blocker list | Human behavior already meets this portion |
| 236 | isolated XDG_CONFIG_HOME; `tkt config` | labels user config `~/.config/tkt/config.toml` | Confirmed misleading header; loader path inspected in src/config.rs; macOS execution unavailable |
| 237 | isolated user `debug=true`, no TKT_DEBUG/debug.format; `tkt query` | exit 0; two `[tkt:debug]` stderr lines | Confirmed config-only default mismatch |
| 238 | body cites nonexistent #999 and ticket 998; `tkt validate --brief` | exit 0; pass (0 findings) | Confirmed missing reference diagnostic only; collision-renumber reproduction still needed |
| 125 / 148 | `tkt migrate --map example.toml`; `tkt init --role observer` | each exits 2 with unexpected argument | Proposed interfaces do not exist yet |

Help observations also show no edit tag flags, moved-AC flags, new body/template
flags or no-push flag. Source inspection: `src/git.rs::commit` runs `git commit -qm`
without bypassing hooks; `src/mutation.rs` already preserves a local commit and
reports push failure; `src/transaction.rs` already respects push.enabled. #235 must
extend those behaviors, not claim they are all missing. The failing/slow-hook and
collision fixtures were not executed here. `src/update_check.rs` lacks a DO_NOT_TRACK
guard, but #145 remains needs-info until an isolated request/cache probe exists.

## Decisions and ordering corrections

- #140 is reproduced, distinct from strict value parsing in #190, and is now open.
- #158 now depends on #157's shared hook configuration. #196 now depends on #195's
  parent integrity rules as well as the already-completed #192 representation.
- #177 now waits for #233 as well as #176, no longer calls missing tag editing
  intentional, and delegates classified evidence guidance to #230.
- #176 explicitly includes the named-template AC absorbed from #87.
- #197 needs a decision: computed parent progress must not silently bypass
  `tkt close` evidence gates or rewrite stored status as done.
- #159's automatic-CI prescription conflicts with the current standing manual-only
  policy. It needs a scoped optional-consumer proposal, not a new hosted workflow.
- #87/#94/#115/#161/#208 retain backlog status and now record #227 as the actual
  cancellation prerequisite. No unfulfilled acceptance criteria were checked.
- The old deferred section mixed open work and backlog. Open work stays actionable,
  with low-priority items identified separately; no bulk demotion was used to hide it.
- Plan queue: #138 → #127, then #153. Other streams are grouped with real blockers;
  no scheduling preference was encoded as a fake dependency.

## Per-ticket disposition

“Ready-for-agent” means the brief is actionable once its real blockers are done;
backlog still means intentionally deferred. “Needs-info” can name reproduction
work an agent can gather, or a design choice requiring owner input. It does not
automatically imply an implementation-ready ticket. `tkt ready` is a structural
frontier and does not currently filter triage tags.

Readiness: 21 needs-info, 40 ready-for-agent, 5 wontfix.

| ID | Kind / triage | Stream | Current assessment and next action |
|---:|---|---|---|
| [66](../../.tickets/66-dist-config-migration.md) | enhancement / ready-for-agent | tooling | Cargo.toml still owns workspace.metadata.dist; use dist (0.32.0), not cargo dist. Preserve generated staging workflow and generator checks. |
| [69](../../.tickets/69-agent-integrations.md) | enhancement / needs-info | integrations | Skills and plugins exist; the remaining deliverable is a recorded integration decision, not a second plugin implementation. Confirm which historical ACs are satisfied before closing or cancelling. |
| [79](../../.tickets/79-undo-command.md) | enhancement / ready-for-agent | cli | Deferred feature; src/cli.rs has no undo command. Match actual chore(tickets) commit messages, refuse unrelated HEAD, and preserve push gating. |
| [83](../../.tickets/83-note-command.md) | enhancement / needs-info | cli | Deferred proposal combines a noncommitting editor with a committing append form. Decide the mutation/commit contract and noninteractive input before implementation. |
| [86](../../.tickets/86-github-import.md) | enhancement / ready-for-agent | integrations | Deferred one-way import. src/cli.rs has migrate but no import command; preserve source issue identity and bounded allocation. No external service changes are authorized by triage. |
| [87](../../.tickets/87-templates.md) | enhancement / wontfix | cli | Approved supersession by #176; await #227 cancellation with --by 176. Do not implement twice or check unfulfilled ACs. |
| [88](../../.tickets/88-done-report.md) | enhancement / needs-info | cli | Deferred reporting feature. Decide authoritative completion timestamp and behavior when git history is unavailable; mtime is not proof of completion time. |
| [89](../../.tickets/89-hooks.md) | enhancement / needs-info | integrations | Deferred lifecycle hooks, distinct from git-hook bypass in #235 and installer #157. Resolve the contradictory timeout/nonblocking-push contract and supported Windows invocation. |
| [90](../../.tickets/90-open-command.md) | enhancement / ready-for-agent | cli | Deferred URL helper is absent from src/cli.rs. --list must list without launching; launch only the explicit selected/default URL and handle Windows without shell interpolation. |
| [93](../../.tickets/93-pluggable-schema.md) | enhancement / needs-info | schema | Deferred design proposal, not an implementation brief: runtime CLI fields, schema migration and nested meta exceed the current narrow parser. Decide supported representation before building. |
| [94](../../.tickets/94-test-vc-flag.md) | enhancement / wontfix | quality | Approved retirement of an empty test ticket; await #227 cancellation with reason wontfix. Preserve history rather than manufacturing completed checks. |
| [99](../../.tickets/99-configurable-update-check.md) | enhancement / ready-for-agent | config | Deferred enhancement. src/update_check.rs still hardcodes CHECK_INTERVAL_SECS=86400; define duration validation and env precedence without changing default behavior. |
| [115](../../.tickets/115-recall-telemetry-review.md) | enhancement / wontfix | integrations | Approved obsolete recall integration; await #227 cancellation. Retired consumer is not a current implementation target. |
| [125](../../.tickets/125-import-schema-research.md) | enhancement / needs-info | integrations | Deferred research depends on an unimplemented --map interface: migrate --map example.toml exits 2. First choose adapter/mapping contract; no third-party schema facts were verified in this audit. |
| [127](../../.tickets/127-agent-onboarding.md) | enhancement / ready-for-agent | guidance | Keep real dependency #138. Replace the cargo-install-includes-skills example: Cargo.toml currently excludes skills. Describe plugin installation separately from the binary. |
| [138](../../.tickets/138-fix-capabilities-manifest.md) | bug / ready-for-agent | tooling | Confirmed: capabilities lists 11 commands including config, but omits context/blocked/audit/sync-plan/renumber/init/doctor/migrate/rebase/telemetry. Own command/flag completeness; #208 is its duplicate. |
| [140](../../.tickets/140-fix-exit-code-taxonomy.md) | bug / ready-for-agent | config | Confirmed independently of #190: config --set malformed exits 2 with crash: expected key=value format. Restore input-validation exit 1 and structured Validation error; reopen as actionable work. |
| [142](../../.tickets/142-fix-crew-env-either.md) | bug / needs-info | cli | Earlier no-reproduction result remains unresolved. Supply a minimal CREW_ENV=either fixture and expected capability semantics; the unspecified AC edge cases are not build scope. |
| [143](../../.tickets/143-fix-quiet-json-inconsistency.md) | bug / ready-for-agent | cli | Confirmed: new quiet-json --title Quiet JSON -q -o json prints bare 02. Require the structured success envelope. IDs without leading zero can parse as JSON numbers, so syntax-only testing is insufficient. |
| [144](../../.tickets/144-fix-context-narrows-query.md) | bug / ready-for-agent | cli | Confirmed: TKT_CONTEXT=other query hides an ink-tagged ticket and retains an untagged ticket, with no stderr notice. Keep existing scoped default; add/document bypass. |
| [145](../../.tickets/145-fix-update-check-do-not-track.md) | bug / needs-info | cli | Source inspection confirms update_check.rs has no DO_NOT_TRACK guard. Network/cache behavior was not dynamically reproduced; capture an isolated curl/cache fixture before implementing. Existing privacy request remains actionable investigation. |
| [147](../../.tickets/147-telemetry-consent-resets.md) | bug / needs-info | config | No platform/config/consent reproduction is recorded. Deferred investigation; do not change consent precedence speculatively or assert ambient consent state. |
| [148](../../.tickets/148-primary-agent-pattern.md) | enhancement / needs-info | guidance | init --role observer exits 2 (missing feature). The old use of machine capabilities for agent scheduling conflicts with #224's separate identity/eligibility contract. Redesign examples before implementation. |
| [151](../../.tickets/151-load-corpus-diagnostics.md) | bug / ready-for-agent | parser | Confirmed: TKT_ASCII=1 tkt -o json query on a ticket lacking id exits 0 with empty stdout and a Unicode warning on stderr. Preserve valid-corpus behavior while exposing skipped-ticket diagnostics. |
| [153](../../.tickets/153-stale-binary-guard.md) | bug / ready-for-agent | tooling | Confirmed this session: installed hash 92d4833 differed from HEAD 62c8226 and --agent failed until deployment. The freshness guard is absent; use integration binaries for product checks and compare hashes only when install/deploy itself is under test. |
| [157](../../.tickets/157-hooks-subcommand.md) | enhancement / needs-info | integrations | Deferred opt-in product hook installer. Reconcile with #235 skipping git hooks and standing guidance against installing verification hooks; do not silently install hooks during this triage. Historical scratch research is not a dependency. |
| [158](../../.tickets/158-init-agent-write-nudge.md) | enhancement / needs-info | integrations | Add missing dependency #157 for its shared hook config. Harness hook claims are version-sensitive and unverified; current Codex plugin explicitly installs no hooks. Verify official support before implementation. |
| [159](../../.tickets/159-ci-gate-docs.md) | enhancement / needs-info | guidance | Old automatic-PR-CI prescription conflicts with current standing manual-dispatch-only verification rule. Propose optional consumer enforcement and accurate bypass limits; do not add hosted CI or describe it as universally unbypassable. |
| [161](../../.tickets/161-frontier-work-aggregate-provenance.md) | enhancement / wontfix | guidance | Approved obsolete steering/consumer work; #207 retired the steering surface. Await #227 cancellation; do not recreate frontier-work.md. |
| [163](../../.tickets/163-lint-backfill-missing-id.md) | bug / ready-for-agent | compliance | Confirmed: bare lint skips missing-id ticket and leaves bytes unchanged. lint --fix is currently rejected (exit 2); use existing bare lint mutation interface unless an explicitly justified CLI extension is chosen. |
| [164](../../.tickets/164-support-slug-ids.md) | bug / ready-for-agent | compliance | Confirmed: id slug-key in slug-key.md produces id-filename-mismatch, exit 1. Preserve numeric mismatch detection; record supported-ID decision as part of the fix. |
| [165](../../.tickets/165-config-set-project-scope.md) | bug / ready-for-agent | config | Confirmed: config --project --set close.require_resolution=true exits 2 because --project is absent. Preserve default user scope and native TOML values; coordinate typed reads with #190 without treating it as a hard prerequisite. |
| [168](../../.tickets/168-tooling-nudge-backlog-tags.md) | enhancement / needs-info | guidance | Deferred advisory proposal after completed #167. First measure whether nudges are still wanted; keep quiet/JSON behavior and do not promote advisory tag checks into a mandatory gate. |
| [173](../../.tickets/173-validate-frontier-readiness-rule.md) | enhancement / needs-info | quality | Deferred readiness proposal overlaps #175 source-reference rule. Separate frontier readiness from transitive backlog-blocker advisory and decide strict-mode policy before adding duplicate findings. |
| [175](../../.tickets/175-validate-source-reference.md) | enhancement / ready-for-agent | quality | Deferred source-reference advisory after completed #174. Own reusable source-reference predicate; #173 should reuse it. Durable inline requirements suffice; missing historical scratch research is not required. |
| [176](../../.tickets/176-new-body-template.md) | enhancement / ready-for-agent | cli | new --help has no body/template flags and creation still emits TBD. #87's named-template scope must be represented explicitly in ACs; keep #174 stub detection working. |
| [177](../../.tickets/177-guidance-sync-ticket-content.md) | enhancement / ready-for-agent | guidance | Retain dependency #176; remove the claim that absent edit --tags is intentional. #233 owns tag-edit behavior; #230 owns classified evidence guidance. #179 already delivered index mapping. |
| [184](../../.tickets/184-sync-plan-narrative-impl.md) | enhancement / needs-info | planning | ADR 0002 governs optional narrative parsing; table projection remains #187. Decide how to isolate the PlanState/ASCII correctness fix from the deferred parser and resolve benchmark-vs-AST wording before building. |
| [186](../../.tickets/186-reconcile-parity-harness.md) | bug / needs-info | quality | tests/parity/run.sh still expects python -m tkt (or an external command), absent from tracked source. No runnable Python reference was found; choose retirement or provide a pinned reference, never count the dormant harness as a passing check. |
| [187](../../.tickets/187-sync-plan-projection.md) | enhancement / ready-for-agent | planning | Deferred deterministic projection per ADR 0002; current sync-plan only checks/fixes tables. Preserve human narrative and independent optional #184 scope. |
| [190](../../.tickets/190-strict-config-policy-parsing.md) | bug / ready-for-agent | config | Confirmed: push.enabled="treu" followed by config --show exits 0 and reports push.enabled="false" (project). Parse at the config boundary; do not silently weaken policy. |
| [191](../../.tickets/191-centralize-mutation-effects.md) | enhancement / ready-for-agent | architecture | Deferred consolidation after completed #136. MutationContext and GitTransaction already exist; consolidate effects only where needed and preserve their distinct allocation/existing-ticket responsibilities. |
| [193](../../.tickets/193-label-namespace-config.md) | enhancement / ready-for-agent | config | Label namespace parsing is not present in the known config contract. Parse single_select/values at the boundary; #198 owns enforcement and must not be duplicated here. |
| [194](../../.tickets/194-assignee-field-and-claim.md) | enhancement / ready-for-agent | cli | AgentId/eligibility shipped in #224; assignee ownership has not. Reuse invocation identity, preserve eligibility as a separate policy, and add ownership fixtures. |
| [195](../../.tickets/195-parent-validation.md) | bug / ready-for-agent | schema | Confirmed: parent: "999" on the sole valid ticket passes validate with zero findings. Typed parent storage exists from #192; add dangling/self/cycle findings in src/findings.rs. |
| [196](../../.tickets/196-parent-cli-and-query.md) | enhancement / ready-for-agent | cli | Parent storage exists but new/batch/edit/query have no parent interface. Add dependency #195 so new mutations share valid parent rules; retain completed #192 history. |
| [197](../../.tickets/197-parent-rollup-frontier.md) | enhancement / needs-info | schema | Automatic done contradicts the close-only evidence contract. Decide computed progress versus stored status and cancellation semantics before implementation; do not auto-close parent files. |
| [198](../../.tickets/198-label-namespace-validation.md) | enhancement / ready-for-agent | config | Keep dependency #193. Validate declared namespaces only; unrelated free-form tags retain their existing behavior. |
| [199](../../.tickets/199-native-fields-docs-and-changelog.md) | enhancement / ready-for-agent | guidance | Keep #194/#196/#197/#198 dependencies. This is the final combined walkthrough, not permission to defer each feature's own required guidance updates. |
| [200](../../.tickets/200-ready-scoped-to-parent.md) | enhancement / ready-for-agent | cli | Keep #196/#197 dependencies. Parent-scoped frontier is absent; reuse readiness and priority/ID ordering. Decide parent-only versus interim tag alias without inventing computed sort overrides. |
| [202](../../.tickets/202-global-agent-integration.md) | enhancement / needs-info | integrations | Repository deploy already exists; remaining binary-only skill delivery is excluded by Cargo.toml include inventory. Decide embedded-assets/deploy command before implementation; verify harness paths when worked. |
| [208](../../.tickets/208-capabilities-manifest-incomplete.md) | bug / wontfix | tooling | Approved duplicate of #138; await #227 cancellation with --by 138. Do not implement a second manifest fix; the earlier statement that #208 was already closed was inaccurate. |
| [226](../../.tickets/226-propose-agents-restructuring.md) | enhancement / ready-for-agent | guidance | Proposal-only task remains current. Include verification-policy contradictions and progressive disclosure; restructuring requires approval after the concrete coverage map is reviewed. |
| [227](../../.tickets/227-cancelled-status.md) | enhancement / ready-for-agent | cli | Accepted ADR 0004 defines cancellation; no cancelled status or cancel/reopen commands exist. Own honest retirement of #87/#94/#115/#161/#208; never bypass close gates to clear them. |
| [228](../../.tickets/228-migrate-dashboard-plan.md) | enhancement / ready-for-agent | tooling | Dashboard lacks gm-tickets and Plan renderer. Migration is separate work; this reconciliation changes only gm-entries, preserves history, and records the existing checker warning. |
| [229](../../.tickets/229-close-artifact-cleanup.md) | enhancement / ready-for-agent | guidance | Close-artifact guidance remains portable standalone tkt scope. Sync work/reconcile/init wording; do not introduce a foreign-skill dependency or delete artifacts of unknown ownership. |
| [230](../../.tickets/230-qualified-closure-evidence.md) | enhancement / ready-for-agent | guidance | Own classified artifact/functionality/user-outcome evidence examples, including the evidence refinement previously appended to #177. No duplicate implementation in #177. |
| [231](../../.tickets/231-query-all-frontmatter.md) | bug / ready-for-agent | cli | Confirmed: query omits tags/requires/custom even when stored, but agents is already emitted. Preserve agents and add raw-field coverage without losing typed known-field semantics; coordinate parent with #196. |
| [232](../../.tickets/232-frontier-pending-blockers.md) | bug / ready-for-agent | cli | Confirmed JSON has blocked_by:["01"] for ready ticket 02 after 01 is done, and no pending_blockers. Human ready already prints no blockers. Keep historical edges; add pending diagnostics without regressing human output. |
| [233](../../.tickets/233-edit-tags.md) | enhancement / ready-for-agent | cli | edit --help exposes no tag flags; new --tags already accepts a list. Define replace/add/remove/clear deterministically and update #177's obsolete intentional-limitation wording. |
| [234](../../.tickets/234-moved-acceptance-criteria.md) | enhancement / needs-info | cli | Moved-AC intent is current; decide marker grammar, multiple source/destination criteria, evidence mapping, idempotence and atomic multi-ticket recovery before implementation. Do not mark moved work done. |
| [235](../../.tickets/235-mutation-hooks-and-deferred-push.md) | enhancement / ready-for-agent | cli | Confirmed git::commit invokes git commit -qm (hooks run), and new lacks --no-push. push.enabled=false already works; MutationContext already reports local durable commit on push failure. Extend missing behavior rather than rebuild those parts; preserve plan-file changes in mutation commits. |
| [236](../../.tickets/236-config-show-macos-path.md) | bug / ready-for-agent | config | Confirmed with isolated XDG_CONFIG_HOME: config still labels ~/.config/tkt/config.toml instead of the file actually loaded. Cross-platform loader uses dirs::config_dir; macOS runtime remains untested here. |
| [237](../../.tickets/237-debug-config-defaults-to-log.md) | bug / ready-for-agent | config | Confirmed isolated user config debug=true, no TKT_DEBUG/debug.format: query emits two [tkt:debug] stderr lines. Honor documented log default and test explicit format overrides. |
| [238](../../.tickets/238-rebase-stale-body-refs.md) | bug / needs-info | cli | Confirmed dangling #999/ticket 998 body references pass validate, but collision-renumber stale-body behavior was not reproduced. Capture both local/upstream before/after bytes before changing rebase; false-positive/body-ownership policy needs a bounded design. |
| [239](../../.tickets/239-historical-closure-evidence.md) | enhancement / ready-for-agent | quality | Audit --deep reports 20 historical warnings: five missing-evidence closures and fifteen template closures. Review and disposition without invented historical results; unrelated informational high-priority notices are not failures. |

## Completed-ticket audit and remaining uncertainty

`tkt audit --deep --brief` reports **32 findings**: 12 informational high-priority
open notices and **20 warnings**. It exits 0 because warnings are advisory; this is
a **pass with warnings**, not clean historical evidence.

- Five `low-evidence-closure` warnings: #154, #155, #156, #174, #206. Their original
  checks cannot be recreated by adding a new fabricated Verification section.
- Fifteen `template-only-closure` warnings: #48, #80, #81, #82, #84, #91, #92, #95,
  #96, #97, #102, #129, #149, #152, #174. Some are duplicates or discuss templates;
  #239 must discriminate actual stubs from accepted historical context.

ADR 0003 intentionally does not exempt these rules. #239 owns a bounded review;
none of the 173 done statuses was changed and no historic success was re-certified.
The local v0.4.0 tag and #146 resolution support correcting the plan's stale release
warning; remote registry/publication state was not independently re-audited.

## Verification and retention

Observed final documentation checks:

| Check | Outcome |
|---|---|
| `tkt --agent codex validate --strict --brief` | Pass, zero findings |
| `tkt --agent codex sync-plan --check --strict --brief` | Pass, zero findings |
| `tkt lint --check` | Pass, all files canonical |
| Inline Python metadata/coverage/link audit | Pass: 239 records, 66 unique unfinished plan rows, correct triage cardinality, no broken local Markdown links, all 173 completed files identical to baseline |
| `git diff --check` | Pass, no whitespace errors |
| Dashboard `check-dashboard.mjs docs/status.html` | Pass with existing missing gm-tickets warning owned by #228; initial new-entry state value failed validation and was corrected to `now` before this passing run |

 Rust build/test/clippy are
skipped: no Rust or tooling behavior changed. Dashboard changes are limited to its
permitted data block; the missing Plan tab remains #228 and is not repaired here.
No rendered dashboard review is claimed. Final check outcomes are recorded in the
commit's delivery entry and session response.

The compact observations above are promoted evidence. Newly created temporary
fixture repositories and the temporary triage index are disposable after these
notes and final checks are complete; remove them to the OS trash. Pre-existing
build output, deployment bundles, scratch and references were not deleted.

Measured logical bytes before cleanup (not allocated or reclaimable bytes):

| Existing family, preserved | Logical bytes |
|---|---:|
| target/ | 3,537,191,661 |
| dist/ | 318,787 |
| .scratch/ including the temporary index before removal | 515,278 |
| .references/ | 1,672 |

These existing families were not created by this reconciliation and were not
deleted. The new fixture family was 247,862 logical bytes (251 files); its relevant
observations are promoted above. The new temporary index is redundant after the
report and ticket assessments are written. Both new temporary artifacts are sent
to the OS trash, not permanently deleted. Free space stayed at 92.0 GiB.

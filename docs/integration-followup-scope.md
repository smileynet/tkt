# Harness integration: delivered pieces and one bounded follow-up

Ticket #69 records a scope decision, not a new integration implementation. The
owner chose a focused follow-up instead of reopening a broad hooks/MCP exploration.
Current Cargo/plugin metadata remains 0.4.0; the historical ticket paragraph saying
that 0.5.0 already shipped is not used as release evidence.

## Delivered integration inventory

| Piece | Current source | Delivered boundary |
|---|---|---|
| Standalone workflows | `skills/tkt-core`, `tkt-orient`, `tkt-work`, `tkt-reconcile` | Four independently packaged workflow skills; installation is separate from the binary. They describe the installed CLI rather than require a second workflow package. |
| Invocation ownership and scheduling eligibility | `src/core/ticket.rs`, `src/commands/claim.rs`, `src/commands/ready.rs`, `src/mutation.rs` | Identity, eligibility and optional assignee are distinct from tag scope and machine capabilities. Existing ticket transactions own fetch/commit/push. This is CLI scheduling policy, not a native harness scheduler. |
| Claude session context | `.claude-plugin/claude-hooks.json`, `.claude-plugin/hooks/session-start.mjs` | A private SessionStart command collects ready JSON and in-progress query output from the launch project. Failure produces no partial context. The manifest uses one shell-string command (no `args` — a Claude direct-spawn field ZCode's command type ignores, spawning bare node; ticket 286), and the script speaks each host's output dialect: plain stdout for Claude Code, a strict `hookEventName`/`additionalContext` JSON envelope when a `ZCODE_*` host marker is set. Ready respects invocation, tag and environment/capability filters. The in-progress query uses status and tag context, but can include other owners/agents/environments; it is not narrowed like ready. Neither output is an unfiltered board export. |
| Codex workflow plugin | `.codex-plugin/plugin.json` | Four skills, with an explicit empty hooks list. Claude's private hook is not presented as a Codex feature. |
| Other harness packaging and root ownership | `plugin.json`, `.claude-plugin/plugin.json` deployment metadata, `tools/deploy.mjs` | Explicit package/runtime files and guarded owned scan-root copies; one path per harness. Deployment receipts describe observed installation separately from available source. Kiro Crew can remain staged when its CLI is absent. |
| Optional project Git hook | `assets/pre-commit.sh`, `src/commands/hooks.rs`, `docs/optional-hooks.md` | An explicitly installed advisory pre-commit shim; init does not activate it. This is separate from intercepting a native file-writing tool. |
| Prepared local files | `src/effects.rs`, config/context/init callers | Preview/apply and individually atomic publication for local files. This does not complete parent #191's every-mutator consolidation or create native-hook configuration. |

The final #253 gate passed 435 Rust tests and 42 Node fixtures, including existing
SessionStart, ownership and deployment fixture coverage; one existing Windows
relative-link fixture was skipped. That source verification does not establish a
new native model session or activation in every harness. Native deployment is
refreshed separately at the end of this execution batch.

## Selected follow-up: advisory native write guidance

Continue existing #158; create no duplicate umbrella ticket or new dependency edge.
The concrete gap is that a generic file-writing tool can bypass CLI lifecycle gates
while legitimate ticket-body editing must remain possible. The bounded hypothesis
is that model-visible advisory context at a recognized ticket write can steer the
agent toward the lifecycle command without denying the edit. Its effectiveness
remains unmeasured; this is not a claim of recurring production misuse.

The owner-approved scope is explicit owned project configuration for the Claude
and Codex paths described in [the native-hook evidence](evidence/native-write-hooks-158.md).
That record names observed profiles, tool-input contracts and trust limitations;
exact minimum versions remain unknown. Do not invent version gates, new flags,
permission/deny policies or implicit installation while implementing it. Do not
alter user or project trust records. A reviewable generated project configuration
does not prove native execution or model-visible output.

### Validation plan for that follow-up

1. Fixture tests bind supported input envelopes to intended ticket paths, with
   unrelated files, malformed input, mixed patches and unsupported tool routes as
   negative controls. Do not claim arbitrary shell or MCP write coverage.
2. Generation/merge tests preserve foreign settings and changed owned definitions,
   reject conflicts, avoid duplicate entries on rerun, and prove dry-run leaves the
   complete file tree unchanged. No trust or permission state is changed.
3. Exercise the generated advisory command in isolation with positive and negative
   input. Record malformed-input behavior, output and exit status independently of
   native dispatch. Advisory failures must not become a deny decision.
4. Bind any native acceptance to the actual harness/profile, reviewed definition
   and observed tool call. Record trust/dispatch/model-visibility limits explicitly
   when unavailable. Fixture success alone is not native qualification.
5. Obtain independent critic acceptance against final source/evidence before
   closing #158, then refresh the plan and dashboard.

## Scope excluded from this decision

No MCP server, background agent, cross-harness scheduler, default native write hook
or automatic trust change is justified by this inventory. #148 already owns role
examples; #202 completed an embedded-assets design, not an offline runtime skill
installer. These are separate existing contracts and do not become prerequisites
for #158. The Git hook in #157 does not provide a shared native-hook configuration
layer. Broader integration possibilities remain possibilities, not missing
acceptance for #69 or promised release content.

# Proposal: restructure the contributor `AGENTS.md`

## Scope and baseline

This is a proposal only. The inventory was read from `AGENTS.md` at repository
`HEAD` **81151b18c0932ad51a4348a1ad08fffd35fd65e6** on 2026-11-02. The file has
concurrent edits in the worktree; the listed contents and headings match that
checked-out version. Reviewers can reproduce the inventory with
`git show 81151b18c0932ad51a4348a1ad08fffd35fd65e6:AGENTS.md` and compare each
heading below. No guidance surface is changed by this proposal.

The design follows `writing-for-agents`: keep steps and high-frequency hard
constraints in the always-loaded file; disclose reference material behind
branch-specific pointers. Every pointer should say what it contains and when to
read it. Do not extract material merely to make the file shorter.

## Section-by-section disposition

| Current section | Proposal | Destination and pointer trigger |
|---|---|---|
| `Project` | Keep a short identity and repository-purpose paragraph. | Always loaded; it establishes the project being edited. |
| `Workspace Layout` | Keep the `GLOSSARY.md` / ADR / workspace-layout pointers and dashboard rule. Keep the session-operations pointer. | `docs/agents/specs/workspace-layout.md` when locating or changing source; `docs/agents/session-operations.md` when dispatching collaboration workers or inventorying references. Dashboard rule stays inline because it applies after any surfaced work. |
| `Commands` examples | Keep a compact entry-point list for build, test, lint, format-check, install, and `mise run check`; remove duplicated CLI inventory from this section. | `skills/tkt-core/references/commands.md` for command usage; `tkt capabilities -o json` for the complete current command/flag schema. Trigger when using or documenting the CLI. |
| `Verification gate (run before every commit)` | Keep the required gate command and its completion rule inline. | Always loaded before code commits. Keep the exact tracked command aligned with `mise.toml`; retain the full-gate requirement unless an explicit project verification-phase policy is adopted. |
| `Session workflow (commit + push + deploy cadence)` | Keep commit, fetch/divergence-check, push, and deploy defaults inline as ordered session steps. | Always loaded when completing repository work. These are operational constraints, not reference material. |
| `Deploy (run after pulling or making changes)` | Keep the short decision rule inline: Rust changes install the CLI; skill/guidance/plugin changes deploy the pack. Move detailed deploy mechanics, root selection, audit modes, and recovery notes. | Proposed `docs/agents/plugin-maintenance.md` (new) when deploying, auditing, or diagnosing a skill/plugin install. Keep one clear pointer beside the inline decision rule. |
| `Releases` | Keep the hard authorization boundary inline: release guidance is mandatory; publication is separate and not authorized by staging. | `docs/releasing.md` when preparing, verifying, staging, resuming, or publishing. Preserve the current statement that authorization stops at staged candidates. |
| `Claude Code plugin (keep it current)` | Keep only the cross-harness invariant that one skill must load once per harness and the requirement to refresh after skill/CLI surface changes. Move detailed Claude hook, marketplace, manifest, cache/version, and validation instructions. | Proposed `docs/agents/plugin-maintenance.md` (new), triggered by Claude/plugin/deployment changes. Keep release version changes linked to `docs/releasing.md`; never duplicate release authority here. |
| `tkt CLI (the product)` command block and prose | Replace the full inventory with the supported-use pointer: consult `tkt capabilities -o json`; use the tkt-core command reference for workflows. Keep a short statement that schemas describe supported workflows and completion goes through `close`. | Existing `skills/tkt-core/references/commands.md` and `capabilities.md`, triggered by CLI use or CLI-facing documentation. The machine manifest is the inventory source; human reference explains semantics. |
| `Global flags` | Move the CLI flag table out of always-loaded contributor guidance. | `commands.md` / `capabilities.md`, triggered by CLI invocation or documentation. Global contributor verification requirements remain inline in their own section. |
| `Priority levels` | Move detailed value/order reference. | `commands.md`, triggered when creating, ordering, or explaining tickets. |
| `Status values` | Keep only the short close-only invariant and pointer inline; move lifecycle/default details. | `commands.md` and `capabilities.md`, triggered by lifecycle or status behavior. The close-only rule remains prominent because it is a safety boundary. |
| `Agent eligibility` | Move the full syntax and matching semantics. | `commands.md`, triggered when using `--agent`, eligibility, `ready`, or `claim`. Keep the distinction between tags, requirements, eligibility, and ownership in the reference. |
| `Configuration` | Move the list of sections and field semantics. | `commands.md` (or a focused config reference if that section becomes too large), triggered when editing `.tickets/config.toml` or config behavior. |
| `Environment variables` | Move the variable table. Keep only test isolation (`DO_NOT_TRACK=1`, `TKT_NO_USER_CONFIG=1`) next to the integration-test rule if tests remain a common contributor path. | `commands.md` or a focused config reference, triggered by environment/configuration work; test isolation stays next to test guidance. |
| `Architecture Decisions` | Keep a pointer to the ADR index and move each unrecorded decision below to a proposed architecture reference. Existing ADRs remain authoritative only for decisions they actually contain. | Proposed `docs/agents/architecture-reference.md` (new), triggered by architecture/design changes; `docs/agents/specs/workspace-layout.md` remains authoritative for file locations. Do not label an implementation constraint an ADR unless recorded as one. |
| `Contract` | Keep plan synchronization and the “commit or stash existing plan edits first” step inline. Move each remaining contract detail below to a proposed contract reference; `commands.md` and `tkt-conventions.md` do not currently contain all of them. | Proposed `docs/agents/tkt-contract.md` (new), triggered by ticket storage/format changes, CLI compatibility, close/claim behavior, or worktree support. Existing `commands.md` remains the command workflow reference, not a replacement for the exact contract. |
| `Constraints` | Split every bullet by trigger; see the exhaustive constraint map below. | `docs/agents/agent-guidance-surfaces.md` for CLI/agent-facing behavior changes; use existing release/session references or the proposed maintainer reference only on their stated triggers. |

### Exhaustive map of current `Architecture Decisions` bullets

The five existing ADRs are `0001-enforcement-warn-default`,
`0002-plan-narrative-support`, `0003-audit-debt-grandfather-cutoff`,
`0004-cancelled-status`, and `0005-slug-id-validation`. They do not record the
implementation decisions below, so do not cite them as destinations for these
rules. A future implementation should place these exact rules in a proposed
`docs/agents/architecture-reference.md` (new), or retain the individual rule
inline if the reference would not be reliably reached:

| Current decision | Proposed home | Trigger |
|---|---|---|
| Shell out to git rather than libgit2, for SSH/HTTPS compatibility and the chosen simple v1 model. | Proposed `architecture-reference.md`. | Git integration or dependency/design changes. |
| `TicketFile` preserves raw frontmatter for surgical edits; typed `Ticket` supplies validated fields; mutations use `.file`, reads use typed fields. | Proposed `architecture-reference.md`; link to `docs/agents/specs/workspace-layout.md` for source locations. | Ticket model, parsing, or mutation changes. |
| Custom line-based frontmatter parser preserves raw content, supports escaped YAML double-quoted scalars, and deliberately does not implement full YAML. | Proposed `architecture-reference.md`. | Frontmatter parser or serialization changes. |
| Operations are sequential; there is no async runtime. | Proposed `architecture-reference.md`. | Runtime/concurrency or dependency changes. |
| Telemetry is local-only, opt-in JSONL, segmented by project, session-aware, rotated, and nonblocking. | Proposed `architecture-reference.md`; link to `TELEMETRY.md` for user-facing disclosure. | Telemetry, storage, or privacy changes. |
| Fixed regexes use `LazyLock` statics. | Proposed `architecture-reference.md`. | Regex/performance or initialization changes. |
| No color crate; ANSI and terminal detection are handled directly. | Proposed `architecture-reference.md`. | Terminal output or dependency changes. |
| Errors use `DomainError`/`ErrorKind`; `domain_bail!` defaults to Validation and JSON errors go to stderr under `-o json`. | Proposed `architecture-reference.md`; user-visible CLI behavior also belongs in `commands.md`/capabilities. | Error model or output changes. |
| Re-export only types callers name directly from the core facade; avoid unused-import warnings from unused re-exports. | Proposed `architecture-reference.md`. | Public module/facade changes. |

### Exhaustive map of current `Contract` details

Only the existing-plan synchronization rule has an immediate mutation procedure
that should stay in `AGENTS.md`. The remaining details are not fully described by
the current command reference or triage conventions. Put them in a proposed
`docs/agents/tkt-contract.md` (new), preserving the following mapping:

| Current contract detail | Proposed home | Trigger |
|---|---|---|
| Claim, close, and status-changing edit synchronize existing `docs/plan.md` rows in the same commit; commit or stash plan edits first; missing rows warn until `sync-plan --fix`; no plan means no new file; alternate plans require explicit selection. | Keep the same-commit and commit/stash steps inline; put the full behavior in `tkt-contract.md`, linked to `commands.md` for sync-plan usage. | Claim/close/status mutation or plan synchronization changes. |
| Ticket files `.tickets/{NN}-{slug}.md` are the database. | `tkt-contract.md`; retain the concise “ticket files are source of truth” statement inline. | Ticket storage, allocation, or worktree behavior changes. |
| Ticket body/specification prose is user-owned; the tool does not manage it; close appends a `Resolution` section. | `tkt-contract.md`; retain the user-owned-body boundary inline beside mutation safety. | Ticket editing, close, or body-preservation changes. |
| Push-to-claim means the pushed commit is the claimed ID; push rejection triggers race detection. | `tkt-contract.md`; cross-link mutation push-gating in the inline constraints. | Claim/allocation transaction changes. |
| Exit codes: 0 success, 1 domain failure, 2 operational crash. | `tkt-contract.md`; mirror in `capabilities.md` only if the manifest owns exit semantics. | CLI compatibility or error/reporting changes. |
| Default output is JSON for validate/sync-plan and human for ready/new/claim/close/edit; ready supports `--json`. | `tkt-contract.md` and `commands.md` where command workflows need the detail. | Command output or format changes. |
| Closing from `spike/*` appends the branch name to resolution. | `tkt-contract.md`, with command workflow pointer from `commands.md`. | Close/resolution behavior or spike workflow changes. |
| The CLI works from Git worktrees; `.tickets/` belongs to the checked-out tree. | `tkt-contract.md`; source layout remains in `workspace-layout.md`. | Git/worktree support or path resolution changes. |

### Exhaustive map of current `Constraints` bullets

| Current rule | Proposed home | Trigger / always-loaded reason |
|---|---|---|
| Frontmatter changes require updating `tkt-work`. | Keep inline. | Always loaded: a compatibility guard for ticket contract changes. |
| Distributed skills must be self-sufficient for tkt-only adopters and link only tkt/sibling skills. | Keep inline. | Always loaded: applies to every edit under `skills/`. |
| CLI flags or agent-facing behavior changes require updating all guidance surfaces. | Keep the requirement inline; point to `docs/agents/agent-guidance-surfaces.md`. | Trigger on any CLI or agent-guidance behavior change; the checklist is the exhaustive inventory. |
| Do not add computed frontier sort overrides; priority then ID ordering is intentional. | Keep the short invariant inline and record details in proposed `docs/agents/architecture-reference.md`; the command reference retains user-facing ordering. | Trigger when changing frontier behavior or sort logic; consult the architecture reference before design changes. |
| Do not add dependencies without justification. | Keep inline. | Always loaded for dependency changes. |
| Do not use libgit2/gix; shell out to git. | Proposed `docs/agents/architecture-reference.md`, with a concise pointer from the architecture invariant section. | Trigger when changing Git integration or proposing a Git library. |
| Do not suggest releasing with open tickets absent explicit user deferral. | Keep the release guard inline and point to `docs/releasing.md`. | Trigger at release planning; keep inline because it is a hard release gate. |
| Maintain CLI compatibility. | Keep the principle inline; detailed behavior belongs in `commands.md` and `capabilities.md`. | Trigger on CLI changes; consult both references and the guidance-surface checklist. |
| Clippy has zero warnings and formatting check has no diff. | Keep inline with the verification command. | Always loaded for code changes and commits. |
| Integration tests set `DO_NOT_TRACK=1`. | Keep beside test commands in the contributor runbook. | Trigger when adding or changing integration tests. |
| Unit tests must not assert ambient consent state. | Keep beside test commands in the contributor runbook. | Trigger when writing tests involving telemetry/consent; prevents environment-dependent tests. |
| Windows rename replaces an existing file on supported filesystems; never delete destination first; source and destination stay on one filesystem. | Keep the atomic replacement invariant inline; link Windows-specific evidence/details to proposed `docs/agents/windows-maintenance.md` if retained. | The no-delete-before-rename safety rule triggers for all atomic-write changes; filesystem evidence is needed for Windows file-operation changes. |
| Ticket writes use `core::atomic_write`; classify I/O failures and do not bypass close gates by hand-editing `done`. | Keep inline in compact form. | Always loaded for ticket mutation and error diagnosis because it protects data integrity and lifecycle policy. |
| Codex review dispatch uses `codex exec --dangerously-bypass-approvals-and-sandbox` due to local bwrap constraints; `codex review --base` cannot use a custom prompt. | Existing `docs/agents/session-operations.md`. | Trigger when dispatching a Codex review; pointer already describes dispatch/reference-inventory traps and should explicitly name this review invocation branch. |
| cargo-dist executable is `dist`; generated release workflow is generated, not hand-edited. | `docs/release-staging.md`. | Trigger when inspecting or changing release workflow/tooling. |
| New mutation commands use push-gated transactions; direct push calls check `push.enabled`. | Keep inline. | Always loaded for new mutation paths because it gates external side effects. |
| Windows Cargo/Rust homes, PATH order, `os error 193` proxy repair, and possible Defender exclusion. | Proposed `docs/agents/windows-maintenance.md` (new). | Trigger on Windows toolchain setup/build/formatter failures; elevated Defender changes remain a last-resort instruction, not a default action. |
| Product changes are validated with `cargo test` integration tests using the current target binary; installed `tkt` is for deploy/real-repository use; use freshness check for install-path tests. | Keep concise distinction by test/deploy commands; details in `docs/agents/maintainer-tooling.md` (new) only if the test workflow needs more space. | Trigger on product validation or install/deploy-path tests. |
| Use `git commit -F <file>` for multiline/quoted messages on Windows. | Proposed `docs/agents/windows-maintenance.md` or existing `session-operations.md`. | Trigger when composing a multiline/quote-containing commit message in Windows PowerShell. |
| Check crate-name availability with `cargo search`, not raw API `curl`. | Proposed `docs/agents/maintainer-tooling.md` (new), or keep the one-line trap inline if a separate maintainer reference is not justified. | Trigger when checking crates.io name availability. |
| Generate `.github/workflows/release.yml` through `node tools/generate-release-workflow.mjs --write`; do not hand-edit; `mise run check` checks generation. | `docs/release-staging.md`. | Trigger when changing the generated release workflow or release tooling. |

## Proposed always-loaded shape

Keep `AGENTS.md` as a short contributor runbook, in this order:

1. Project identity and source-of-truth pointers.
2. Common build/check commands and the exact required verification gate.
3. Commit, fetch, push, and deploy completion steps.
4. Release and mutation safety boundaries.
5. Compact project invariants that apply across tasks.
6. A small set of branch-specific pointers to CLI, architecture, plugin, workspace,
   and platform references.

The body should not repeat a reference's full inventory. A pointer must identify
the branch (“when using the CLI”, “when changing architecture”, etc.) and the
material reached. Keep the tkt-only skill boundary explicit because it applies
even when no one opens the reference. Keep close-only lifecycle, atomic write,
push gating, release authorization, and plan synchronization rules visible where
they constrain a mutation.

## Guidance-surface matrix and checklist changes

Update `docs/agents/agent-guidance-surfaces.md` as part of a future approved
implementation, not in this proposal:

- Change the `AGENTS.md (tkt CLI section)` surface description from the complete
  command/config inventory to **contributor summary plus conditional pointers**.
- Add `skills/tkt-core/references/capabilities.md` as the authoritative machine
  schema and `skills/tkt-core/references/commands.md` as the workflow/semantics
  reference. Keep init snippets, skills, README, and plugin surfaces in the
  matrix; this proposal does not replace those audiences.
- Reassign each matrix topic currently marked in `AGENTS.md` (flags, enforcement
  config, env filtering, eligibility, health, tags, migration, validation,
  batch, and advisory hints) to the CLI reference/manifest where appropriate.
  Keep the AGENTS column marked only for the compact lifecycle, safety, and
  contributor-operating rules actually retained there.
- Add matrix rows for contributor operations, release authorization, deployment
  and platform setup only if the matrix is intended to track those non-CLI
  contracts. Otherwise state that its scope is CLI behavior and keep the new
  contributor references out of that table.
- Add checklist items requiring the `AGENTS.md` pointers and matrix assignments
  to be updated whenever CLI coverage moves, and requiring the new references to
  be checked for stale duplicated rules. Preserve existing init-snippet and
  release-version checklist items.

The machine manifest currently derives inventory from Clap, while prose
references explain supported semantics. Keep that distinction explicit; do not
make `AGENTS.md` a second manually maintained flag list.

## Policy contradictions and resolution before implementation

1. **Verification cadence:** the user-level standing rules select a speed or
   stability phase using a `Verification phase:` line and describe one check in
   speed mode. Project `AGENTS.md` says `mise run check` before every commit and
   calls for the full Rust gate. No `Verification phase:` line appears in the
   current file. Decide and state whether the repository opts into the full gate
   for every code commit or follows the phase policy; keep one unambiguous rule.
   Do not silently weaken the existing project gate during restructuring.
2. **Format command:** the command examples show bare `cargo fmt`, while the
   explanatory rule says to use `cargo fmt --check` because bare formatting
   mutates and cannot gate a diff. Future examples should show `cargo fmt
   --check` for verification; if the mutating formatter is retained, label it as
   a repair command.
3. **Deploy descriptions:** the session rule says `node tools/deploy.mjs` refreshes
   harness roots and Antigravity; the detailed section also describes Claude
   marketplace verification and the wrapper. Consolidate these mechanics in one
   reference while preserving the inline deploy decision and failure boundary.
4. **Release status:** `AGENTS.md` says the current authorization stops at staged
   candidates; `docs/releasing.md` also describes production publication
   machinery and says it has not run. Keep the authorization sentence inline
   and treat the release guide as detailed procedure, not implied permission.
5. **Working tree freshness:** current `AGENTS.md` contains live edits from the
   concurrent ticket batch. Before implementing this proposal, re-inventory the
   then-current file and update line/section mappings; this document's cited
   baseline makes the present map reproducible but does not authorize applying
   it to changed text without reconciliation.

## Approval and implementation checklist

This document requests review of the proposed information hierarchy and coverage
map. After approval, an implementation should:

- Re-read the then-current `AGENTS.md` and reconcile every inventory row.
- Create only the new references approved above; keep existing canonical docs as
  the source of truth where available.
- Preserve each hard safety/operational rule and write one branch-specific
  pointer at the retained location.
- Update the guidance-surface matrix and checklist in the same change as the
  moved CLI guidance.
- Check each pointer for a clear trigger and each moved rule for an authoritative
  home; remove duplicate prose rather than leaving stale copies.
- Review the final `AGENTS.md`, every proposed destination, and the matrix against
  this map. The acceptance condition is that every current rule is retained,
  relocated with a reliable trigger, or deliberately retired with an explicit
  reason.

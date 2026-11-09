# AGENTS.md

## Project

tkt — Git-native ticket CLI. Manages `.tickets/` frontmatter-driven work tracking with atomic git operations, race detection, and dependency-graph frontier computation.

Single Rust binary. Originally ported from a Python implementation (now removed). Same interface, same contract.

## Workspace Layout

Read `GLOSSARY.md` for domain vocabulary and `docs/adr/` for recorded decisions.

See `docs/agents/specs/workspace-layout.md` for the full source tree.

**Dashboard:** the delivery-dashboard skill owns routine generation, updates and
verification scope for `docs/status.html`; keep unavailable visual checks as
evidence limits within that workflow, not as a standing validation ticket. Update
the dashboard after completed work worth surfacing; edit only its `gm-config` and
`gm-entries` JSON blocks by hand. The delivery-dashboard `snapshot-tickets.mjs`
helper owns `gm-tickets`; refresh it without an active tag context after ticket
changes, then run the delivery-dashboard checker. Keep the renderer aligned with
the canonical template; renderer migrations require their own ticket and
laptop/phone inspection.

For dispatch and reference-inventory traps, read `docs/agents/session-operations.md`.

## Commands

```bash
cargo build                    # debug build
cargo build --release          # release build (stripped, LTO)
bash tools/install-local.sh  # deploy CLI to ~/.local/bin (shows git hash in --version)
cargo test                     # all tests
cargo test -- --nocapture      # with output
cargo clippy                   # lint (must be 0 warnings)
cargo fmt --check              # format (must produce no diff)
```

### Verification gate (run before every commit)

```bash
mise run check   # source-selected tooling fixtures + Rust gate, PATH-order-independent
# equivalent raw form:
node tools/check-tools.mjs && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
```

Maintainer verification needs Node 22+ and `npm ci --ignore-scripts`; these pinned
Node packages support release tooling, not the installed Rust CLI.

`node tools/check-tools.mjs [--base REF]` always runs cheap tooling safety fixtures,
then selects real build-identity and cargo-dist regressions from changed source,
including tracked edits and untracked inputs. Default scope is the origin/main
merge-base; unavailable scope conservatively selects both. Invalid explicit bases
fail. `--plan` shows selection without executing tests. For release verification,
`mise run check:release` forces both real regressions and the full Rust gate.

Use `--check` (not bare `cargo fmt`) — bare `cargo fmt` mutates in place and exits 0 regardless, so it cannot gate a diff or reveal a broken formatter.

The local `mise run check` gate remains required. The former automatic push/PR
CI workflow was removed by the owner; remaining hosted workflows are manual-only
and require an explicit request. No pre-push hook is installed by default; the
optional hook proposal is tracked in ticket 249; installation remains opt-in.

When documenting consumer CI, hooks or enforcement limits, read
[the enforcement model](docs/agents/enforcement-model.md). Consumer checks are
opt-in; they do not authorize hosted runs or publication in this repository.

### Session workflow (commit + push + deploy cadence)

- **Always commit after completing a task** — each logical unit of work ends in a commit
  with a `type(scope): what changed` message. Don't leave finished work uncommitted.
- **Always push after committing** (a remote exists). Before pushing, `git fetch` and
  verify no upstream divergence; fast-forward or rebase cleanly, otherwise report the
  upstream state before proceeding.
- **Deploy by default.** Prefer to deploy changes (`bash tools/install-local.sh` when Rust code
  changed; `node tools/deploy.mjs` when skills, guidance, plugin manifests or deployment tooling
  changed, which refreshes both the harness scan roots and the Antigravity `agy` plugin) as
  part of finishing the work, unless the user says otherwise. `mise run deploy` does both.
- Run the full gate for Rust changes; run applicable Node/workflow fixtures for
  tooling changes. Docs/ticket-only edits need accuracy, links and scope checks;
  they do not require rebuilding Rust. Commit + push each completed unit and deploy
  once when skills or guidance changed.

### Releases

Read [release guidance](docs/releasing.md) before preparing, staging, publishing or
resuming a release. It links the coordinated contract and current tooling gaps.
The manual `.github/workflows/publish.yml` is the sole GitHub publisher; the generated
dist workflow only builds staging artifacts. Local tooling dispatches/observes CI and
must not create a second release. Current authorization stops at staged candidates.

## Constraints

- Load each skill once per harness. Keep deployment root/ownership guards intact;
  refresh the workflow pack after relevant skill or CLI-surface changes.

- Do NOT change the frontmatter contract without updating the tkt-work skill

- Distributed skills (`skills/<name>/`) MUST be self-sufficient for a tkt-only adopter (someone who has tkt but NOT grey-matter or any other skill set): reference only `tkt` commands and sibling `tkt-*` skills, never a non-tkt skill by name (`frontier-work`, `read-handoff`, `to-tickets`, …). tkt is the standalone tool; grey-matter is an opinionated layer that *wraps* tkt's skills. `tools/deploy-skills.sh` lint enforces this (fails the deploy on a foreign-skill reference).

- Do NOT change CLI flags or agent-facing behavior without updating all guidance surfaces (see `docs/agents/agent-guidance-surfaces.md` for checklist — init snippets, skill, steering, AGENTS.md, README)

- Do NOT implement computed sort overrides for the frontier — priority+ID ordering is intentional and gives explicit control over agent work sequence

- Do NOT add dependencies beyond what's in Cargo.toml without justification

- Do NOT use libgit2/gix for v1 — shell out to git binary

- Do NOT suggest releasing with open tickets — all tickets ship first, or the user explicitly defers them

- Maintain CLI compatibility (same commands, flags, output)

- `cargo clippy` must produce 0 warnings; `cargo fmt --check` must produce no diff

- Integration tests MUST set `DO_NOT_TRACK=1` on child processes (prevents ambient env pollution)

- Unit tests must NOT assert specific consent state (result depends on ambient env vars)

- Ticket writes go through `core::atomic_write` (temp-file + rename) so a failed/interrupted write never leaves a torn ticket file; a failed write surfaces as `DomainError { Io }` with a hint (`cannot write <path>: <errno>`), exit 2. `crash: writing …` (now shows the errno) is an fs/permission error (read-only file, locked, read-only worktree) — NOT a validation bug; never hand-edit `status: done` to work around it (that bypasses the close gates — the exact anti-pattern #178/#179 fixed)

- New mutation commands MUST route push through a push-gated path (GitTransaction respects `push.enabled`; direct `git::push_with_retry` calls must check `pcfg.push_enabled` first)

- Validate product changes with `cargo test` integration tests, which rebuild and exercise the current `target/` binary; do not use the PATH-installed `tkt` to validate code changes. Use the installed binary for deployment and real-repository use. When specifically testing the install/deploy path, run `tools/check-binary-fresh.sh` to compare its embedded git hash with HEAD.

## Ticket mutation workflow

Ticket files are the database and body/specification prose remains user-owned.
Use `tkt close` to reach done, with actual acceptance and evidence; never hand-edit
done or use a generic status edit. Explicit criterion transfers record moved work,
not implemented work. Claim, close and status-changing edits synchronize existing
`docs/plan.md` rows in the same commit: commit or stash plan edits first. Use
`sync-plan --fix` for missing rows; no plan is created by lifecycle mutations.
New mutation commands must use the existing push-gated transaction paths.
Push-to-claim: the pushed commit establishes the claim; push rejection participates
in race detection. Report a durable local commit separately from publication.

## Read by task

User-level standing rules remain in force. These pointers disclose reference
material; they do not relax the inline verification, safety or release rules.

| Before this work | Read |
|---|---|
| Using or documenting CLI commands, flags, configuration, lifecycle, ownership, eligibility, query output, parent progress, criterion transfers or plan projection | `tkt capabilities -o json`, [command workflows](skills/tkt-core/references/commands.md), [manifest semantics](skills/tkt-core/references/capabilities.md), and [the ticket/CLI contract](docs/agents/tkt-contract.md) |
| Changing modules, parser/storage, Git transactions, telemetry, errors, terminal output, dependencies or facade exports | [Architecture reference](docs/agents/architecture-reference.md), plus applicable recorded ADRs |
| Installing the CLI, refreshing/auditing skills/plugins, changing manifests/hooks, or diagnosing marketplace/cache/root ownership | [Plugin maintenance](docs/agents/plugin-maintenance.md); install Rust changes with `cargo install --path .`, refresh pack changes once with `node tools/deploy.mjs` |
| Building, formatting, installing or composing commits on Windows; changing atomic replacement/filesystem behavior on any platform | [Windows maintenance](docs/agents/windows-maintenance.md) |
| Inspecting cargo-dist, checking crate names, or regenerating release workflows | [Maintainer tooling](docs/agents/maintainer-tooling.md) and [release staging](docs/release-staging.md); the release authorization boundary above still applies |
| Dispatching a Codex review or inventorying reference checkouts | [Session operations](docs/agents/session-operations.md) |
| Changing CLI/agent-facing behavior or guidance layout | [Guidance surface checklist](docs/agents/agent-guidance-surfaces.md); check current accuracy and links. The [frozen migration audit](docs/agents/guidance-coverage.md) is on-demand, not a gate on later instruction changes. |

## Mutation effect boundary

Route new persistence through prepared `effects::Phase` plans. Keep scoped Git
commit/publication and recovery in their existing owners; do not add free-form
Git mutation or callback-shaped phases. Preview must return before filesystem,
Git, lock, process or diagnostic application. See [the effect inventory](docs/mutation-effects-audit.md).

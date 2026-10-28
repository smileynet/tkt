# AGENTS.md

## Project

tkt — Git-native ticket CLI. Manages `.tickets/` frontmatter-driven work tracking with atomic git operations, race detection, and dependency-graph frontier computation.

Single Rust binary. Originally ported from a Python implementation (now removed). Same interface, same contract.

## Workspace Layout

See `.memory/specs/workspace-layout.md` for the full source tree.

**Dashboard:** update `docs/status.html` after completed work worth surfacing; edit only
its `gm-config` and `gm-entries` JSON blocks, then run the delivery-dashboard checker.

## Commands

```bash
cargo build                    # debug build
cargo build --release          # release build (stripped, LTO)
cargo install --path .         # deploy to PATH (shows git hash in --version)
cargo test                     # all tests
cargo test -- --nocapture      # with output
cargo clippy                   # lint (must be 0 warnings)
cargo fmt                      # format (must produce no diff)
```

### Verification gate (run before every commit)

```bash
mise run check   # release-tool fixtures + Rust gate, PATH-order-independent
# equivalent raw form:
node --test tools/release-tools.test.mjs tools/release-ci.test.mjs && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
```

Maintainer verification needs Node 22+ and `npm ci --ignore-scripts`; these pinned
Node packages support release tooling, not the installed Rust CLI.

Use `--check` (not bare `cargo fmt`) — bare `cargo fmt` mutates in place and exits 0 regardless, so it cannot gate a diff or reveal a broken formatter.

GitHub Actions runs this same gate automatically for every push and pull request.
That automated check complements, but does not replace, the required local check before
each commit.

### Session workflow (commit + push + deploy cadence)

- **Always commit after completing a task** — each logical unit of work ends in a commit
  with a `type(scope): what changed` message. Don't leave finished work uncommitted.
- **Always push after committing** (a remote exists). Before pushing, `git fetch` and
  verify no upstream divergence; fast-forward or rebase cleanly, otherwise report the
  upstream state before proceeding.
- **Deploy by default.** Prefer to deploy changes (`cargo install --path .` when Rust code
  changed; `node tools/deploy.mjs` when skills, guidance, plugin manifests or deployment tooling
  changed, which refreshes both the harness scan roots and the Antigravity `agy` plugin) as
  part of finishing the work, unless the user says otherwise.
- Run the full gate for Rust changes; run applicable Node/workflow fixtures for
  tooling changes. Docs/ticket-only edits need accuracy, links and scope checks;
  they do not require rebuilding Rust. Commit + push each completed unit and deploy
  once when skills or guidance changed.

### Deploy (run after pulling or making changes)

```bash
cargo install --path .     # when Rust code changed
node tools/deploy.mjs     # once after skill/guidance/plugin changes
```

`node tools/deploy.mjs` verifies Claude reads this checkout directly and refreshes the
enabled Codex plugin, then packages Antigravity when `~/.gemini` exists. `bash tools/deploy-skills.sh` is a compatibility
entry point to the same command; run either once after skill or guidance changes.
The engine is vendored from grey-matter's `scripts/deploy.mjs`; change harness behavior
there and copy it here. Pack-specific forbidden skill references live in
`.claude-plugin/plugin.json` under `metadata.deploy.forbidRefs`.

The harness table chooses roots so each harness sees a skill once, including installed
plugins. Only manifest-owned copies are pruned. Use `--dry-run`, `list`, `audit`, and
`check` to inspect deployment; `--allow-dual` is an explicit duplicate override.
Codex must use this checkout as its local marketplace. A plugin refresh failure stops
deployment before existing skill copies are removed. Start a new chat after refresh.

### Releases

Read [release guidance](docs/releasing.md) before preparing, staging, publishing or
resuming a release. It links the coordinated contract and current tooling gaps.
The manual `.github/workflows/publish.yml` is the sole GitHub publisher; the generated
dist workflow only builds staging artifacts. Local tooling dispatches/observes CI and
must not create a second release. Current authorization stops at staged candidates.

### Claude Code plugin (keep it current)

The Claude-only SessionStart hook adds the launch project's frontier and in-progress
tickets to context when `.tickets/` and the `tkt` executable exist. It runs on each
SessionStart event, including resume and compact, and emits no partial context on
failure. It targets Claude 2.1.310+ and Node 22+. Private hook config/scripts live in
`.claude-plugin/` and `metadata.deploy.claudeFiles`; never put them in shared
`runtimeFiles` or Markdown resource links. Codex explicitly selects no hooks, and
release packaging ships these private files only in the Claude archive.

tkt is also a Claude Code plugin: `.claude-plugin/plugin.json` and `.claude-plugin/marketplace.json`,
with skills from the default `skills/` dir. grey-matter's plugin depends on it as
`tkt@tkt` (this marketplace), so keep the plugin and marketplace name `tkt`. When a change touches skills or
the CLI surface the skills describe:

- **New or changed skill under `skills/`:** no manifest edit. The plugin picks it up as is.
- **Existing cached Claude installs need a version bump.** New installs from a
  main-backed marketplace can receive current main; version fields do not snapshot
  source. Stable/pinned delivery follows the [release contract](docs/releasing.md).
  Local preparation updates Cargo/lock and all plugin manifests from one approved version.
  `tests/plugin_manifests.rs` fails the gate if any drift from `Cargo.toml`. Never
  hand-edit `version` outside a release.
- **Refresh this machine's install** as part of Deploy. Use a marketplace added from the
  local checkout (`claude plugin marketplace add <path-to-tkt>`, once). Deploy verifies
  the enabled CLI row's `readFromFolder` and `folderVersion` match this checkout; update
  Claude/register the correct marketplace if that observation fails. Current Claude
  loads the folder directly, so reload plugins or start a new session after edits.
  Audit uses that active folder rather than a historical cache. For a GitHub
  install, run `claude plugin marketplace update tkt`, then `claude plugin update tkt@tkt`
  (cached updates depend on the manifest version; new default-ref installs can
  receive unreleased files).
- **One install path per harness (guarded).** A skill must never load twice in one harness.
  With the plugin installed, `tools/deploy.mjs` skips every root Claude Code reads (any
  `.claude/skills`, and a project `.agents/skills`). It also removes copies it put there
  earlier and never touches foreign dirs. `--allow-dual` overrides this; don't use it in a
  normal deploy. Any new deploy path, or a new harness root that Claude Code reads, must keep
  this guard (`tests/deploy_skills.rs` covers it).
- **Check a manifest or skill change** with `claude plugin validate .claude-plugin/plugin.json --strict`
  and `claude --plugin-dir . plugin details tkt` (lists the skills, no model call).

All three gate checks must pass with zero warnings before presenting work as done.

## tkt CLI (the product)

```bash
tkt --version                                     # print version
tkt ready [--json]                                # frontier: open + deps done + env match
tkt context [TAGS...] [--clear]                   # set/show/clear active tag context for scoped work
tkt migrate [--from FORMAT] [--detect] [--confirm] # convert foreign ticket schemas (tk → tkt; preview first)
tkt new <slug> --title "..." [--spec S] [--blocked-by NN,NN] [--priority P] [--env E] [--status S] [--tags T] [--validation VC] [--requires R]
tkt batch <slug[:title]>... [--spec S] [--blocked-by IDS] [--priority P] [--env E] [--status S] [--tags T] [--validation VC] [--requires R]
tkt claim <id>                                    # status→in_progress, pushed
tkt close <id> [--note "..."] [--resolution "..."] [--ac N,N] [--check-all] [--force] [--evidence "..." (repeatable: one per validation criterion, or N=text)]
tkt edit <id> [--title T] [--blocked-by IDS] [--priority P|''] [--env E|''] [--spec S|''] [--status S] [--ac N,N] [--validation VC]
tkt renumber <old> <new> [--file NAME]            # birth-window only
tkt query [--status S] [--priority P]             # full corpus as JSON Lines (filterable)
tkt blocked                                       # open tickets with unsatisfied deps
tkt capabilities                                  # machine-readable JSON feature manifest
tkt rebase [--dry-run]                            # resolve ID collisions with upstream
tkt audit [--strict] [--brief] [--deep]           # closure quality check; --deep adds evidence/template analysis
tkt sync-plan [--strict] [--brief] [plan]         # report drift (advisory, exit 0)
tkt sync-plan --check [--strict] [--brief] [plan]  # CI gate: exit 1 on drift
tkt sync-plan --fix [--dry-run] [--brief] [plan]  # fix derivable columns (default plan: docs/plan.md)
tkt validate [--strict] [--brief] [--fix [--dry-run]]  # contract + cycle + decay findings
tkt lint [--check] [IDs...]                       # normalize frontmatter style + blocked_by id refs
tkt doctor [<path>] [--strict] [--fix]                    # health check (single or cross-project)
tkt init [--target T] [--all] [--write [FILE]] [--agent-only]  # scaffold project + agent instructions
tkt config [--set K=V] [--get K] [--unset K] [--list] [--show]  # user + project config
tkt telemetry [--enable|--disable|--status|--show [--all]|--clear]  # manage local telemetry
```

### Global flags

| Flag | Effect |
|------|--------|
| `-q` / `--quiet` | Suppress confirmations, emit only essential data |
| `-o json` / `--output=json` | Structured JSON output (success to stdout, errors to stderr) |
| `--dry-run` | Preview mutations without writing |
| `--color=always\|never\|auto` | Control ANSI color output (default: auto) |

### Priority levels

`urgent` > `high` > `medium` (default) > `low`. Frontier sorts by priority bucket then ID.

### Status values

`backlog` (parked, excluded from frontier) → `open` → `in_progress` → `done`

Reach `done` only via `tkt close` — editing a ticket's `status` to `done` by hand bypasses the close gates (ACs, resolution, evidence) and is flagged by `tkt validate`.

New tickets default to `open` (frontier-eligible); reserve `--status backlog` for work deliberately deferred out of the current cycle. Set `--tags <stream>` at creation in multi-stream projects — tags are the primary scoping mechanism for `tkt ready`/`tkt context`.

### Configuration

- **User config**: `~/.config/tkt/config.toml` — debug mode, format preferences
- **Project config**: `.tickets/config.toml` — committed to repo, shared by contributors
  - `[close]` require_resolution, require_checked_acs, require_validation_criteria, require_validation_evidence, allow_force
  - `[validate]` strict, missing_resolution_before_id (project-specific legacy ratchet)
  - `[ready]` default_env (legacy single capability; prefer `[machine] capabilities`)
  - `[priority]` warn_unknown
  - `[new]` default_priority
  - `[push]` enabled (set false for local-only repos)
  - `[machine]` capabilities (comma-separated canonical eligibility set for `requires` matching)

### Environment variables

| Var | Effect |
|-----|--------|
| `TKT_DEBUG=1\|true\|log\|file` | Debug output to the local debug log (stderr remains clean) |
| `TKT_DEBUG=stderr\|human` | Human-readable debug output to stderr |
| `TKT_DEBUG=json` | Debug output as JSON Lines on stderr |
| `TKT_ASCII=1` | ASCII-only symbols (✓→[ok], ✗→[err], ⚠→[warn]) |
| `NO_COLOR=1` | Disable ANSI color |
| `TKT_ADVICE=0` | Disable advisory hints (e.g. the batch nudge); `off`/`false` also accepted |
| `CREW_ENV` | Filter frontier by env (corp/personal) |
| `DO_NOT_TRACK=1` | Disable telemetry |
| `TKT_NO_USER_CONFIG=1` | Skip user config file (for testing — prevents ambient config leaking into child processes) |

## Architecture Decisions

- **Shell out to git** (not libgit2): full SSH/HTTPS auth compat, matches gh CLI pattern, simplest v1
- **TicketFile + Ticket split**: TicketFile owns raw frontmatter for surgical edits; Ticket provides typed, validated fields (Status enum, zero-cost &str access). Mutations go through `.file`, reads use typed fields directly.
- **Custom frontmatter parser**: line-based key:value parsing with raw preservation. Supports YAML double-quoted scalar escaping (encode on write, decode on read). Not a full YAML parser — deliberately supports a narrow, round-trip-safe subset.
- **No async**: all operations are sequential (fetch → scan → write → commit → push)
- **Local-only telemetry**: opt-in JSONL file sink, per-project segmentation, session-aware rotation, never blocks CLI
- **LazyLock regex statics**: fixed patterns compiled once via `std::sync::LazyLock`
- **No color crate**: raw ANSI codes + `std::io::IsTerminal` — zero additional dependencies
- **Structured errors**: `DomainError { kind: ErrorKind, message, hint }` with 8-variant `ErrorKind` enum. `domain_bail!(Kind, "msg")` or `domain_bail!(Kind, "msg", hint: "fix")` or `domain_bail!("msg")` (defaults to Validation). JSON envelope emitted to stderr when `-o json`.
- **Facade re-exports**: only re-export types from `core/mod.rs` that callers name directly; types returned by methods but never named (e.g., `AcStats`) stay unexported from the facade to avoid unused-import warnings

## Contract

- Files are the database: `.tickets/{NN}-{slug}.md` with YAML frontmatter
- Tool never manages specification prose (body is user-owned; close appends a Resolution section)
- Push-to-claim: pushed commit = claimed id (race detection on push rejection)
- Exit codes: 0=success, 1=domain failure (not found, conflict, drift), 2=operational crash (I/O, git, parse)
- Output: JSON by default for validate/sync-plan; human by default for ready/new/claim/close/edit (use --json for ready)
- Spike branches: closing from `spike/*` auto-appends branch name to resolution
- Worktree-aware: works from git worktrees (`.tickets/` is part of the checked-out tree)

## Constraints

- Do NOT change the frontmatter contract without updating the tkt-work skill
- Distributed skills (`skills/<name>/`) MUST be self-sufficient for a tkt-only adopter (someone who has tkt but NOT grey-matter or any other skill set): reference only `tkt` commands and sibling `tkt-*` skills, never a non-tkt skill by name (`frontier-work`, `read-handoff`, `to-tickets`, …). tkt is the standalone tool; grey-matter is an opinionated layer that *wraps* tkt's skills. `tools/deploy-skills.sh` lint enforces this (fails the deploy on a foreign-skill reference).
- Do NOT change CLI flags or agent-facing behavior without updating all guidance surfaces (see `.memory/agent-guidance-surfaces.md` for checklist — init snippets, skill, steering, AGENTS.md, README)
- Do NOT implement computed sort overrides for the frontier — priority+ID ordering is intentional and gives explicit control over agent work sequence
- Do NOT add dependencies beyond what's in Cargo.toml without justification
- Do NOT use libgit2/gix for v1 — shell out to git binary
- Do NOT suggest releasing with open tickets — all tickets ship first, or the user explicitly defers them
- Maintain CLI compatibility (same commands, flags, output)
- `cargo clippy` must produce 0 warnings; `cargo fmt --check` must produce no diff
- Integration tests MUST set `DO_NOT_TRACK=1` on child processes (prevents ambient env pollution)
- Unit tests must NOT assert specific consent state (result depends on ambient env vars)
- Windows: `std::fs::rename` replaces an existing file on supported filesystems (verified on this Windows/MSVC environment; Rust uses `MoveFileExW` with a `SetFileInformationByHandle` fallback). Do not delete a destination before an atomic replacement: that would reintroduce a missing-file window. The source and destination must remain on the same filesystem.
- Ticket writes go through `core::atomic_write` (temp-file + rename) so a failed/interrupted write never leaves a torn ticket file; a failed write surfaces as `DomainError { Io }` with a hint (`cannot write <path>: <errno>`), exit 2. `crash: writing …` (now shows the errno) is an fs/permission error (read-only file, locked, read-only worktree) — NOT a validation bug; never hand-edit `status: done` to work around it (that bypasses the close gates — the exact anti-pattern #178/#179 fixed)
- Codex review dispatch: `codex exec --dangerously-bypass-approvals-and-sandbox` (bwrap namespace restriction on this machine; `codex review --base <SHA>` cannot combine --base with custom prompt)
- cargo-dist binary name is `dist` (not `cargo dist`) — `cargo dist --version` will fail; use `dist --version`
- New mutation commands MUST route push through a push-gated path (GitTransaction respects `push.enabled`; direct `git::push_with_retry` calls must check `pcfg.push_enabled` first)
- Windows (this machine): `CARGO_HOME=D:\dev-tools\cargo`, `RUSTUP_HOME=D:\dev-tools\rustup` (relocated off C:). Bare `cargo` resolves to the mise shim first, then to the rustup proxies in `D:\dev-tools\cargo\bin\` (Windows symlinks → `rustup.exe`). Prefer `mise run check` for the gate — it is PATH-order-independent. **Failure mode (seen 2026-08-31):** if `cargo fmt` dies with `os error 193` ("not a valid Win32 application"), the `cargo-fmt.exe`/`rustfmt.exe` proxy symlinks in `D:\dev-tools\cargo\bin\` have been corrupted into 0-byte regular files (cross-drive relocation / AV). `cargo build/test/clippy` still work (clippy's proxy stayed intact, and build bypasses proxies), which masks the break. Repair: delete the 0-byte files and recreate them as symlinks to `rustup.exe` (Developer Mode is on, so no elevation needed), or run `rustup-init --default-toolchain=none -y --no-modify-path`. If corruption recurs, add `D:\dev-tools` to Defender exclusions (needs an elevated shell). Verified working 2026-08-31 after symlink repair.
- After any code change, run `cargo install --path .` BEFORE end-to-end testing with the installed `tkt` binary — `tkt --version` git hash must match HEAD, or you are testing a stale binary (validated 2026-08-26: closed #131 against a stale binary that still reproduced the bug)
- Windows: use `git commit -F <file>` for multi-line or bracket/quote-containing commit messages — inline `-m` gets mangled by PowerShell quoting
- crates.io name availability: use `cargo search <name>`, NOT raw `curl` to the API (returns 403 "violation of API data access policy" for automated requests); a 404 on the web page also confirms availability
- Release CI is generated through `node tools/generate-release-workflow.mjs --write` using cargo-dist 0.32.0. Never hand-edit `.github/workflows/release.yml`; `mise run check` checks real generation. See `docs/release-staging.md`. Staging is build-only; the separate manual `publish.yml` owns reviewed publication. Final candidate/install/audit/staging gates remain open.

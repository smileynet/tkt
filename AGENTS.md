# AGENTS.md

## Project

tkt — Git-native ticket CLI. Manages `.tickets/` frontmatter-driven work tracking with atomic git operations, race detection, and dependency-graph frontier computation.

Single Rust binary. Originally ported from a Python implementation (now removed). Same interface, same contract.

## Workspace Layout

```
src/
├── main.rs          — entry point
├── cli.rs           — clap derive commands + dispatch
├── color.rs         — color/symbol support (NO_COLOR, --color, TKT_ASCII)
├── config.rs        — user config (~/.config/tkt/) + project config (.tickets/config.toml)
├── telemetry.rs     — consent, session tracking, JSONL sink, rotation
├── mutation.rs      — MutationContext (push-gated lifecycle for existing-ticket mutations)
├── renumber.rs      — RenumberPlan (pure ID remapping: plan + apply)
├── audit.rs         — pure audit rules (injectable deps, no I/O)
├── core/
│   ├── mod.rs       — re-exports
│   ├── ticket.rs    — TicketFile (raw editor + typed mutations) + Ticket (typed domain)
│   └── validate.rs  — input validation (slugs, free text, IDs, enums)
├── findings.rs      — validation rules, Finding struct, output formatting
├── transaction.rs   — GitTransaction (allocation: fetch→scan→commit→push→retry)
└── git.rs           — git subprocess wrapper (fetch, commit, push, remote scanning)
tests/
├── integration.rs   — integration tests (tempdir repos, race scenarios, telemetry, debug)
└── parity/          — historical Python parity comparison harness
.memory/CONTEXT.md   — project glossary
.tickets/            — tkt's own tickets (dogfooding)
.references/         — cloned reference repos (gitignored): cargo-release, git-cliff, release-plz
TELEMETRY.md         — transparency document for telemetry collection
```

## Commands

```bash
cargo build                    # debug build
cargo build --release          # release build (stripped, LTO)
cargo test                     # all tests
cargo test -- --nocapture      # with output
cargo clippy                   # lint (must be 0 warnings)
cargo fmt                      # format (must produce no diff)
```

### Verification gate (run before every commit)

```bash
cargo fmt && cargo clippy --all-targets && cargo test
```

All three must pass with zero warnings before presenting work as done.

## tkt CLI (the product)

```bash
tkt --version                                     # print version
tkt ready [--json]                                # frontier: open + deps done + env match
tkt new <slug> --title "..." [--spec S] [--blocked-by NN,NN] [--priority P] [--env E] [--status S]
tkt batch <slug[:title]>... [--spec S] [--blocked-by IDS] [--priority P] [--env E]
tkt claim <id>                                    # status→in_progress, pushed
tkt close <id> [--note "..."] [--resolution "..."] [--ac N,N] [--check-all] [--force]
tkt edit <id> [--title T] [--blocked-by IDS] [--priority P|''] [--env E|''] [--spec S|''] [--status S] [--ac N,N]
tkt renumber <old> <new> [--file NAME]            # birth-window only
tkt query [--status S] [--priority P]             # full corpus as JSON Lines (filterable)
tkt blocked                                       # open tickets with unsatisfied deps
tkt capabilities                                  # machine-readable JSON feature manifest
tkt rebase [--dry-run]                            # resolve ID collisions with upstream
tkt audit [--strict] [--brief]                    # closure quality check
tkt sync-plan --check [--strict] [--brief] [plan] # report drift
tkt sync-plan --fix [--strict] [--brief] [plan]   # fix derivable columns
tkt validate [--strict] [--brief]                 # contract + cycle + decay findings
tkt config [--set K=V] [--get K] [--unset K] [--list] [--show]  # user + project config
tkt telemetry [--enable|--disable|--status|--show|--clear]  # manage local telemetry
```

### Global flags

| Flag | Effect |
|------|--------|
| `-q` / `--quiet` | Suppress confirmations, emit only essential data |
| `--color=always\|never\|auto` | Control ANSI color output (default: auto) |

### Priority levels

`urgent` > `high` > `medium` (default) > `low`. Frontier sorts by priority bucket then ID.

### Status values

`backlog` (parked, excluded from frontier) → `open` → `in_progress` → `done`

### Configuration

- **User config**: `~/.config/tkt/config.toml` — debug mode, format preferences
- **Project config**: `.tickets/config.toml` — committed to repo, shared by contributors
  - `[close]` require_resolution, require_checked_acs
  - `[validate]` strict
  - `[ready]` default_env
  - `[priority]` warn_unknown
  - `[new]` default_priority
  - `[push]` enabled (set false for local-only repos)

### Environment variables

| Var | Effect |
|-----|--------|
| `TKT_DEBUG=1\|json` | Debug output to stderr |
| `TKT_ASCII=1` | ASCII-only symbols (✓→[ok], ✗→[err], ⚠→[warn]) |
| `NO_COLOR=1` | Disable ANSI color |
| `CREW_ENV` | Filter frontier by env (corp/personal) |
| `DO_NOT_TRACK=1` | Disable telemetry |

## Architecture Decisions

- **Shell out to git** (not libgit2): full SSH/HTTPS auth compat, matches gh CLI pattern, simplest v1
- **TicketFile + Ticket split**: TicketFile owns raw frontmatter for surgical edits; Ticket provides typed, validated fields (Status enum, zero-cost &str access). Mutations go through `.file`, reads use typed fields directly.
- **Custom frontmatter parser**: line-based key:value parsing with raw preservation. Supports YAML double-quoted scalar escaping (encode on write, decode on read). Not a full YAML parser — deliberately supports a narrow, round-trip-safe subset.
- **No async**: all operations are sequential (fetch → scan → write → commit → push)
- **Local-only telemetry**: opt-in JSONL file sink, per-project segmentation, session-aware rotation, never blocks CLI
- **LazyLock regex statics**: fixed patterns compiled once via `std::sync::LazyLock`
- **No color crate**: raw ANSI codes + `std::io::IsTerminal` — zero additional dependencies
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

- Do NOT change the frontmatter contract without updating frontier-work steering
- Do NOT add dependencies beyond what's in Cargo.toml without justification
- Do NOT use libgit2/gix for v1 — shell out to git binary
- Maintain CLI compatibility (same commands, flags, output)
- `cargo clippy` must produce 0 warnings; `cargo fmt --check` must produce no diff
- Integration tests MUST set `DO_NOT_TRACK=1` on child processes (prevents ambient env pollution)
- Unit tests must NOT assert specific consent state (result depends on ambient env vars)
- Windows: `std::fs::rename` cannot overwrite — always delete destination before rename
- Codex review dispatch: `codex exec --dangerously-bypass-approvals-and-sandbox` (bwrap namespace restriction on this machine; `codex review --base <SHA>` cannot combine --base with custom prompt)
- cargo-dist binary name is `dist` (not `cargo dist`) — `cargo dist --version` will fail; use `dist --version`
- New mutation commands MUST route push through a push-gated path (GitTransaction respects `push.enabled`; direct `git::push_with_retry` calls must check `pcfg.push_enabled` first)

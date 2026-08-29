# AGENTS.md

## Project

tkt — Git-native ticket CLI. Manages `.tickets/` frontmatter-driven work tracking with atomic git operations, race detection, and dependency-graph frontier computation.

Single Rust binary. Originally ported from a Python implementation (now removed). Same interface, same contract.

## Workspace Layout

See `.memory/specs/workspace-layout.md` for the full source tree.

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
cargo fmt && cargo clippy --all-targets && cargo test
```

### Deploy (run after pulling or making changes)

```bash
cargo build --release && cargo install --path . && bash tools/deploy-skills.sh
```

All three gate checks must pass with zero warnings before presenting work as done.

## tkt CLI (the product)

```bash
tkt --version                                     # print version
tkt ready [--json]                                # frontier: open + deps done + env match
tkt context [TAGS...] [--clear]                   # set/show/clear active tag context for scoped work
tkt migrate [--from FORMAT] [--detect]            # convert foreign ticket schemas (tk → tkt)
tkt new <slug> --title "..." [--spec S] [--blocked-by NN,NN] [--priority P] [--env E] [--status S] [--tags T] [--validation VC] [--requires R]
tkt batch <slug[:title]>... [--spec S] [--blocked-by IDS] [--priority P] [--env E] [--status S] [--tags T] [--validation VC] [--requires R]
tkt claim <id>                                    # status→in_progress, pushed
tkt close <id> [--note "..."] [--resolution "..."] [--ac N,N] [--check-all] [--force] [--evidence "..."]
tkt edit <id> [--title T] [--blocked-by IDS] [--priority P|''] [--env E|''] [--spec S|''] [--status S] [--ac N,N] [--validation VC]
tkt renumber <old> <new> [--file NAME]            # birth-window only
tkt query [--status S] [--priority P]             # full corpus as JSON Lines (filterable)
tkt blocked                                       # open tickets with unsatisfied deps
tkt capabilities                                  # machine-readable JSON feature manifest
tkt rebase [--dry-run]                            # resolve ID collisions with upstream
tkt audit [--strict] [--brief]                    # closure quality check
tkt sync-plan --check [--strict] [--brief] [plan] # report drift
tkt sync-plan --fix [--strict] [--brief] [plan]   # fix derivable columns
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
  - `[validate]` strict
  - `[ready]` default_env
  - `[priority]` warn_unknown
  - `[new]` default_priority
  - `[push]` enabled (set false for local-only repos)
  - `[machine]` capabilities (comma-separated list of workstation capabilities for requires matching)

### Environment variables

| Var | Effect |
|-----|--------|
| `TKT_DEBUG=1\|json` | Debug output (`1`=file, `json`=stderr JSONL, `stderr`=stderr human) |
| `TKT_ASCII=1` | ASCII-only symbols (✓→[ok], ✗→[err], ⚠→[warn]) |
| `NO_COLOR=1` | Disable ANSI color |
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

- Do NOT change the frontmatter contract without updating frontier-work steering
- Do NOT change CLI flags or agent-facing behavior without updating all guidance surfaces (see `.memory/agent-guidance-surfaces.md` for checklist — init snippets, skill, steering, AGENTS.md, README)
- Do NOT implement computed sort overrides for the frontier — priority+ID ordering is intentional and gives explicit control over agent work sequence
- Do NOT add dependencies beyond what's in Cargo.toml without justification
- Do NOT use libgit2/gix for v1 — shell out to git binary
- Do NOT suggest releasing with open tickets — all tickets ship first, or the user explicitly defers them
- Maintain CLI compatibility (same commands, flags, output)
- `cargo clippy` must produce 0 warnings; `cargo fmt --check` must produce no diff
- Integration tests MUST set `DO_NOT_TRACK=1` on child processes (prevents ambient env pollution)
- Unit tests must NOT assert specific consent state (result depends on ambient env vars)
- Windows: `std::fs::rename` cannot overwrite — always delete destination before rename
- Codex review dispatch: `codex exec --dangerously-bypass-approvals-and-sandbox` (bwrap namespace restriction on this machine; `codex review --base <SHA>` cannot combine --base with custom prompt)
- cargo-dist binary name is `dist` (not `cargo dist`) — `cargo dist --version` will fail; use `dist --version`
- New mutation commands MUST route push through a push-gated path (GitTransaction respects `push.enabled`; direct `git::push_with_retry` calls must check `pcfg.push_enabled` first)
- Windows (this machine): `cargo`, `cargo fmt`, and `rustfmt` all work directly on PATH from `C:\Users\uosmi\.cargo\bin` (rustup shims, verified 2026-08-28). The old `D:\dev-tools\...` direct-path workaround is obsolete — mise shims are no longer in the toolchain path
- After any code change, run `cargo install --path .` BEFORE end-to-end testing with the installed `tkt` binary — `tkt --version` git hash must match HEAD, or you are testing a stale binary (validated 2026-08-26: closed #131 against a stale binary that still reproduced the bug)
- Windows: use `git commit -F <file>` for multi-line or bracket/quote-containing commit messages — inline `-m` gets mangled by PowerShell quoting
- crates.io name availability: use `cargo search <name>`, NOT raw `curl` to the API (returns 403 "violation of API data access policy" for automated requests); a 404 on the web page also confirms availability

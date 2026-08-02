# AGENTS.md

## Project

tkt — Git-native ticket CLI. Manages `.tickets/` frontmatter-driven work tracking with atomic git operations, race detection, and dependency-graph frontier computation.

Single Rust binary. Originally ported from a Python implementation (now removed). Same interface, same contract.

## Workspace Layout

```
src/
├── main.rs          — entry point
├── cli.rs           — clap derive commands + dispatch
├── telemetry.rs     — consent, session tracking, JSONL sink, rotation
├── core/
│   ├── mod.rs       — re-exports
│   ├── ticket.rs    — TicketFile (raw editor) + Ticket (typed domain), Status/Env/Priority enums
│   └── validate.rs  — input validation (slugs, free text, IDs, enums)
├── findings.rs      — validation rules, Finding struct, output formatting
├── transaction.rs   — GitTransaction (allocation: fetch→scan→commit→push→retry)
└── git.rs           — git subprocess wrapper (fetch, commit, push, remote scanning)
tests/
├── integration.rs   — integration tests (tempdir repos, race scenarios, telemetry, debug)
└── parity/          — historical Python parity comparison harness
.memory/CONTEXT.md   — project glossary
.tickets/            — tkt's own tickets (dogfooding)
TELEMETRY.md         — transparency document for telemetry collection
```

## Commands

```bash
cargo build                    # debug build
cargo build --release          # release build (stripped, LTO)
cargo test                     # all tests (40 unit + 25 integration)
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
tkt new <slug> --title "..." [--spec S] [--blocked-by NN,NN] [--priority high] [--env E]
tkt batch <slug[:title]>... [--spec S] [--blocked-by IDS] [--priority P] [--env E]
tkt claim <id>                                    # status→in_progress, pushed
tkt close <id> [--note "..."] [--ac N,N]          # status→done
tkt edit <id> [--title T] [--blocked-by IDS] [--priority high|''] [--env E|''] [--spec S|''] [--ac N,N]
tkt renumber <old> <new> [--file NAME]            # birth-window only
tkt query                                         # full corpus as JSON Lines
tkt sync-plan --check [--strict] [--brief] [plan] # report drift
tkt sync-plan --fix [--strict] [--brief] [plan]   # fix derivable columns
tkt validate [--strict] [--brief]                 # contract + cycle + decay findings
tkt telemetry [--enable|--disable|--status|--show|--clear]  # manage local telemetry
```

## Architecture Decisions

- **Shell out to git** (not libgit2): full SSH/HTTPS auth compat, matches gh CLI pattern, simplest v1
- **TicketFile + Ticket split**: TicketFile owns raw frontmatter for surgical edits; Ticket provides typed, validated fields (Status enum, zero-cost &str access). Mutations go through `.file`, reads use typed fields directly.
- **Custom frontmatter parser**: line-based key:value parsing with raw preservation. Supports YAML double-quoted scalar escaping (encode on write, decode on read). Not a full YAML parser — deliberately supports a narrow, round-trip-safe subset.
- **No async**: all operations are sequential (fetch → scan → write → commit → push)
- **Local-only telemetry**: opt-in JSONL file sink, per-project segmentation, session-aware rotation, never blocks CLI
- **LazyLock regex statics**: fixed patterns compiled once via `std::sync::LazyLock`

## Contract

- Files are the database: `.tickets/{NN}-{slug}.md` with YAML frontmatter
- Tool never manages specification prose (body is user-owned; close appends a Resolution section)
- Push-to-claim: pushed commit = claimed id (race detection on push rejection)
- Exit codes: 0=success, 1=domain failure (not found, conflict, drift), 2=operational crash (I/O, git, parse)
- Output: JSON by default for validate/sync-plan; human by default for ready/new/claim/close/edit (use --json for ready)

## Constraints

- Do NOT change the frontmatter contract without updating frontier-work steering
- Do NOT add dependencies beyond what's in Cargo.toml without justification
- Do NOT use libgit2/gix for v1 — shell out to git binary
- Maintain CLI compatibility (same commands, flags, output)
- `cargo clippy` must produce 0 warnings; `cargo fmt --check` must produce no diff
- Integration tests MUST set `DO_NOT_TRACK=1` on child processes (prevents ambient env pollution)
- Unit tests must NOT assert specific consent state (result depends on ambient env vars)
- Windows: `std::fs::rename` cannot overwrite — always delete destination before rename
- Codex review dispatch: `codex review --base <SHA>` (cannot combine --base with custom prompt)

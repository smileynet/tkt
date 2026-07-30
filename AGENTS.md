# AGENTS.md

## Project

tkt — Git-native ticket CLI. Manages `.tickets/` frontmatter-driven work tracking with atomic git operations, race detection, and dependency-graph frontier computation.

Rebuilt in Rust from the original Python implementation (crew-research/tools/tkt). Same interface, same contract, single binary distribution.

## Workspace Layout

```
src/
├── main.rs          — entry point
├── cli.rs           — clap derive commands + dispatch
├── core/
│   ├── mod.rs       — re-exports
│   ├── ticket.rs    — Ticket struct, frontmatter parser, validation, escape helpers
│   └── validate.rs  — input validation (slugs, free text, IDs, enums)
└── git.rs           — git subprocess wrapper (fetch, commit, push, remote scanning)
tests/
├── integration.rs   — integration tests (tempdir repos, race scenarios)
└── parity/          — Python parity comparison harness
.memory/CONTEXT.md   — project glossary
.tickets/            — tkt's own tickets (dogfooding)
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

## tkt CLI (the product)

```bash
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
```

## Architecture Decisions

- **Shell out to git** (not libgit2): full SSH/HTTPS auth compat, matches gh CLI pattern, simplest v1
- **Custom frontmatter parser**: line-based key:value parsing with raw preservation for surgical edits. Supports YAML double-quoted scalar escaping (encode on write, decode on read). Not a full YAML parser — deliberately supports a narrow, round-trip-safe subset.
- **No async**: all operations are sequential (fetch → scan → write → commit → push)
- **Integration tests over unit tests**: the value is in the git interaction, not pure logic
- **LazyLock regex statics**: fixed patterns compiled once via `std::sync::LazyLock`

## Contract (from crew-research spec)

- Files are the database: `.tickets/{NN}-{slug}.md` with YAML frontmatter
- Tool never manages specification prose (body is user-owned; close appends a Resolution section)
- Push-to-claim: pushed commit = claimed id (race detection on push rejection)
- Exit codes: 0=success, 1=domain failure (not found, conflict, drift), 2=operational crash (I/O, git, parse)
- Output: JSON by default for validate/sync-plan; human by default for ready/new/claim/close/edit (use --json for ready)

## Constraints

- Do NOT change the frontmatter contract without updating crew-research's frontier-work steering
- Do NOT add dependencies beyond what's in Cargo.toml without justification
- Do NOT use libgit2/gix for v1 — shell out to git binary
- Maintain CLI compatibility with the Python tkt (same commands, flags, output)
- `cargo clippy` must produce 0 warnings; `cargo fmt --check` must produce no diff

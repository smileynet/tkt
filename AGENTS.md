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
│   └── ticket.rs    — Ticket struct, frontmatter parser, validation
└── git.rs           — git subprocess wrapper (fetch, commit, push)
tests/               — integration tests (tempdir repos)
.memory/CONTEXT.md   — project glossary
.tickets/            — tkt's own tickets (dogfooding)
```

## Commands

```bash
cargo build                    # debug build
cargo build --release          # release build (stripped, LTO)
cargo test                     # all tests
cargo test -- --nocapture      # with output
cargo clippy                   # lint
cargo fmt                      # format
```

## tkt CLI (the product)

```bash
tkt ready                                     # frontier: open + deps done + env match
tkt new <slug> --title "..." [--spec S] [--blocked-by NN,NN] [--priority high]
tkt batch <slug[:title]>... [--spec S] [--blocked-by IDS]
tkt claim <id>                                # status→in_progress, pushed
tkt close <id> [--note "..."] [--ac N,N]      # status→done
tkt edit <id> [--blocked-by IDS] [--priority high|''] [--env E|'']
tkt renumber <old> <new> [--file NAME]        # birth-window only
tkt sync-plan --check [--strict] [--brief] [plan]
tkt sync-plan --fix [--strict] [--brief] [plan]
tkt validate [--brief]                        # contract + decay findings
```

## Architecture Decisions

- **Shell out to git** (not libgit2): full SSH/HTTPS auth compat, matches gh CLI pattern, simplest v1
- **serde_yaml for frontmatter**: type-safe deserialization, preserves unknown fields via raw string manipulation for writes
- **No async**: all operations are sequential (fetch → scan → write → commit → push)
- **Integration tests over unit tests**: the value is in the git interaction, not pure logic

## Contract (from crew-research spec)

- Files are the database: `.tickets/{NN}-{slug}.md` with YAML frontmatter
- Tool never manages prose (body is the spec)
- Push-to-claim: pushed commit = claimed id (race detection on push rejection)
- Exit codes: 0=success, 1=failure/drift, 2=crash
- Output: JSON by default for machine consumption, --brief for humans

## Constraints

- Do NOT change the frontmatter contract without updating crew-research's frontier-work steering
- Do NOT add dependencies beyond the 5 in Cargo.toml without justification
- Do NOT use libgit2/gix for v1 — shell out to git binary
- Maintain CLI compatibility with the Python tkt (same commands, flags, output)

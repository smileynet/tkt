---
created_at: 2026-07-29T06:13:00-07:00
base_commit: 850576f
handoff_key: tkt-rust-production-readiness
---

# Handoff

## Objective
Ship tkt v0.1.0 as a production Rust CLI — complete cargo-dist setup, publish to crates.io, then iterate on parity gaps and improvements.

## Constraints
- CLI must remain 100% compatible with the Python tkt (same commands, flags, output format, exit codes)
- crew-research still has the Python tkt at `tools/tkt/` — it stays until the Rust binary is published and proven
- Single binary, no runtime dependencies beyond `git` on PATH
- CREW_ENV=personal on this machine (full tool access)

## Current State
- Repo: https://github.com/smileynet/tkt (5 commits, main branch)
- All 10 commands implemented and verified against crew-research's live .tickets/ corpus (69 tickets)
- 15 tests passing (4 unit + 11 integration), ~5s total
- Release binary: 2.35 MB, 19.6ms startup, 98ms `tkt ready` on 69 tickets
- cargo-dist config in Cargo.toml (targets: linux x86_64/aarch64, macOS x86_64/aarch64, Windows x86_64)
- Race detection: push_with_retry with renumber-on-lost-race in `new`

## Prior Decisions
- Rust over Go: fastembed-rs for future recall integration, smaller binaries, type safety for frontmatter parsing
- Shell out to git binary (not libgit2): full SSH/HTTPS auth compat, matches gh CLI pattern
- Raw frontmatter parsing (not serde_yaml): surgical edits that preserve unknown fields and formatting
- 4 crate dependencies only: clap, regex, anyhow, thiserror

## Next Steps

### Immediate (ship v0.1.0)
1. Install cargo-dist: `cargo install cargo-dist`
2. Run `cargo dist init` — generates `.github/workflows/release.yml`
3. Commit the workflow, tag v0.1.0: `git tag v0.1.0 && git push --tags`
4. Verify: GitHub Actions builds all 5 platform binaries + generates install scripts
5. Publish to crates.io: `cargo publish`
6. Test install: `cargo binstall tkt` (or `cargo install tkt`) on a clean machine

### Parity gaps (Python features not yet in Rust)
7. Remote ticket name scanning for race detection (Python's `gitio.remote_ticket_names`)
8. Preflight race check (Python fetches + inspects upstream status BEFORE editing — prevents same-second byte-identical commits)
9. R18 input validation: Windows reserved device names, free-text validation (no quotes/backslashes/newlines in titles)
10. `--json` flag for `ready` (partial — currently emits simplified JSON, Python emits full row objects)
11. Cycle detection in `validate` (Python has `_cycles` with DFS; Rust `validate` doesn't check for cycles yet)

### Improvements over Python
12. Consider: `--dry-run` for new/claim/close (show what would happen without pushing)
13. Consider: colored output for frontier (priority tickets highlighted)
14. Consider: `tkt status` — one-line summary of corpus health (N open, N in_progress, N done, N blocked)
15. Performance: compile regex patterns once (currently re-compiled per call)

### Integration
16. Update crew-research install docs to point at `cargo install tkt` as primary
17. Remove `tools/tkt/` from crew-research once Rust tkt is proven stable (~2 weeks)
18. Update crew-research AGENTS.md to drop the Python fallback note

## Fog
- Whether cargo-dist's generated workflow works out of the box for this project (no workspace, single binary — should be straightforward)
- Whether crates.io name "tkt" is available (check before publishing)
- Whether the 48 Python integration tests reveal edge cases the 11 Rust tests don't cover

## Evidence
- Test suite: `cargo test` (15 tests, ~5s)
- Live corpus validation: run against C:\Users\uosmi\code\crew-research\.tickets\ — produces identical output to Python tkt
- Release build: `cargo build --release` (target\release\tkt.exe, 2.35 MB)

---
created_at: 2026-07-30T06:18:00-07:00
base_commit: 5b51634
handoff_key: tkt-rust-production-readiness
---

# Handoff

## Objective
Ship tkt v0.1.0 as a production Rust CLI — complete the last deepening refactor (#17), then cargo-dist release.

## Constraints
- CLI must remain compatible with Python tkt (same commands, flags, output format, exit codes)
- Single binary, no runtime deps beyond `git` on PATH
- `cargo fmt && cargo clippy --all-targets && cargo test` must pass with zero warnings before every commit

## Prior Decisions
- Shell out to git (not libgit2): full SSH/HTTPS auth compat
- Custom frontmatter parser with YAML double-quoted escaping (encode on write, decode on read via yaml_scalar_unescape)
- Push classification: only "non-fast-forward" / "fetch first" = retryable race; all other push failures are operational (exit 2)
- Hard reset for allocation recovery (not soft reset) — prevents stale index
- Preflight reads remote state via `git show origin/main:<path>` without modifying working tree

## Current State
- 44 tests (22 unit + 22 integration), clippy 0 warnings, fmt clean
- Architecture: `transaction.rs` (GitTransaction for allocation), `findings.rs` (validation rules), helpers (`preflight_mutation`, `check_remote_status`, `commit_and_publish`) in cli.rs
- Tickets done: 02-04, 06-16. Open: #01 (release), #05 (remove Python), #17 (Ticket/TicketFile split)
- Frontier: #17 (no blockers, all deps done)

## Next Steps
1. Implement #17: split `Ticket` into `TicketFile` (raw preservation) + `Ticket` (owned validated fields). id()/title() become `&str` borrows, status becomes enum, blocked_by parsed once at construction. Touches every caller.
2. After #17: ticket #01 — cargo-dist workflow, check crates.io name, tag v0.1.0, verify 5-platform binaries
3. After #01: ticket #05 — remove Python tkt from crew-research

## Fog
- Whether crates.io name "tkt" is available (check before publishing)
- Whether cargo-dist workflow works out of the box (no workspace, single binary — likely fine)
- Ticket #17 might reveal that `TicketFile` needs interior mutability or Rc for the Ticket→TicketFile back-reference during writes

## Evidence
- Test suite: `cargo test` (44 tests)
- Codex reviews: `.scratch/codex-review/01-04.md` (two rounds of full-codebase review)
- Release build: `cargo build --release` (~2.35 MB binary)

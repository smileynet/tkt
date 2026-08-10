---
id: "70"
title: "Extract command implementations from cli.rs into src/commands/"
status: in_progress
blocked_by: []
priority: high
---

# Extract command implementations from cli.rs into src/commands/

## What to build

Split command implementations out of the 80KB `src/cli.rs` god module into focused per-command modules under `src/commands/`.

### Intent

cli.rs currently owns both clap argument definitions AND all 20+ command implementations. A developer adding `tkt init` must edit an 80KB file. A bug in `close` shares search space with `rebase`, `telemetry`, and `config`. The module is shallow in the wrong direction — one interface (run()) but enormous, unrelated implementation surface.

### Context

- `src/cli.rs` contains: `Cli` struct, `Commands` enum, `run()` dispatch, 20+ `cmd_*` functions, shared helpers (`preflight_mutation`, `commit_and_publish`, `success_msg`, `is_quiet`, `project_config`, etc.)
- ADR: "shell out to git" — no conflict, this is about code organization not I/O strategy
- The TicketFile/Ticket split (CONTEXT.md) is already well-separated; this is about the command layer above it
- Upcoming `tkt init` (ticket #68) will need to live somewhere — this establishes the pattern first

### Desired outcome

After this work:
- `src/cli.rs` is ~200 lines: `Cli`, `Commands`, `run()` dispatch, error handling, telemetry setup
- Each command lives in `src/commands/{name}.rs` with a `pub fn run(...) -> Result<i32>` entry point
- Shared mutation helpers live in `src/commands/common.rs` (temporary home until ticket #71 deepens them)
- No behavioral change — same CLI contract, same output, same exit codes

### How to validate

1. `cargo fmt && cargo clippy --all-targets && cargo test` — zero regressions
2. `wc -l src/cli.rs` — under 300 lines
3. `ls src/commands/*.rs | wc -l` — 12+ files (one per command group)
4. Integration tests still pass unmodified (behavioral parity)
5. Each command module is self-contained: reading it explains the full command without jumping back to cli.rs

## Acceptance criteria

- [ ] `src/commands/` module created with per-command files
- [ ] `src/cli.rs` reduced to dispatch + framework concerns only
- [ ] Shared helpers extracted to `src/commands/common.rs`
- [ ] All 48 integration tests pass without modification
- [ ] `cargo clippy` zero warnings
- [ ] No behavioral change (output, exit codes, flags unchanged)

---
created_at: 2026-08-09T19:36:00-07:00
base_commit: 9ba1e59
handoff_key: tkt-architecture-deepening
---

# Handoff

## Objective
Complete ticket #70: extract all command implementations from cli.rs into src/commands/.

## Constraints
- `cargo fmt && cargo clippy --all-targets && cargo test` gate before every commit
- No behavioral change — same CLI contract, same output, same exit codes
- Integration tests must pass unmodified throughout
- DomainError + QUIET now live in main.rs (crate-level visibility)

## Current State
- 9 of 16 commands extracted to src/commands/:
  ✅ ready, claim, close, edit, new, blocked, query, validate, capabilities
- 7 commands still dispatched to local cli.rs functions:
  ❌ batch, renumber, rebase, audit, sync-plan, config, telemetry
- cli.rs still 2460 lines (dead functions not yet deleted)
- Pattern is proven and stable — all 48 tests pass at every checkpoint
- src/commands/common.rs has all shared helpers

## Prior Decisions
- DomainError promoted from cli.rs to main.rs for crate-wide visibility
- domain_bail! macro re-exported from commands::common
- AC regex statics duplicated in close.rs and edit.rs (acceptable until ticket #72 consolidates them)
- flip_ac_boxes duplicated in close.rs and edit.rs (same reason — #72 will unify)

## Next Steps
1. Extract `batch` → src/commands/batch.rs (similar to new, uses GitTransaction)
2. Extract `renumber` → src/commands/renumber.rs
3. Extract `rebase` → src/commands/rebase.rs (largest remaining, ~165 lines)
4. Extract `audit` → src/commands/audit.rs
5. Extract `sync-plan` → src/commands/sync_plan.rs
6. Extract `config` → src/commands/config.rs
7. Extract `telemetry` → src/commands/telemetry.rs
8. Delete dead cmd_* functions from cli.rs
9. Delete unused imports/statics from cli.rs
10. Final verify: `wc -l src/cli.rs` should be ~250 lines
11. Close ticket #70

## Pattern (for each remaining command)
1. Read the cmd_* function from cli.rs
2. Create src/commands/{name}.rs with the function body
3. Replace local helper calls with `crate::commands::common::*`
4. Update cli.rs dispatch: `Commands::X { .. } => crate::commands::x::run(..)`
5. `cargo check && cargo test` — must pass before moving to next

## Evidence
- `tkt ready` → ticket #70 in_progress
- `cargo test` → 48 pass, 0 fail
- `ls src/commands/*.rs` → 17 files (mod.rs + common + 15 command files, 7 still stubs)

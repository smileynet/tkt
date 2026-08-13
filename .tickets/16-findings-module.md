---
id: "16"
title: "move Finding + validation rules into findings module"
status: done
blocked_by: []
---

# Move Finding + validation rules into findings module

## What to build

cmd_validate is 220 lines mixing three concerns: corpus loading, 6 rule implementations (including 90-line DFS cycle detection), and output formatting. Extracting these makes each rule independently testable and the validate command a thin orchestrator.

### Changes needed

1. Create `src/findings.rs` with:
   - `Finding` struct (moved from cli.rs)
   - `print_findings(findings, brief, status)` (moved from cli.rs)
   - `check_status(corpus) → Vec<Finding>`
   - `check_env(corpus) → Vec<Finding>`
   - `check_id_filename(corpus) → Vec<Finding>`
   - `check_duplicate_ids(corpus) → Vec<Finding>`
   - `check_dangling_deps(corpus) → Vec<Finding>`
   - `check_cycles(corpus) → Vec<Finding>`
   - `check_unchecked_acs(corpus) → Vec<Finding>`
2. `cmd_validate` becomes: load corpus → collect parse errors → call each check → determine status → print
3. `cmd_sync_plan` uses the same `Finding` and `print_findings` from the module

### Deletion test

If the findings module were deleted, cycle detection logic reappears inline in cmd_validate (90 lines), and each rule's test requires spinning up a full CLI + git repo instead of just passing a corpus slice.

## Acceptance criteria

- [x] `src/findings.rs` exists with Finding struct and print_findings
- [x] check_cycles is a standalone function testable with a Vec<Ticket>
- [x] cmd_validate is < 40 lines (orchestration only)
- [x] cmd_sync_plan uses Finding and print_findings from the module
- [x] New unit tests: check_cycles with cyclic/acyclic corpus (no git needed)
- [x] All existing integration tests pass unchanged
- [x] cargo clippy clean, cargo fmt clean

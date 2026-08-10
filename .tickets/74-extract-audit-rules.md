---
id: "74"
title: "Move audit rules beside validation findings"
status: open
blocked_by: ["70"]
priority: high
---

# Move audit rules beside validation findings

## What to build

Move audit rule logic from inline `cmd_audit` code into a dedicated module beside the existing `findings.rs` validation rules.

### Intent

`cmd_validate` properly delegates structural checks to `findings.rs`, which produces `Vec<Finding>`. But `cmd_audit` builds its Findings inline in cli.rs: unchecked ACs on done tickets, TBD/missing resolutions, stale WIP detection, and high-priority frontier items. This splits Finding-producing logic between a domain module and command code. The stale-WIP rule also mixes pure rule logic (is this ticket old enough?) with git timestamp lookup (when was the file last committed?), making it untestable without a real git repo.

### Context

- `findings.rs` already defines `Finding { file, rule, message, severity }` and output functions
- `cmd_audit` in cli.rs produces Findings for: resolution quality, AC completeness, stale WIP, frontier priorities
- The stale-WIP check shells out to `git log --format=%at -1 -- <file>` — coupling rule logic to I/O
- `cmd_validate` and `cmd_audit` both consume `Vec<Finding>` and render them identically
- Blocked by #70 because audit moves to its own command module first, then the rules move out of it

### Desired outcome

After this work:
- `src/audit.rs` exports pure rule functions:
  - `check_resolution_quality(corpus) -> Vec<Finding>` — TBD resolutions, missing resolution on done tickets
  - `check_ac_completeness(corpus) -> Vec<Finding>` — unchecked ACs on done tickets
  - `check_stale_wip(corpus, last_commit_fn: impl Fn(&Path) -> Option<u64>) -> Vec<Finding>` — injectable timestamp
  - `check_frontier_health(corpus) -> Vec<Finding>` — high-priority items languishing
- The command module (`commands/audit.rs`) becomes: load corpus → provide git timestamp adapter → call audit checks → print findings
- Audit rules testable with synthetic `Ticket` values and a fake timestamp function

### How to validate

1. `cargo test` — all tests pass
2. Unit tests for each audit rule using constructed `Ticket` values (no git, no tempdir)
3. Stale-WIP tested with a fake `last_commit_fn` that returns a fixed timestamp
4. `tkt audit` produces identical output to current behavior
5. `grep -r "Finding {" src/commands/audit.rs` — zero inline Finding construction (all delegated to audit module)

## Acceptance criteria

- [ ] `src/audit.rs` created with pure rule functions
- [ ] `check_resolution_quality` extracted and unit-tested
- [ ] `check_ac_completeness` extracted and unit-tested
- [ ] `check_stale_wip` extracted with injectable timestamp provider
- [ ] `check_frontier_health` extracted and unit-tested
- [ ] Command module only orchestrates (load → check → print), no inline rule logic
- [ ] Stale-WIP unit test uses fake timestamps (no git)
- [ ] All integration tests pass unchanged
- [ ] Audit output format/content identical to current behavior

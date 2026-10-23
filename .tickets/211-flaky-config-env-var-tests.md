---
id: "211"
title: "config tests race on TKT_NO_USER_CONFIG (set_var/remove_var across parallel tests)"
status: done
blocked_by: []
priority: medium
validation_criteria:
  - "cargo test passes 20 consecutive runs of 'cargo test --bin tkt config::tests' with default parallelism"
tags: ["type:bug"]
---

# config tests race on TKT_NO_USER_CONFIG (set_var/remove_var across parallel tests)

## What to build

`config::tests::test_project_config_defaults` fails intermittently. Seen on 2026-10-23: one
failure in three runs of `cargo test --bin tkt config::tests`, and one red `mise run check`.
The test reports `assertion failed: !cfg.close_require_resolution` at `src/config.rs:561`.
Three tests in `src/config.rs` (around lines 558, 598, 613) call
`std::env::set_var("TKT_NO_USER_CONFIG", "1")` and later `remove_var`. Tests run in parallel
and share the process environment, so one test can clear the variable while another is still
loading config. That lets the ambient user config (`require_resolution = true`) leak in.

Fix: pass the skip-user-config decision as a parameter, or serialize these tests (a shared
`Mutex`, or `serial_test`), so no test depends on process-global env set by another.

## Acceptance criteria

- [x] cargo test passes 20 consecutive runs of 'cargo test --bin tkt config::tests' with default parallelism

## Resolution (2026-10-23)

done: tests call ProjectConfig::load_inner(dir, true) instead of setting the env var (commit above).

### Verification
1. ✓ cargo test passes 20 consecutive runs of 'cargo test --bin tkt config::tests' with default parallelism — "20 consecutive 'cargo test --bin tkt config::tests' runs: 20x 'ok. 10 passed; 0 failed' (before: 1 failure in 3). mise run check green. No test in src/config.rs mutates TKT_NO_USER_CONFIG anymore."

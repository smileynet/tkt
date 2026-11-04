# Telemetry preview regression — 2026-11-04

The initial #191 CLI matrix incorrectly assumed APPDATA/XDG environment variables
isolated Windows `dirs` Known Folders. It invoked dry-run telemetry enable,
disable and clear against the real platform paths. The consent file was observed
afterward with `enabled = false` and this date; clear was executed. There is no
before snapshot: prior consent bytes and any removed event data are unknown.
The user was informed. No guessed restoration was performed. This is an observed
regression and a test-isolation mistake, not a passing dry-run test.

Explicit telemetry CLI mutation cases were removed from integration testing.
Tests must supply private paths directly and must never infer Known Folder
isolation from redirected environment variables. No further ambient consent or
clear operation was used for verification.

Preview now returns before a lazy platform-path resolver can run. A private
table covers enable, disable and clear with existing and absent files, plus a
resolver that panics if invoked. Apply controls exercise atomic consent creation
and replacement, selected data removal, no-data reporting and unrelated-file
preservation. Shared directory removal rejects a linked root and preserves its
foreign target; preview skips recursive removal. Telemetry event recording and
status cleanup also return before mutation in global preview mode.

Focused checks passed: `cargo test private_telemetry --bin tkt` (two tests) and
`cargo test --bin tkt directory_removal_preserves` (one test). These tests use only
private temporary paths and make no assertion about ambient consent state.

The parent #191 matrix and complete filesystem/Git executor consolidation remain
separate work. This repair does not complete those acceptance criteria.

Fresh final mise run check: PASS 512 Rust tests (including one pending parent191
integration matrix), 42 Node checks, clean formatting and zero clippy warnings.
SKIP one existing Windows relative-link fixture. First gate failed clippy on a
useless conversion and test-module placement; both were corrected before this
fresh run. Guidance coverage45/355 and diff whitespace checks pass. Independent
critic accepted the final source and lazy-resolver proof before closure.

# Telemetry preview regression — 2026-11-04

The initial #191 CLI matrix incorrectly assumed APPDATA/XDG environment variables
isolated Windows `dirs` Known Folders. It invoked dry-run telemetry enable,
disable and clear against the real platform paths. The consent file was observed
afterward with `enabled = false` and this date; clear was executed. There is no
before snapshot: prior consent bytes and any removed event data are unknown.
The user was informed. No guessed restoration was performed. This is an observed
regression and a test-isolation mistake, not a passing dry-run test.

The newly added telemetry CLI matrix cases were removed first.
Tests must supply private paths directly and must never infer Known Folder
isolation from redirected environment variables. A subsequent gate exposed two
pre-existing exceptions that this first repair missed: the integration consent
cycle removed DO_NOT_TRACK and applied ambient enable/disable/status, while the
quiet-JSON suite applied ambient enable/disable/clear with ineffective APPDATA/XDG
redirection. Those fixtures ran during intervening gates, so the earlier statement
that no further ambient verification occurred was incorrect. The ticket was
reopened and the user informed. These explicit CLI cases are now removed; private
unit controls retain apply coverage. The integration environment helper forces
DO_NOT_TRACK=1 after overrides, and debug config fixtures use private state roots.

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
critic accepted the runtime source and lazy-resolver proof before the initial
closure. That gate did not prove complete fixture isolation; the ticket was
reopened for the legacy fixture correction described above.

After correction, fresh `mise run check` PASS: 516 Rust tests, 42 Node checks,
clean formatting and zero clippy warnings. One Rust subprocess entry is
intentionally ignored in direct suite execution and exercised by the hook
control; one existing Windows relative-link Node fixture is skipped. This run
also includes the separately reviewed #260 filesystem changes and #191 matrix.
Focused quiet-JSON checks passed all three tests; the human-debug test passed
with explicit private state and debug-format isolation. Critic accepted the
legacy fixture removal and helper isolation before renewed closure.

The affected consent file remained 68 bytes with its previously observed
modification time during this gate; the debug log remained 42,669 bytes with its
previous modification time. These observations do not recover earlier contents.
Retained user state: consent68B, debug log42,669B, three telemetry files4,015B,
and an existing update-cache17B dated before this session. None was deleted or
guessed-restored. Private test repositories are cleaned by their owners.

# Prepared filesystem phases — 2026-11-04

Command-owned text effects now use the shared `FilePlan` dispatcher. Its prepared
variants distinguish snapshot replacement, no-clobber creation, relocation,
owned removal, explicit context-entry unlinking and directory operations.
Preparation validates snapshots and ownership without writing, including Preview;
Apply rechecks at publication boundaries. Parsed tickets retain the original raw
snapshot. `core::atomic_write` and `atomic_create` remain the final publication
primitives, reached by commands through this dispatcher.

Migration/renumber relocation validates both sides, publishes the destination
without clobbering, then removes the matching source. Owned removal quarantines
the entry and verifies it again; foreign arrivals are restored without clobbering
or retained at a reported staging path. Context clear deliberately remains a
named-entry unlink that does not follow a symlink. Command output is unchanged.

Focused controls cover stale parsed tickets, prepared stale replacements, late
creation collisions, relocation preview/source/target/success cases, linked
directory refusal and foreign recovery. Existing migration, renumber, hooks,
import, note and lifecycle fixtures remain part of the full gate.

The initial gate caught a context-clear compatibility regression: a linked
context entry was rejected. The distinct unlink effect repaired it; all four
`cargo test --test local_effects` fixtures then passed. Another gate failed a
human-debug configuration assertion; its focused rerun passed after test state
and debug-format isolation was made explicit. These failed/intermediate gates
are not final passing evidence. Legacy telemetry fixture isolation was reopened
as #259 and corrected before the next gate.

Checks are phase-local, not cross-file transactions or external-editor locks.
Preparation errors stop a phase before its first write; later I/O failures can
leave earlier publications, and relocation failure may retain both versions.
Ancestor links remain filesystem-resolved. Git/process and supporting
lock/index/cadence/telemetry/cache phases remain #261, so parent #191 stays open.

Final fresh `mise run check` PASS: 516 Rust tests, 42 Node checks, clean formatting
and zero clippy warnings. One Rust helper is intentionally excluded from direct
suite execution and exercised via its private hook control; one existing Windows
relative-link Node fixture is skipped. The run includes #259's final isolation
correction and the separate parent preview matrix. Guidance coverage45/355 and
`git diff --check` pass. Independent critic accepted the final source, relocation
controls and context unlink distinction before closure.

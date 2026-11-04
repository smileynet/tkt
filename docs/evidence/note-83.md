# Noninteractive ticket notes

PASS: final `mise run check` completed 451 Rust tests, 42 Node fixtures,
`cargo fmt --check` and warning-free all-target clippy. SKIP: one existing
Windows relative-link fixture. Seven public note tests exercised all lifecycle
states, prefix-byte preservation, caller attribution, file/stdin input, reserved
resolution input, no-write previews, missing/dirty/ignored-untracked targets,
same-worktree competing writers, linked worktrees, failed commits, failed pushes
and explicit retries. Fresh `--version` and `--help` smoke checks passed.

Independent critic accepted the final implementation after two review fixes:
verify tracking before writing an ignored file, and distinguish retained staged
content after commit failure from a durable local commit after push failure.
The final gate satisfied that acceptance condition. Companion JSON pins source
hashes. No real project ticket received a note during verification.

Initial new-command invocations failed with a Windows native stack overflow.
Instrumentation localized it to Clap command construction before argument
matching. Separating the existing large new and batch argument groups into
derived structs preserved their flags and reduced the generated frame size.
The existing repeated capability-construction regression and schema checks pass.
No stack-size override was introduced. A later targeted build encountered an
infrastructure error because the preceding gate held the Windows executable;
after that gate exited, rebuilding and the final gate passed.

See [input, coordination and recovery limits](../ticket-notes.md). Unix execution
and external-editor coordination remain unverified. This does not complete the
broader mutation-effect consolidation in parent 191. Disposable gate logs are
promoted here as compact results, then recycled; shared build outputs and
managed plugin caches remain for ongoing work, and unknown artifacts are kept.

# Guarded Git undo

Ticket #79 adds a concrete prepared inverse for receipted new/claim/close at HEAD. Version 1 binds parent, full before/after trees, sorted paths and blob/mode identities. Recognition checks actual Git objects, supported ticket transitions and the exact existing plan status rewrite. Legacy, arbitrary, malformed/duplicated/unknown receipts, unrelated metadata/body/plan edits, unsupported modes, merge/root/rebased and repeated history are refused. This is structural provenance, not author authentication.

PASS: final mise run check ran 443 Rust tests and 42 Node fixtures with clean formatting and zero clippy warnings. SKIP: one existing Windows relative-link fixture. Eight new public tests cover all three inverse forms, full-tree read-only preview and JSON output, forged-looking/altered receipts, root/merge/rebase, dirty/preexisting state, actual locked-file Git revert failure, commit-hook failure, unexpected hook tree, publication success/failure, unreachable remote with no-push, and upstream advance without automatic rebase. Original and remote commits remain intact in failure controls. Unix execution was not observed.

The first new-ticket test exposed an already-staged deletion that could not be staged again. The checked publisher now commits the verified prepared index, then verifies parent/tree before one push. It preserves conflict/sequencer state, uncommitted inverses and durable local commits rather than resetting. Normal publish callers retain existing behavior. A clippy empty-string-comparison failure was corrected without suppressing the lint; the full gate was rerun. Critic accepted final source after that behavior-equivalent correction.

Private-fixture operation timings were roughly 0.7–1.2seconds under concurrent test load; these are observations, not a controlled performance comparison or service-level promise. Source hashes are in the companion JSON; supported recovery limits are in docs/undo.md. The all-mutator effect/matrix acceptance in parent #191 remains pending. No real project history was reverted while validating this feature.

Source, maintained tests and compact receipts are promoted. Disposable task helpers/logs are recycled after promotion. Shared target and managed deployment outputs remain maintainer/harness-owned for ongoing frontier work; installation and pack refresh follow the integrated batch.

Recycled 4 disposable task files (64947 logical bytes) to the OS Recycle Bin after promoting results. Runtime private-index directories use scoped cleanup; no unknown artifact was removed.

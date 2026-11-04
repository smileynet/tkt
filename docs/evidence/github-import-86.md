# Bounded create-only GitHub import

PASS: final `mise run check` ran 470 Rust tests across 41 suites and 42 Node
fixtures, with clean `cargo fmt --check` and zero all-target clippy warnings.
SKIP: one existing Windows relative-link fixture. Independent critic accepted
the final source, guidance and test inventory conditional on this gate; the
condition is satisfied. Source hashes are in the companion JSON.

Nineteen public importer tests cover current-origin selection, missing `gh`,
explicit repository/label/validation inputs, null bodies, repeated pages,
unknown source preservation, repository renames, separate repositories sharing
issue numbers, transfer conflicts, bounds, failed reads and full no-write
previews. Ownership controls exercise unmanaged snapshots, duplicate identities,
missing tickets, label policy, staged data and modified receipts. Shared local
criteria satisfy required creation policy without inferring provider criteria;
no-new-ticket replay remains usable without supplying new criteria.

Recovery controls cover absent, one-sided, complete, staged and committed
pairs, selector mismatch, API outage, commit-hook failure, failed push, linked
worktrees, each rollback ref/index/file boundary and failed rebase recovery.
The `retry_ready` journal survives until the next attempt is atomically
prepared or a verified replay needs no work. Provider request counts prove
that retries and recovery use the recorded source rather than refetching it.

Private bare repositories and a small native Git probe force an actual
non-fast-forward race before advertisement, a separate mutation before rollback,
and an advance immediately before the branch compare-and-swap. Reallocation
passes; both unexpected advances remain intact with import evidence retained.
The existing push-error classifier was not broadened to treat hook/policy
failures as races. `new`, `batch` and import serialization, preview without a
lock, and no-push against an unreachable Git remote also pass.

A red public regression demonstrated that the old close writer changed a
provider checkbox inside a fence, left local acceptance unchecked and omitted
the local resolution. Shared fence-aware AC/resolution ranges now preserve
the source and isolate actual lifecycle sections across close, audit/fix and
undo. The green regression verifies local completion text and an eligible
undo preview. All nine existing undo tests pass after this repair.

The initial fake-gh failure was infrastructure: Node's preload option treated
Windows backslashes as escapes. A direct fixture probe exposed it; forward-slash
paths restored the positive control before rejection results were trusted.
Fixtures reference the installed Node executable through a symlink, rather than
copying it. The private native race probe is compiled inside a disposable fixture.
All child processes set `DO_NOT_TRACK=1` and use isolated configuration and
credentials. No actual issues were imported or written to GitHub.

Guidance coverage passed (45 units / 355 frozen lines). Native manifest checks
passed, Claude inventory listed the four workflows, and the workflow pack lint
passed. Disposable task logs are promoted here as compact evidence, then
recycled. Shared build outputs and managed deployment caches are retained for
the remaining work; unknown artifacts are preserved. Unix execution and a
real-service migration remain unobserved. Parent 191 is still unfinished.

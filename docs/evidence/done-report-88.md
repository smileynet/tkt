# Completed-work report evidence

PASS: fresh `mise run check` completed 482 Rust tests, 42 Node fixtures,
`cargo fmt --check` without diff and all-target clippy with zero warnings.
SKIP: one existing Windows relative-link fixture. The first gate stopped on
formatting; scoped formatting was corrected before the fresh passing run.

Eleven public report tests verify recorded committer timestamps against raw Git
transitions, exact rolling UTC membership, latest reclosure, cancellation and
reopening, context and identity behavior, root/untracked/uncommitted/duplicate/
renamed/changed-ID/unparseable history, merges and shallow histories. Fixtures
exercise per-ticket and total lookup budgets, oversized objects and tree listings,
and the aggregate historical payload cap. Missing promisor objects use a positive
fetch oracle before proving the report performs no lazy fetch. Whole-repository
byte snapshots verify no persistent effects. A unit test checks both exact
seven-day endpoints and adjacent excluded seconds.

The history reader uses one sequential batch-object child and bounded listing,
object, cache and aggregate payload reads; partial listings never attribute dates.
Unknown rows remain visible with reasons and null window membership. Returned,
dated, dated-in-window and unknown counts are separate. No stored completion
field was added. See [the reporting contract](../done-report.md) for conservative
identity rules and interpretation limits.

PASS: guidance coverage (45 units/355 frozen source lines), pack lint, and native
Antigravity strict validation. A skill-body size failure was corrected by routing
the workflow summary to its standalone sibling command reference. Independent
critic accepted the final source, guidance, counts and fresh passing gate before
closure. Source identities are in the companion JSON.

Unix execution, older Git versions and real-project attribution performance remain
unverified. These dates describe current Git history, not an independent work clock
or proof of acceptance quality. No real project tickets were changed by tests.
Shared build outputs and managed plugin caches are retained for continuing work;
unrelated artifacts are preserved. Owned diagnostic logs are recycled after this
compact receipt is committed.

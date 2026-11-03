# Workstream graph and planning gates — ticket 240

Owner request, 2026-11-02: arrange the ticket task graph by workstream and distinguish
unblocked work from unfinished ticket prerequisites, decisions and other gates.

## Representation

All 67 unfinished tickets belong to exactly one of six `stream:` workstreams:
Discovery; CLI workflows; Configuration and labels; Relations and lifecycle;
Agent guidance; Tooling, integrity and planning. Existing domain/triage tags remain.
Completed historical records are unchanged. All 240 tickets remain in the full list.

An explicit `gate:decision` or `gate:evidence` tag and a `## Planning gate` body
section own the reason in the ticket file. No gate is inferred from backlog or a
generic needs-info tag. Sixteen unresolved decisions and seven evidence gates are
recorded, including the two dashboard tasks whose rendered checks are unavailable.
The snapshot helper derives the display data. Status, priority, dependencies and
the CLI frontier are unchanged for every pre-existing ticket.

The graph's all-work arrangement uses six workstream lanes; dependency-depth
arrangement remains available. A narrowed stream discloses remaining work outside
the view, and cross-stream prerequisite edges stay present in the all-work view.
Selecting downstream work exposes upstream planning gates. Parent rollup's #197
decision appears upstream of #199 and #200; #87 and the other pending retirements
remain deferred and still depend on cancellation implementation #227.

## Checks

The canonical renderer, example, snapshot helper, checker and fixtures are maintained
in grey-matter's delivery-dashboard skill. The adopter uses that canonical renderer
without project-specific JavaScript changes. Meaningful regression fixtures cover
decision versus evidence gates, deferred and completed states, workstream partitioning,
cross-stream edges, malformed gates and rejecting ambiguous source gates before writes.

Initial contract run: assertion failure because the worked example still held the
old renderer (37 pass, 1 fail). After synchronizing the example, the replacement
`node --test scripts/delivery-dashboard.test.mjs` passed all 38 tests with no skips.

The adopter model audit checks all IDs/statuses/frontier against the real board,
all graph edges against actual blocked_by, complete and unique workstream membership,
every stream's elsewhere count, upstream decision paths, preservation of all 17
existing entries and snapshot repeatability. Expected exclusive graph buckets:

| Bucket | Tickets |
|---|---:|
| Open and unblocked | 26 |
| In progress | 2 |
| Open, blocked by ticket prerequisites | 7 |
| Open, prerequisites met, decision needed | 2 |
| Open, prerequisites met, evidence needed | 2 |
| Deferred backlog | 28 |

The adopter model audit passed: 240 tickets, 67 uniquely grouped unfinished tickets,
18 actual prerequisite edges, all 17 earlier entries preserved, repeatable snapshot
refresh and zero dashboard feedback asks. One initial check-script invocation had
an infrastructure error (Windows ESM path needed a file URL); the corrected run
produced those passing results. The initial error was not product evidence.

These buckets total 67. Gate badges can also appear on dependency-blocked,
in-progress or deferred work; the 16/7 gate totals are annotations, not additional
tickets. Resolved prerequisites remain distinguishable from unfinished ones.

## Limits and retention

Local browser use is disabled in Work settings, as observed during #228. No alternate
browser/Playwright route was attempted. Browser-gate execution, laptop/phone layout,
visible edge routing and interactive behavior remain unverified; source/model checks
do not establish rendered acceptance. No accepted review receipt or fabricated
captures exist. This limitation is recorded here and under dashboard Details, not
turned into an ask for feedback. The user provides feedback when needed.

No Rust code or CLI contract changed. Rust build/test/clippy were skipped. Only
small temporary editing/check scripts and their JSON output were created; relevant
results are promoted here before sending those temporary files to OS trash.

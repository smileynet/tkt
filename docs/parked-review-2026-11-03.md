# Parked-work review — 2026-11-03

The owner requested review and execution of actionable parked work. All 23 tickets were read against the current board and implementation. Historical blanket-deferral notes are superseded by this record and each active ticket's reactivation note. Opening a ticket authorizes its stated bounded work, not automatic hooks, hosted CI, a release, or invented completion.

| Ticket | Disposition | Executable scope or remaining gate |
|---:|---|---|
| 69 | Open | Inventory delivered integrations; define focused follow-up from observed gaps. |
| 79 | Open | Tighten undo to a guarded Git revert of a recognized HEAD mutation; never expose reset-based internal recovery as public undo. Reject dirty/unrecognized history. |
| 83 | Open | Atomic explicit noninteractive note append; preserve lifecycle and existing prose. |
| 86 | Open, blocked by 125 | One-way GitHub issue import, the provider explicitly requested in this ticket. Contract first: stable repository-qualified identity, mapping and repeat-import behavior; no external writes. |
| 88 | Open | Report verified done transitions from Git history; unknown dates stay unknown, cancelled remains distinct. |
| 89 | Open | Explicitly opted-in, bounded advisory lifecycle hooks; failures cannot delay publishing or change lifecycle truth. |
| 90 | Open | First HTTP(S) URL helper with portable launch and dry-run; test launch arguments through isolated substitutes. Actual OS browser launch remains unverified under current browser restrictions. |
| 93 | Open, design only | Narrow optional extension schema, supported scalar representations, compatibility and migration fixtures. Historical nested YAML/dynamic CLI ideas are not approved implementation. |
| 99 | Open | Strict user-level interval setting, preserved default cadence and consent/environment overrides; isolated cache/clock/network tests. |
| 125 | Open, design only | GitHub issue adapter contract grounded in the explicit import request in 86, not merely this repository's hosting. Identity, state, relations, unknown fields and idempotence fixtures. |
| 147 | Parked, evidence | Original cross-session command/environment sequence is missing. Isolated consent precedence investigation may narrow the report; do not change consent from an unreproduced hypothesis. |
| 148 | Open | Examples distinguish invocation identity, eligibility, assignee, tags and machine capabilities. |
| 157 | Open | Optional advisory project hook installer, foreign-hook preservation, ownership, portability and recovery. No hook enabled here. |
| 158 | Parked, evidence; depends on 157 | Pin and verify current Claude/Codex write-tool hook contracts, trust and supported versions before implementing. Existing private SessionStart support is not evidence of write-hook support. Current official documentation may permit this; do not treat historical absence as proof of impossibility. |
| 168 | Open, measurement only | Sample repository/history evidence without enabling telemetry; assess demand, no new runtime nudge. |
| 173 | Open, blocked by 175 | Bounded advisory frontier readiness reuses source-reference predicate; no ordering or eligibility change. |
| 175 | Open | Shared source-reference union and explicit severity/config contract; default advisory. |
| 184 | Parked, explicit deferral | Owner chose to keep narrative parsing deferred; independent exact-status defect already completed in 248. |
| 186 | Open | Retire unsupported Python parity instructions/runner; preserve useful cases in maintained Rust coverage. |
| 187 | Open | Deterministic derived projection, atomic/dry-run semantics, preserve human narrative. Not narrative parsing. |
| 191 | Open, audit first | Inventory mutation effects and dry-run paths before choosing a shared executor; preserve atomicity/push gating, avoid a shallow wrapper-only refactor. |
| 202 | Open, design only | Versioned embedded assets and binary-only export/install contract with ownership, rollback, duplicate and conflict fixtures. |
| 249 | Open | Optional local pre-push convenience, verified pushed-tip scope. It does not replace hosted enforcement or activate hooks here. |

Execution queue: 202 (high priority design), 99 (medium configuration), 186 (retirement), 125 (adapter design), 175 (shared validation predicate), then other independent ready work. Dependencies encode required outputs, not preferred sequencing. The broad 191 audit comes after concrete feature work to avoid premature abstraction.

No ticket was closed by this review. Every closure requires fresh scoped checks and independent critic acceptance. UI tickets 228, 240 and 243 retain their existing rendered-acceptance limitation separately from this parked cohort.

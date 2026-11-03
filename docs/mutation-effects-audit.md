# Mutation effects audit (in progress)

Ticket191 remains implementation work, not completed by this audit or child250.
The source survey at d197af2 found these distinct effect families:

| Family | Current owner | Evidence and consolidation constraint |
|---|---|---|
| Existing ticket lifecycle | MutationContext | Resolves fetch/ownership, atomic ticket writes, scoped commit and push; preserve this lifecycle. |
| Allocation/retry | GitTransaction plus new/batch | Fetch/ID allocation and race retry are distinct; do not merge with MutationContext. |
| Local configuration/context/init | Commands plus config/context modules | Atomic config replacement exists, but directory creation and context/init writes remain scattered. |
| Owned local publication/removal | hooks and plan_projection | New-file no-clobber, exact ownership, quarantine/recovery and external-editor limits must survive any common plan. |
| Migration/fix/renumber | Materialized domain plans | Migration still directly writes ticket output; audit its backup/remove/write order before routing to common effects. |
| Diagnostics/consent/update cache | telemetry/update_check and CLI entry | Persistent debug sink currently leaks under dry-run; independently reproduced and captured in child250. Never change user consent during tests. |

Observed250reproduction: private XDG state/config roots, DO_NOT_TRACK=1,
TKT_DEBUG=log and --dry-run config --set debug=true exit0 and create debug.log.
CLI currently emits the first event before storing the preview flag, and the
file sink has no preview guard. The requested config itself stays absent.

The shared plan must preserve semantic operations rather than turn domain
transactions into arbitrary shell callbacks: atomic replacement of a prepared
snapshot, no-clobber creation, exact-owned removal with recovery, directory/
backup operations and an explicit publication disposition. Preview/application
must be distinguished before any persistent diagnostics, filesystem or Git step.
Existing transaction owners retain their allocation and existing-ticket roles.

Child250is complete after public red/green controls,fullgate andcriticacceptance.
Next steps: inventory all
mutating paths including newly added hooks/projection; define a small concrete
plan and its result types; migrate families with existing error/output policies;
run a cross-command no-write matrix. No generic trait/plugin framework, opaque
callback-only wrapper or claim of transactional external-editor locking is approved
by this audit. Remaining all-mutator acceptance criteria are still pending.

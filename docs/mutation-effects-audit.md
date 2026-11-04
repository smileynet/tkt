# Mutation effects audit (in progress)

Ticket #191 remains implementation work, not completed by this audit or its repaired children #250 and #251.
The source survey at d197af2 found these distinct effect families:

| Family | Current owner | Evidence and consolidation constraint |
|---|---|---|
| Existing ticket lifecycle | MutationContext | Resolves fetch/ownership, atomic ticket writes, scoped commit and push; preserve this lifecycle. |
| Allocation/retry | GitTransaction plus new/batch | Fetch/ID allocation and race retry are distinct; do not merge with MutationContext. |
| Local configuration/context/init | Commands plus config/context modules | Atomic config replacement exists, but directory creation and context/init writes remain scattered. |
| Owned local publication/removal | hooks and plan_projection | New-file no-clobber, exact ownership, quarantine/recovery and external-editor limits must survive any common plan. |
| Migration/fix/renumber | Materialized domain plans | The original audit found unsafe publication order. Child #251 now prepares snapshots and safely publishes backups/output; broader command-plan routing remains incomplete. |
| Diagnostics/consent/update cache | telemetry/update_check and CLI entry | The original dry-run debug leak was reproduced and repaired in child #250. Consent policy was not changed. Never change user consent during tests. |

Original #250 reproduction: private XDG state/config roots, DO_NOT_TRACK=1,
TKT_DEBUG=log and --dry-run config --set debug=true exited 0 and created debug.log.
Before the repair, the CLI emitted its first event before storing the preview flag, and the
file sink had no preview guard. The requested config itself stays absent.

The shared plan must preserve semantic operations rather than turn domain
transactions into arbitrary shell callbacks: atomic replacement of a prepared
snapshot, no-clobber creation, exact-owned removal with recovery, directory/
backup operations and an explicit publication disposition. Preview/application
must be distinguished before any persistent diagnostics, filesystem or Git step.
Existing transaction owners retain their allocation and existing-ticket roles.

Child #250 is complete after public red/green controls, the full gate and critic acceptance.
Next steps: inventory all
mutating paths including newly added hooks/projection; define a small concrete
plan and its result types; migrate families with existing error/output policies;
run a cross-command no-write matrix. No generic trait/plugin framework, opaque
callback-only wrapper or claim of transactional external-editor locking is approved
by this audit. Remaining all-mutator acceptance criteria are still pending.

Child 251 is complete after red/green failure controls, the full gate and critic review. Prepared migration and shared no-overwrite creation now cover migration, hooks and projection. The all-mutator executor, remaining local effects and full matrix remain pending. Guidance follow-up #252 corrected the unsupported map-file reference after command checks and critic acceptance.

Child #253 routes configuration, context and init filesystem persistence through concrete prepared effects. Snapshots guard ordinary stale merges; new publication uses the no-overwrite primitive. Replacement still has an external check/rename race, ancestor links are filesystem-resolved, and multi-file init is not transactional. Parent #191 remains in progress for other families, Git effects and the all-mutator matrix.

Completed #79 adds a prepared Git inverse with receipt-bound recognition and an optional exact parent/tree expectation in the existing publisher. Exact undo publishes once and never rebases automatically after rejection. Include this new mutator in #191’s remaining all-command no-write matrix; ordinary publisher/allocation ownership remains distinct.

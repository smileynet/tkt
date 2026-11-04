# Mutation effects audit

Current implementation and verification are recorded in the
[261 receipt](evidence/git-effects-261.md) and the final section below.

## Historical survey and intermediate delivery states

The following survey and child updates record their state when written. Their
then-pending work is superseded by the 2026-11-04 execution section below.
Ticket #191 was not completed by this initial audit or children #250 and #251.
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

Ticket #86 extends the allocation boundary with a shared new/batch/import marker, preview without fetch, a one-shot immutable-OID push seam and journaled ticket/snapshot ownership. Its importer retry uses a recorded branch CAS and scoped index/file cleanup instead of the legacy generic reset. A reproduced fenced-source close regression now uses shared fence-aware AC/resolution ranges across writer, audit/fix and undo. This does not complete other mutation families or the all-command no-write matrix in #191.

Ticket #89 adds explicitly configured, locally trusted advisory process effects after new/claim/close publication attempts. Preview/failed commits/rolled-back allocations do not dispatch. The publisher carries an explicit event; other mutation families pass none. Hooks can make arbitrary trusted external effects and have no inverse or durable delivery journal. Include process invocation absence in #191’s all-mutator preview matrix; lifecycle hooks do not replace or complete the shared effect plan.

Ticket #158 adds explicit native-nudge config/helper/ownership effects through FilePlan. Its install/remove modes preflight selected files before snapshot-guarded writes; check and global preview do not mutate native files or Git refs. Include all modes in #191’s remaining no-write matrix. Generation is per-file atomic, not a multi-file transaction, and native trust/user settings remain outside its effects.

## Final execution phases — 2026-11-04

Child #259 repaired observed telemetry preview writes and the unsafe Windows test
isolation assumption; [the receipt](evidence/telemetry-preview-259.md) records the
unknown prior user state. Telemetry mutation tests now use explicit private paths.

Child #260 centralizes command-owned text publication in `FilePlan`:
`ReplaceSnapshot`, `CreateNoClobber`, `RelocateSnapshot`, `RemoveOwned`, explicit
context `UnlinkEntry`, directory creation and selected telemetry-directory removal.
Preparation checks snapshots/ownership before Preview and before starting a phase;
Apply checks again at each publication boundary. All command call sites now reach
atomic text primitives only through this dispatcher. Parsed tickets retain their
original raw snapshot so a later owner edit conflicts rather than being overwritten.
Renumber/migration relocation publishes without clobbering before removing the
matching source; a later failure can retain both versions. Owned removal stages
and inspects bytes, restores a foreign arrival without clobbering, or preserves
its staging path for manual recovery. Context clear intentionally unlinks its
named entry, including a symlink, without following it.

These are phase-local checks, not a cross-file transaction or external-editor lock.
Preparation failures now stop a phase before earlier writes; later I/O failures
can still leave earlier publications. Ancestor links remain filesystem-resolved.

Child #261 adds concrete typed dispatcher phases for Git publication/fetch/recovery,
allocation commit/retry, undo, collision rebase, lifecycle notifications, operation
locks, cadence and telemetry diagnostics. Existing transaction algorithms retain
ownership of their scoped Git/index/CAS/recovery behavior; no callback-based plan
or combined transaction abstraction was introduced. Git support summaries include
temporary undo indexes and hook directories, configured commit hooks and owned
lock release. Cadence and update-cache replacement use the same FilePlan boundary.

Real command preview branches now reach prepared phases before returning their
existing output. JSON mode adds stderr `preview_effects` records without changing
stdout. Cached Git reads disable optional locks and lazy fetching; remote results
and generated support paths remain explicitly unresolved. Phase-local plans are
materialized again by existing retry owners rather than replayed after refresh.

The [matrix](mutation-preview-coverage.md) covers capability-declared mutation
commands, private telemetry controls, cached and stale remote refs, partial clones,
import/pending recovery and trusted lifecycle process markers. The
[execution receipt](evidence/git-effects-261.md) records verification and limits.

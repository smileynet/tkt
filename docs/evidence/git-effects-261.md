# Git, process and support effects — 2026-11-04

Ticket 261 completes the remaining dispatcher slice of parent 191. Concrete
phase payloads delegate publication, allocation, import recovery, collision
rebase and undo to their existing transaction owners. No opaque callback or
combined transaction framework was introduced. Files still use prepared
snapshot/ownership checks and atomic publication. Existing scoped staging,
push.enabled gating, CAS checks and recovery journals remain in their owners.

Lifecycle notifications, operation-lock acquisition/release, cadence, telemetry
diagnostics and consent mutations are mode-gated phases. Commit summaries include
temporary undo index/hook-directory cleanup and conditional configured hooks.
Cadence and updater cache replacement use FilePlan. Real preview command branches
reach prepared phases before returning their established output. JSON previews
add stderr JSON Lines under preview_effects without changing command stdout.
Summaries describe phase-local intent, not replayable transactions: retry owners
re-materialize work, and remote/generated support outcomes remain unresolved.

The capability-linked matrix exercises 29 local routes with complete operation
sets and full fixture snapshots, plus cached remote collisions/stale allocation,
import/pending recovery and a real partial clone. Missing partial-clone objects
remain missing during preview; an ordinary subsequent Git read fetches them as
the positive control. Preview Git reads disable optional locks and lazy fetching.
Trusted private lifecycle hooks have a real successful-claim marker control.
The ignored preview_hook_probe is explicitly invoked as a subprocess by that
control, not an unverified behavior. Private unit controls assert telemetry
operation/path summaries and prove lazy platform resolvers are not called.
Import recovery unit controls assert exact affected ticket/snapshot paths.

## Verification history and limits

Intermediate checks caught a clippy PathBuf reference warning (changed to Path),
an integration assertion reporting Windows Access denied while writing a ticket,
and incomplete preview expectations. The strengthened remote/import tests also
caught incorrect test operation names and an uncommitted private fixture config;
these were assertion failures, not passing checks. Their replacement results must
come from the final gate below.

The critic accepted the final source and preview contract after requesting exact
operation-set assertions, support effects and import recovery path coverage.
Final results:

- `mise run check`: pass, 523 Rust tests across 46 suites, zero failures;
  one explicitly invoked subprocess-helper test marked ignored. Node fixtures:
  42 pass, zero failures, one existing Windows relative-link skip. Formatting
  produced no diff and clippy completed with warnings denied and none reported.
- `node tools/check-guidance-coverage.mjs --final`: pass, 45 units cover all
  355 frozen lines. `git diff --check`: pass. Scope audit found only dispatcher,
  preview tests and their guidance/evidence changes.
- `node tools/deploy.mjs check` and install dry-run: pass. Current
  `agy plugin validate dist/agy-plugin --strict`: pass after shortening the new
  work-skill line to avoid the 24,000-byte body limit; the first strict attempt
  was a warning failure at 24,029 bytes.

The critic independently accepted the source, final docs and parent criteria:
atomic writes, push gating, distinct transaction ownership, complete reviewed
preview routing and unchanged stdout contracts. Acceptance depended on the fresh
gate above; the earlier failed attempts were not used as completion evidence.

## Deployment and reconciliation

Source commit `e755765` was committed and pushed. `cargo install --path . --locked`
passed (release build 1m15s); `bash tools/check-binary-fresh.sh` reported installed
and HEAD `e755765` before closure-only metadata commits. `node tools/deploy.mjs`
passed: four workflow skills refreshed, Claude checkout ownership verified,
Codex plugin refreshed, Kiro/OpenCode copies updated and Antigravity plugin
strictly validated and installed. KiroCrew was built only because its CLI is
absent. No release, hosted CI, trust activation or project hook installation.

Tickets 261 and parent 191 were closed explicitly through `tkt close`, with
criterion-specific evidence after critic acceptance; closure commits were pushed.
Parent body acceptance is 5/5. `tkt sync-plan` passes with no findings. `tkt validate`
passes its error checks but retains eight pre-existing missing-resolution warnings
on historical tickets 50,91,154,155,156,174,178,206; those were not rewritten.
The refreshed dashboard checker reports 100 entries, 261 tickets, zero open asks,
zero errors and zero warnings. The LAN endpoint returned HTTP200 and the current
entry bytes. Rendering remains unverified; existing UI tickets stay open.

No ready open ticket remains. Four WIP tickets retain their existing gates:
90 (policy rejection), 228/240/243 (unavailable rendered acceptance).
Backlog 147 requires its exact reproduction; 184 remains explicitly deferred.

No hosted CI, real GitHub provider writes, rendered dashboard checks or native
hook activation were exercised. Hooks remain trusted advisory external effects,
not sandboxed or exactly-once delivery. File phases remain per-file atomic,
not multi-file transactions. External editors and ancestor symlinks remain outside
that guarantee. The earlier Windows telemetry isolation incident and unknown
prior user state are preserved in [the 259 receipt](telemetry-preview-259.md).

## Artifact disposition

The compact result belongs in this tracked receipt. Ticket-owned transformation
scripts and intermediate logs are disposable after final review; the final raw
gate log is retained only through the explicit parent 191 closure review, then
recycled. Shared target output measured 6,644,872,628 bytes during final validation;
retain it for installation and subsequent maintainer builds until a deliberate
build-cache cleanup. Generated dist packs measured 799,979 bytes and are retained
for deployment/inspection until regenerated. Free space remained above 150 GB on
C: and 200 GB on D: before the final gate.

Pre-existing artifacts were preserved: node_modules 3,934,146 bytes; scratch
research 148,002, review 80,544, review-input 85,113, review-output 43,080,
subagent-input 2,553 and subagent-raw 130,827 bytes; proposal 5,129 bytes,
dashboard server state 298 bytes and .gitkeep 51 bytes. Their lifecycle is not
established by this ticket. Existing user telemetry bytes were not guessed or
restored. No machine-wide cache cleanup was performed.

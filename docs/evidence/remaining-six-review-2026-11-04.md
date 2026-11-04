# Remaining six tickets — 2026-11-04

Reviewed the current ticket bodies, existing evidence, relevant source and the
owner's recorded deferral. No product implementation or lifecycle status changed.

| Ticket | Fresh finding | Disposition / remaining gate |
|---|---|---|
| 147 | Private fresh-process consent-reader probe did not reset saved consent. Environment/path controls reproduce effective disabled state without changing enabled bytes. | Keep needs-info/backlog: original platform/path/enable/restart sequence remains unknown. No speculative persistence fix. |
| 184 | ADR0002 and the owner's explicit deferral still apply. Independent status-cell repair was delivered in #248; current table parser recognizes lifecycle states and all eight focused sync-plan integration tests pass. | Keep deferred. No new independent PlanState defect verified, no narrative parser activated. |
| 228 | Existing migration/data checks remain recorded; current dashboard checker has zero errors/warnings. | Actual Overview/History/Plan laptop/phone inspection still required. |
| 240 | The complete ticket snapshot remains available; graph model evidence does not establish pixels or interaction. | Actual workstream/filter/dependency/terminal-state controls need rendered inspection. |
| 243 | All 17 decisions are already answered; deterministic renderer evidence is historical, not native clipboard evidence. | Inspect real answered state and a controlled pending-decision fixture using the same renderer; recommended choices, notes, export and denied-clipboard fallback. |
| 90 | The earlier implementation-writing action was rejected as blocked by policy. No new authorization/capability evidence removes that rejection. | Preserve the gate; do not retry through another route or launch a browser. |

## What rendering evidence is needed

Suggested concrete viewports are laptop 1366×900 and phone 390×844 CSS pixels;
record the actual viewport and browser/device scale used. These are proposed
test identities, not observations. At both sizes:

1. Inspect Overview, searchable History and all-ticket Plan for readable text,
   reachable controls, wrapping, clipping and intended graph scrolling. Counts
   and statuses must match the current board; no active tag context may hide work.
2. Exercise stream/scope filters, select a ticket and follow a real dependency
   chain with completed context enabled. Verify cross-stream edges and cancelled
   versus completed states. The current board's no-ready-work state must remain
   clear, with evidence gates and deferred backlog visibly distinct.
3. Inspect Decisions' real answered history. In a controlled fixture with pending
   decisions and the identical renderer, verify recommended radio defaults,
   alternative selection, optional text and retention when switching tabs.
   Activate copy and compare exported text; test denied clipboard access and
   select/copy fallback. Record native clipboard behavior separately from DOM tests.
4. Capture useful screenshots of the inspected views/states and record interaction
   outcomes. Bind source/config/data/assets and captures to the delivery-dashboard
   review-receipt manifest. The critic must inspect them, accept that snapshot ID,
   and the receipt must still match immediately before closure. Changed rendering
   inputs require affected rechecks/captures/review, not stale acceptance.

Fresh `cua.getState()` returned no browser surfaces and:
`Browsers: Error: Local browser use is disabled in Work settings`.
Outcome: blocked; no browser, native-browser fallback, headless automation,
captures or accepted UI receipt were produced. HTTP/checker results cannot clear
these explicitly required UI acceptance criteria. This creates no owner feedback
task; the owner provides feedback during normal use.

## Consent reproduction attempt

Run from the repository root:

```text
node docs/evidence/consent-session-probe-147.mjs
cargo test private_telemetry --bin tkt
cargo test --test integration sync_plan
```

The probe compiles the exact current `check_consent` and `read_consent_file`
function bodies into a temporary reader, substituting only a path resolver bounded
to its private temporary roots. A source SHA guard requires re-audit if the source
changes. It cannot resolve Windows Known Folders or invoke the full CLI; it does
not exercise main startup, the real platform resolver or a historical reporter
sequence. The enabled file is explicitly seeded, not written by a CLI enable
command. Actual private writer/preview controls are the separate two Rust tests.

Observed: 15 successful controls, including five independent fresh-process reads
returning `Enabled ConfigFile`, all preserving the private enabled bytes. DNT=1,
tool opt-out and present CI (even empty) report disabled without rewriting. Tool
opt-in precedes CI, while DNT precedes tool opt-in. An alternate absent config root
reports `Disabled Default`, and returning to the original root reports enabled.
An injected private persistent reset is rejected by the enabled-state oracle.
Full compact results and source SHA are in
[the result record](consent-session-results-147.json).

This reproduces an effective-state change, not the ticket's unproven persistent
reset. No root cause is assigned to the original report. Next decisive evidence
is an actual failing sequence with platform/version, resolved consent path,
before/after file bytes and times, effective-state reason and the presence of
DNT/tool/CI overrides in both sessions. No real user consent was enabled, disabled,
cleared, restored or inferred by this probe. The separate earlier #259 incident
remains historical and does not prove the original #147 cause.

Verification: the promoted probe reran and its JSON matched the tracked result;
private telemetry tests passed 2/2; focused sync-plan integration passed 8/8.
No assertion failures, skips or timeouts in these checks. No Rust production
source changed; a full rebuild/redeployment is not needed for this evidence pass.

Artifacts: promoted the bounded probe, compact results and this report. Temporary
reader executables/fixtures were removed by their owning probe; owned scratch
scripts/results are recycled after review. Existing build caches, dependencies,
dashboard server and unknown-owner scratch remain preserved under prior scopes.

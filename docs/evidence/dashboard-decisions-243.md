# Dashboard decisions tab — 243

The owner's 2026-11-03 request is implemented in `docs/status.html#decisions`:
each decision has multiple-choice radio options, the recommendation selected by
default, and a labelled free-text field. The bottom button prepares one message
with every selected option and any entered notes for pasting into chat.

## Decision coverage and behavior

The data includes all 16 recorded decision gates: three current blockers and
13 deliberately deferred design decisions. A seventeenth card covers approval
of the completed guidance proposal, 226. Current blockers appear first.
Deferred work is labelled and remains backlog; choosing a draft does not mutate
ticket status. Recommendations are proposals, not recorded owner approval.

Radio choices and notes remain in the live page when switching tabs. Reloading
starts from the source recommendations. The clipboard handler exposes a read-only
message for manual copying if access is absent, denied or throws. Answered
decisions can be retained in collapsed history after their outcome is recorded
from chat. The overview links to pending cards.

## Canonical source and preservation

The shared canonical renderer/contract was published during this work as
grey-matter `7fbec9b` / `9d909b0`, followed by the layout revision `a812670`.
The adopter is pinned to published `a812670ce48836ba2e0a7c6431f89deeba2cbf98`,
with project decisions in `gm-config`. Existing config is preserved and all
52 previous delivery entries remain unchanged; ticket data is refreshed only
with the snapshot helper. No unrelated shared-source edits were reverted,
stashed or published by this task.

## Verification

- PASS: `node --test tools/dashboard-decisions.test.mjs` — seven tests, zero failures.
  These execute the actual page renderer and event handlers in a small DOM model:
  arrow-key navigation includes all four tabs, every decision gate has choices, one recommendation is selected per group,
  labels are linked, notes and alternatives survive navigation, overview links
  route to cards, answered history stays available, and clipboard success plus
  absent/rejected/throwing access produce the expected message/fallback.
- PASS: `node --test D:/code/grey-matter/scripts/delivery-dashboard.test.mjs` —
  42 tests, zero failures/skips against the final canonical source.
- PASS: the canonical checker accepts the adopter with zero errors/warnings.
- PASS: migration assertions preserve prior config and all 52 historical entries,
  and compare the renderer to the canonical template outside its three data blocks
  and page title.
- ASSERTION FAILURE, corrected: the initial decision records lacked required
  status/recommendation-reason fields, and a renderer comparison detected the
  concurrent canonical update. The data now follows the shared contract and the
  renderer is pinned to a published commit; final reruns pass.
- SKIP — policy: actual laptop/phone rendering, native browser radio behavior,
  accessibility and OS clipboard execution were not inspected. Local browser use
  remains disabled; DOM simulation is not a rendering or native-clipboard pass.
  A fresh `cua.getState()` inventory returned: `Browsers: Error: Local browser use
  is disabled in Work settings`. No browser navigation or native-browser bypass
  was attempted.

This is HTML/data plus a Node regression fixture; Rust product code is unchanged,
so no Rust rebuild is required. Ticket 243 remains in progress with its rendered
acceptance unchecked. No request for owner dashboard review is made.

## Artifacts

The dashboard, regression fixture and this report are durable tracked outputs.
Owned scratch scripts, ticket input and test log are recycled after publication.
No captures, media or dependencies were downloaded. Existing mixed-owner build
cache and unknown-owner scratch retain the scopes and sizes in
[the maintenance report](frontier-maintenance-2026-11-03.md); none were deleted.

The canonical pack's `deploy.mjs check` and deployment passed, refreshing the
installed delivery-dashboard skill. Its reproducible local Antigravity output
(55,989,313 bytes) is retained for the active deployment until the next refresh
replaces it. The existing generic-logo note is informational.

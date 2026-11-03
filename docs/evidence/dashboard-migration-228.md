# Dashboard migration — ticket 228

Implementation: 2026-11-02. Source: docs/status.html, migrated from the installed
delivery-dashboard/assets/dashboard-template.html without renderer modifications.
AGENTS.md now distinguishes hand-edited configuration/entries from the helper-owned
gm-tickets snapshot. The old 16 delivery entries and their linked assets are retained.

## Checks and claim boundaries

- `snapshot-tickets.mjs docs/status.html`: 239 tickets, 30 frontier IDs; no tag context.
- `check-dashboard.mjs docs/status.html`: initial runs caught a missing new evidence
  link and required ask id/status; these were corrected before the final passing check.
- Metadata/source checks verify the current canonical renderer, full corpus/status
  agreement, entry preservation, snapshot repeatability and the actual dependency
  chain #193 → #198 → #199. These establish source/data properties, not rendered UI.
- The graph defaults to all remaining work, including backlog. Optional CLI, config,
  guidance and quality scopes do not replace the full ticket list.
- Intended execution ordering remains [the delivery plan](../plan.md); dependency
  edges and the mechanical frontier were not altered to force that ordering.

## Visual acceptance — pending

Observer: tool. Surface: Codex local browser. Scenario: open the local dashboard
using `cua.createBrowserTab`. Outcome: **blocked** — “Local browser use is disabled
in Work settings.” No alternate browser or server route was attempted.

Laptop and phone rendering, all three tab interactions, visible dependency paths,
responsive clipping and scope-switching behavior remain unverified. No captures
exist and no accepted review receipt was fabricated. Ticket 228 stays in progress;
the dashboard carries an open visual-review ask. A human functionality report can
be recorded separately but is not a tool-observed rendering check.

This deliverable is a local dashboard source update, not a release publication.
No Rust source changed and no Rust build/test/clippy run is claimed. No scratch
captures, copied assets or new runtime dependencies were created.

## Observed static results

- Canonical renderer identity after removing only the three data blocks: **pass**.
- Original delivery-entry JSON preservation: **pass, all 16 unchanged**.
- JavaScript syntax compilation with Node vm.Script: **pass**.
- Two snapshot refreshes using the same clock: **pass, byte-identical**; all bytes
  outside gm-tickets remained unchanged.
- Snapshot versus actual query: **pass, all 239 IDs and statuses match**, 30 frontier
  IDs, ticket228 in progress; edges 193→198→199 present.
- Default all-work scope and four optional streams: **pass, source/data only**.
- Snapshot and renderer are offline and have no CDN script/style dependencies.

These checks do not satisfy the pending rendered phone/laptop acceptance criterion.

Reviewed source identities (static checks; no rendered acceptance):

- docs/status.html SHA-256: `e8ce3b5b29da383c3a24c606ee6c5860bb061163b0a29e7ecb3f6d6b91d4cade`
- Canonical template SHA-256: `cbef711bad6441e7ef3f83d0c64b4194b38cacf7ea2cae6e04708c850f7be5b1`

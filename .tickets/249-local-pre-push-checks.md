---
id: "249"
title: "Add optional local pre-push verification"
status: open
blocked_by: []
priority: low
validation_criteria:
  - "An opt-in Git pre-push hook runs mise run check (or an equivalent toolchain-pinned gate) on the commits being pushed"
  - "Contributor docs explain opt-in installation and its local bypass/availability limits; no hosted-CI equivalence is claimed"
tags: ["quality", "kind:enhancement", "triage:ready-for-agent", "stream:quality"]
---

# Add optional local pre-push verification

## What to build

`.github/workflows/ci.yml` (push/pull_request CI, from #185) was removed in commit
4b78bf4; the local `mise run check` gate is now the only change gate. Add an
opt-in pre-push hook (for example `.githooks/pre-push`, enabled with
`git config core.hooksPath .githooks`), mirroring grey-matter's
`scripts/gate.mjs pre-push` pattern. Parked: not needed yet.

## Acceptance criteria

- [ ] With the hook enabled, `git push` runs the gate and refuses the push on failure.
- [ ] `docs/agents/enforcement-model.md` and contributor docs describe enabling the hook.


## Reactivation — 2026-11-03

The owner requested review of all parked work and execution of actionable tickets. This supersedes earlier blanket deferral. See [the parked-work review](../docs/parked-review-2026-11-03.md) for current scope and sequencing. Independent critic review is required before closure.

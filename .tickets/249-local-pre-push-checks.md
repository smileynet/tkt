---
id: "249"
title: "Add local pre-push checks to replace removed CI workflow"
status: open
blocked_by: []
priority: low
validation_criteria:
  - "An opt-in Git pre-push hook runs mise run check (or an equivalent toolchain-pinned gate) on the commits being pushed"
  - "Contributor docs explain how to enable the hook and that it replaces the removed push/PR CI workflow"
tags: ["quality", "kind:enhancement", "triage:ready-for-agent", "stream:quality"]
---

# Add local pre-push checks to replace removed CI workflow

## What to build

`.github/workflows/ci.yml` (push/pull_request CI, from #185) was removed in commit
4b78bf4; the local `mise run check` gate is now the only change gate. Add an
opt-in pre-push hook (for example `.githooks/pre-push`, enabled with
`git config core.hooksPath .githooks`), mirroring grey-matter's
`scripts/gate.mjs pre-push` pattern. Parked: not needed yet.

## Acceptance criteria

- [ ] With the hook enabled, `git push` runs the gate and refuses the push on failure.
- [ ] `docs/agents/enforcement-model.md` and contributor docs describe enabling the hook.

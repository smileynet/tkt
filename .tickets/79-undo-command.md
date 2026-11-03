---
id: "79"
title: "tkt undo: revert last mutation"
status: open
blocked_by: []
priority: low
validation_criteria:
  - "`tkt undo` reverts last tkt-generated commit"
tags: ["cli", "kind:enhancement", "triage:ready-for-agent", "stream:workflow"]
---

# tkt undo: revert last mutation

## Context

Every git-based competitor (dstask, git-bug) offers undo. It's a trivial `git revert HEAD` wrapper but removes fear of "what if I close the wrong ticket?" — especially valuable for agents that might make mistakes.

## What to build

`tkt undo` reverts the last tkt-generated commit. Safety: only reverts if HEAD was authored by tkt (check commit message prefix pattern like "tkt:" or the structured format tkt uses).

```bash
tkt undo
# → ✓ reverted: "close 03 deploy-pipeline"
```

Edge cases:
- If HEAD wasn't a tkt commit, error with "nothing to undo (last commit wasn't a tkt operation)"
- If push.enabled, push the revert commit
- Multiple undos should work (each undo is itself a commit that could be undone)

## Acceptance criteria

- [ ] `tkt undo` reverts last tkt-generated commit
- [ ] Refuses to undo non-tkt commits (safe boundary)
- [ ] Pushes the revert if push.enabled
- [ ] Shows what was reverted in the confirmation message
- [ ] Integration test covering undo of close, claim, and new


## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:ready-for-agent`; stream `cli`.

Deferred feature; src/cli.rs has no undo command. Match actual chore(tickets) commit messages, refuse unrelated HEAD, and preserve push gating.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.


## Reactivation — 2026-11-03

The owner requested review of all parked work and execution of actionable tickets. This supersedes earlier blanket deferral. See [the parked-work review](../docs/parked-review-2026-11-03.md) for current scope and sequencing. Independent critic review is required before closure.

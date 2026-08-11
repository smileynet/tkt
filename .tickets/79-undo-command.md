---
id: "79"
title: "tkt undo: revert last mutation"
status: open
blocked_by: []
priority: high
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

# tkt undo: revert last mutation

## What to build

TBD

## Acceptance criteria

- [ ] TBD

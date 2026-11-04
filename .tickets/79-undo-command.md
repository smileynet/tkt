---
id: "79"
title: "tkt undo: revert last mutation"
status: done
blocked_by: []
priority: low
validation_criteria: 
  - "Versioned receipts bind recognized new claim and close transitions to exact parent tree path blob and mode; arbitrary or unsupported history is refused"
  - "Git revert preserves original history and exact prior ticket/plan state; dirty state conflicts hooks and push failures retain recoverable work without reset or automatic rebase"
  - "Public integration and final local gate pass with standalone guidance and independent critic acceptance"
tags: ["cli", "kind:enhancement", "triage:ready-for-agent", "stream:workflow"]
assignee: "codex"
---

# tkt undo: revert last mutation

## Current executable contract — 2026-11-03

Implement only a Git revert of an exactly recognized tkt mutation at HEAD. Require a clean index and tracked worktree, validate affected ticket/plan paths and recognized mutation structure, and reject arbitrary commits even when author or message prefix resembles tkt. Never expose reset-based internal race recovery. Refuse undo-of-undo and ambiguous/repeated history in the first implementation; repeated undo is historical speculation, not approved scope. Cover conflicts, dry-run and push-disabled behavior.

This contract supersedes conflicting historical proposals below.

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

- [x] `tkt undo` reverts last tkt-generated commit
- [x] Refuses to undo non-tkt commits (safe boundary)
- [x] Pushes the revert if push.enabled
- [x] Shows what was reverted in the confirmation message
- [x] Integration test covering undo of close, claim, and new


## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:ready-for-agent`; stream `cli`.

Deferred feature; src/cli.rs has no undo command. Match actual chore(tickets) commit messages, refuse unrelated HEAD, and preserve push gating.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.


## Reactivation — 2026-11-03

The owner requested review of all parked work and execution of actionable tickets. This supersedes earlier blanket deferral. See [the parked-work review](../docs/parked-review-2026-11-03.md) for current scope and sequencing. Independent critic review is required before closure.

## Recognized first-version boundary — 2026-11-03

The implemented safe recognizer uses canonical versioned Git receipts for new/claim/close, binding full before/after trees, parent, path, blob and mode. Legacy/message-only and other operations remain unrecognized; this is structural provenance, not author authentication. Ordinary mutations remain compatible when not eligible for a receipt. Undo uses an exact staged inverse and checks the commit before a single push, without automatic rebase on rejection. Failed apply/commit states and durable local commits are retained for explicit inspection. See [the supported recovery contract](../docs/undo.md). This scope supersedes the historical trivial-prefix and repeated-undo proposal. Parent #191 remains incomplete for the broader all-mutator effect matrix.

## Resolution (2026-11-04)

Implemented receipt-bound Git revert for recognized new claim and close at clean HEAD, exact inverse checks and push-gated recovery without reset or automatic rebase. Final gate and independent critic accepted.

### Verification
1. ✓ Versioned receipts bind recognized new claim and close transitions to exact parent tree path blob and mode; arbitrary or unsupported history is refused — "Version1 binds actual parent and full trees plus each allowed path blob and mode; semantic operation and plan checks reject arbitrary, legacy, changed, duplicated or unsupported history. See docs/undo.md and docs/evidence/undo-79.json."
2. ✓ Git revert preserves original history and exact prior ticket/plan state; dirty state conflicts hooks and push failures retain recoverable work without reset or automatic rebase — "Eight public tests verify exact tree restoration, retained original history, read-only preview, dirty/conflict states, Windows locked revert, hook failures, successful publication, durable failed pushes, no-push and upstream advance without automatic rebase. No real project history was reverted."
3. ✓ Public integration and final local gate pass with standalone guidance and independent critic acceptance — "Final mise run check passed 443 Rust tests and 42 Node fixtures, clean fmt and zero clippy warnings; one existing Windows skip. Critic accepted final source after the empty-string validator lint correction. All guidance surfaces updated; artifacts promoted/recycled and shared outputs retained for ongoing work."

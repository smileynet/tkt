---
id: "274"
title: "Fix pre-push entry-point detection across canonicalized paths"
status: in_progress
blocked_by: []
priority: high
validation_criteria:
  - "Direct execution and actual-push fixtures invoke the gate across canonicalized paths; all pre-push and Node tooling tests pass"
tags: ["tooling", "kind:bug", "stream:quality"]
assignee: "kimi"
---
# Fix pre-push entry-point detection across canonicalized paths

## Intent source

Sam approved the 2026-11-08 repository-audit fix plan. The failure was reproduced from the current checkout with `node --test tools/pre-push-check.test.mjs`; this regresses the verification behavior delivered in #249.

## Observed problem

On macOS, a fixture invoked through `/var/...` is loaded by Node through `/private/var/...`. The entry-point comparison in `tools/pre-push-check.mjs` therefore does not call `main()`. Three tests fail, and the actual-push fixture completes without invoking the configured gate.

## What to build

Make direct-script detection robust when the invocation path and module URL differ only by filesystem canonicalization. Preserve the existing own-checkout guard, explicit opt-in installation, foreign-hook preservation, and all push verification semantics.

## Acceptance criteria

- [ ] Direct invocation runs `main()` when path aliases differ only by symbolic-link/canonical-path normalization.
- [ ] The actual Git push fixture invokes the gate, and a failing gate blocks the push.
- [ ] Installation, ownership, linked-worktree, malformed-input, timeout, and missing-runtime tests retain their behavior.
- [ ] `node --test tools/pre-push-check.test.mjs` and the complete Node tooling suite pass.

## Out of scope

Enabling the hook, changing hosted-CI policy, or weakening the full-check contract.

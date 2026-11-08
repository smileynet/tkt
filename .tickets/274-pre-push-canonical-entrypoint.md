---
id: "274"
title: "Fix pre-push entry-point detection across canonicalized paths"
status: done
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

- [x] Direct invocation runs `main()` when path aliases differ only by symbolic-link/canonical-path normalization.
- [x] The actual Git push fixture invokes the gate, and a failing gate blocks the push.
- [x] Installation, ownership, linked-worktree, malformed-input, timeout, and missing-runtime tests retain their behavior.
- [x] `node --test tools/pre-push-check.test.mjs` and the complete Node tooling suite pass.

## Out of scope

Enabling the hook, changing hosted-CI policy, or weakening the full-check contract.

## Resolution (2026-11-08)

Canonicalize the invoked script path before comparing its file URL, so direct execution and actual Git pushes reliably invoke the gate.

### Verification
1. ✓ Direct execution and actual-push fixtures invoke the gate across canonicalized paths; all pre-push and Node tooling tests pass — "Commit 20edac8; node --test tools/pre-push-check.test.mjs passed 7/7 including real push success/failure; node --test tools/*.test.mjs passed 96/96; mise run check passed formatting, Clippy, tooling fixtures, and all Rust tests."

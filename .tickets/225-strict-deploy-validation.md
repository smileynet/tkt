---
id: "225"
title: "Make upstream deployment validation enforce strict plugin and catalog checks"
status: done
blocked_by: []
validation_criteria:
  - "Deployment fixtures reject a root CLAUDE.md and missing catalog icon"
tags: ["tooling"]
---

# Make upstream deployment validation enforce strict plugin and catalog checks

## What to build

Make deployment reject plugin warnings that the maintainer's strict checks reject.
Discovered in the 2026-10-29 wrap-session retrospective after root CLAUDE.md and
missing catalog icon warnings escaped deployment checks.

## Context

Read tools/deploy.mjs around its native Claude validation call and
tools/probe-claude-validation.mjs. The deployment engine is vendored from
grey-matter/scripts/deploy.mjs: change the owner source first, then copy it here.
Current deployment intentionally omits --strict; catalog warnings are not gated.
Reuse existing validation rather than adding a parallel validator.
The local plugin-layout regression test now catches this repository's concrete traps.

## Out of scope

Releases, version bumps, and changes to harness ownership or duplicate-skill policy.

## Acceptance criteria

- [x] Upstream deployment and its vendored copy reject strict manifest/content warnings
- [x] Catalog warnings are rejected when catalog validation is requested
- [x] Fixtures seed root CLAUDE.md and a missing icon and observe failure
- [x] A clean plugin deploys successfully and guidance describes the enforced checks

## Resolution (2026-10-29)

Strict native plugin and marketplace validation is enforced by the canonical deployment check and vendored copy; optional directory profile gates catalog warnings. Added native seeded regression fixtures, updated the validation probe and guidance, and moved upstream Claude guidance into its project directory.

### Verification
1. ✓ Deployment fixtures reject a root CLAUDE.md and missing catalog icon — "Canonical and vendored node --test deploy-validation.test.mjs fixtures reject a seeded root CLAUDE.md and missing catalog icon, then accept the clean plugin and valid icon. Node 24 mise run check passed (38 Node tests, 306 Rust tests; fmt and clippy clean). Upstream node scripts/check-repo.mjs passed all 16 invariants. Both native probes passed; tkt check --profile directory passed; both default deployments completed successfully. Standards and Spec reviews found no actionable issues."

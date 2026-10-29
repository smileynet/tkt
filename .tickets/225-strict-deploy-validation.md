---
id: "225"
title: "Make upstream deployment validation enforce strict plugin and catalog checks"
status: in_progress
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

- [ ] Upstream deployment and its vendored copy reject strict manifest/content warnings
- [ ] Catalog warnings are rejected when catalog validation is requested
- [ ] Fixtures seed root CLAUDE.md and a missing icon and observe failure
- [ ] A clean plugin deploys successfully and guidance describes the enforced checks

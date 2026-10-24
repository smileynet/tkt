---
id: "217"
title: "Migrate v0.4 bare skill symlinks before plugin upgrades"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "A dry-run inventories the v0.4 installer symlinks without mutation and execute removes only links proven to target the explicitly supplied old tkt skill directory."
  - "Regular directories, links to other targets, and unrelated user files remain unchanged; second execution is identical and removed/dangling old-source cases are explicit."
  - "Upgrade guidance and isolated tests cover all three former harness roots and bind candidate regeneration to the documentation/runtime change."
tags: ["stream:tooling"]
---

# Migrate v0.4 bare skill symlinks before plugin upgrades

## Intent source

Found during grey-matter release-readiness ticket 69 on 2026-10-24. A clean
v0.4.0 archive ran its original `tools/deploy-skills.sh` into an isolated home.
Deploying candidate `79302cb4f507ce030b367b8723e23946780dc3b6` then left both
`~/.codex/skills/tkt/SKILL.md` and the new `tkt-core/SKILL.md` readable. The old
installer had no ownership manifest, so the new deployer correctly preserved it.

## What to build

Provide an explicit, preview-first migration from the old installer's three skill
symlinks to the four current skills. Match only the user-supplied old target,
including dangling links after an in-place source upgrade; never infer ownership
of copied directories or other links. Preflight all roots before unlinking, leave
source files intact, and keep normal deployment as one subsequent operation.

Read `git show v0.4.0:tools/deploy-skills.sh`, `tools/deploy.mjs`, README install
sections and `tools/migrate-legacy-skills.test.mjs`. Do not change the shared
vendored engine or delete legacy steering as part of this skill-only migration.
Grey-matter ticket 69 owns the coordinated install matrix, so do not duplicate it
here. The tkt source/guidance change requires a new 0.5.0 candidate; unchanged grey
candidate evidence remains reusable.

## Acceptance criteria

- [ ] Preview leaves all three original links unchanged; execution removes only matching links.
- [ ] Foreign content, unrelated files and source bytes survive; repeat and dangling-link controls pass.
- [ ] README upgrade instructions and isolated tests cover all former roots; regenerate the tkt candidate.

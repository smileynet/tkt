---
id: "279"
title: "Check skill body size before deployment"
status: in_progress
blocked_by: []
priority: medium
validation_criteria:
  - "Static skill checking rejects a SKILL.md body over 24000 bytes with a regression fixture and accepts the boundary case"
  - "Skill changes select the byte-limit check before deployment while unrelated changes retain their existing check scope"
  - "The new check is measured on this host and remains within the existing local check budget"
tags: ["tooling", "kind:enhancement", "triage:ready-for-agent", "stream:quality"]
spec: "tools/check-skill-syntax.mjs"
---
## Intent source

2026-11-08 retrospective. Strict deployment caught tkt skill bodies at 24068, 24051 and 24007 bytes only during deployment, requiring repeated late trims after implementation.

## What to change

Extend the existing static skill-syntax validation with the established 24000-byte body limit and add boundary/regression fixtures. Wire the cheap static check into the existing source-selected tooling check for skill and relevant packaging changes; do not create a second policy or require native harness validation for unrelated work.

Measure the completed check on this host and record the result. Preserve existing deployment validation as the final authority.

## Acceptance criteria

- [ ] Static skill checking rejects a SKILL.md body over 24000 bytes with a regression fixture and accepts the boundary case
- [ ] Skill changes select the byte-limit check before deployment while unrelated changes retain their existing check scope
- [ ] The new check is measured on this host and remains within the existing local check budget
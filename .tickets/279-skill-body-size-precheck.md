---
id: "279"
title: "Check skill body size before deployment"
status: done
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

- [x] Static skill checking rejects a SKILL.md body over 24000 bytes with a regression fixture and accepts the boundary case
- [x] Skill changes select the byte-limit check before deployment while unrelated changes retain their existing check scope
- [x] The new check is measured on this host and remains within the existing local check budget

## Resolution (2026-11-08)

Added UTF-8 body-byte enforcement to the existing static skill validator, boundary and multibyte regressions, current-pack coverage, and source-selected wiring for skill/plugin changes. Updated the reviewed vendor hash. Test fixtures use temporary directories with registered cleanup; no implementation artifacts retained.

### Verification
1. ✓ Static skill checking rejects a SKILL.md body over 24000 bytes with a regression fixture and accepts the boundary case — "node --test tools/check-skill-syntax.test.mjs tools/check-tools.test.mjs passed 6/6; fixtures accepted exactly 24000 UTF-8 bytes and rejected 24001-byte and 24002-byte multibyte bodies. The final node tools/check-tools.mjs gate passed 50/50."
2. ✓ Skill changes select the byte-limit check before deployment while unrelated changes retain their existing check scope — "Source-selection tests passed for skills/tkt-core/SKILL.md and .claude-plugin/plugin.json, proved src/main.rs does not select the new check, and node tools/check-tools.mjs --plan listed tools/check-skill-syntax.test.mjs for the actual change set."
3. ✓ The new check is measured on this host and remains within the existing local check budget — "time node tools/check-skill-syntax.mjs passed all four repository skills in 0.604 s real time on this macOS host; the complete selected tooling gate finished in 7.55 s, within the 60 s warm-check budget."

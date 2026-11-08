---
id: "265"
title: "Build a self-contained tkt Kimi plugin package"
status: in_progress
blocked_by: []
validation_criteria:
  - "The builder emits all four tkt skills, linked resources, legal notices, localization, and Kimi metadata into dist/kimi-plugin without Claude hooks."
  - "Package fixtures prove idempotent rebuilds, drift repair, resource integrity, and exclusion of hooks, MCP servers, and automatic installers."
  - "Kimi static validation completes with zero errors and any warnings are recorded."
tags: ["tooling"]
---
## What to build

Add a Kimi-owned source manifest and a standalone builder that reuses tools/deploy.mjs with an explicit staging root. Generate dist/kimi-plugin with the four flat tkt-* skills, their transitive resources, legal notices, localization, and the existing product icon. Do not change default deployment or include Claude-only hooks.

Intent: Grey Matter ticket 271 needs tkt-core in Kimi; the 2026-11-08 review found four standalone tkt skills and existing Claude/Codex packaging but no Kimi package.

## Acceptance criteria

- [ ] The source package contains all four skills and required resources exactly once.
- [ ] Rebuilding unchanged output is a no-op and changed or damaged output is repaired.
- [ ] The package has no hooks, MCP servers, setup script, or Grey Matter dependency.
- [ ] Focused fixtures and Kimi static validation pass, with warnings and unrun checks reported.
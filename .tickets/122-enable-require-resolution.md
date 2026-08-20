---
id: "122"
title: "Enable require_resolution across all projects"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "all projects in ~/code with .tickets/ have require_resolution=true in config"
  - "tkt audit across all projects shows 0 new missing-resolution findings on done tickets"
---

# Enable require_resolution across all projects

## Problem

224 of 373 audit findings (60%) are `missing-resolution` — tickets closed without any Resolution section. This is the dominant quality debt across all projects. The fix is trivial: enable the config gate so future closures require it.

## What to build

For each project in ~/code with a `.tickets/config.toml`:
1. Set `require_resolution = true` under `[close]`
2. If no config exists, create one with the setting

Projects: archwright, crew-research, game-slicer, gdhelper-cli, gdhelper-harness, gdhelper-log, gdhelper-mcp, gdquest-vault, godot-helper, godot-knowledge, lacrosse-bosse-helper, mebo-slicer, recall, teach-me, tkt.

## Context

- **Command:** `tkt config --set close.require_resolution=true` in each project
- **Impact:** Only affects FUTURE closures. Existing done tickets without resolutions remain as-is (historical debt).
- **No breakage risk:** Agents will simply need to provide `--resolution "..."` when closing.

## Acceptance criteria

- [ ] All 15 projects have require_resolution=true
- [ ] tkt close without --resolution is rejected in each project
- [ ] No regressions in any project's test suite or existing workflows

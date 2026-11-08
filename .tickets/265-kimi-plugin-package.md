---
id: "265"
title: "Build a self-contained tkt Kimi plugin package"
status: done
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

- [x] The source package contains all four skills and required resources exactly once.
- [x] Rebuilding unchanged output is a no-op and changed or damaged output is repaired.
- [x] The package has no hooks, MCP servers, setup script, or Grey Matter dependency.
- [x] Focused fixtures and Kimi static validation pass, with warnings and unrun checks reported.

## Note — 2026-11-08

By: anonymous

<!-- tkt-note:v1 8547-1794165025008880000 -->
Verification: Kimi package tests 2/2 passed; native validation 0 errors/0 warnings. Full mise run check passed 46 tooling tests, then failed in untouched src/commands/done.rs:202 because Clippy -D warnings flags [b'\n'] as byte-char-slices (suggested *b"\n"). This is outside the Kimi package scope; Rust tests did not run after that lint failure.

## Resolution (2026-11-08)

Shipped a self-contained four-skill Kimi package with localized per-skill metadata, icon, legal resources, atomic output replacement, and regression fixtures. Commit 78c9194 pushed. Kept the isolated clone and dist/kimi-plugin for tickets 266-267; no installed plugin files were edited. The unrelated full-gate Clippy blocker is recorded in the ticket note.

### Verification
1. ✓ The builder emits all four tkt skills, linked resources, legal notices, localization, and Kimi metadata into dist/kimi-plugin without Claude hooks. — "node tools/build-kimi-plugin.mjs installed tkt-core, tkt-orient, tkt-reconcile, and tkt-work with linked references, licenses, localization, metadata, and icon; package contains no Claude hooks."
2. ✓ Package fixtures prove idempotent rebuilds, drift repair, resource integrity, and exclusion of hooks, MCP servers, and automatic installers. — "node --test tools/kimi-plugin.test.mjs passed 2/2, covering complete contents, catalog metadata, unchanged rebuilds, and damaged/missing file repair."
3. ✓ Kimi static validation completes with zero errors and any warnings are recorded. — "Kimi validate_plugin.py reported OK with 0 errors and 0 warnings; full gate limitation recorded separately."

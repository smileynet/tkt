---
id: "287"
title: "single-source the plugin manifests: root plugin.json vs .claude-plugin/plugin.json, explicit skills, version gate against Cargo.toml"
status: open
blocked_by: []
priority: medium
validation_criteria:
  - "an audit names every reader of the root plugin.json; it is deleted or generated from .claude-plugin/plugin.json (ZCode probes only the dotted dirs; the vendored deployer reads only .claude-plugin)"
  - ".claude-plugin/plugin.json declares skills explicitly (./skills) instead of relying on ZCode's default-root collection (works, trial-verified 2026-11-08, but implicit)"
  - "a pre-push/static gate fails when Cargo.toml, root plugin.json (while it exists), and .claude-plugin/plugin.json versions disagree; release-artifacts stamps the manifest version from the release authority rather than hand-sync"
tags: ["kind:enhancement", "stream:tooling"]
---
# Single-source plugin manifests and version authority

## Intent source

Grey-matter proposal 23 (T1+T2+T3), informed by the Windows ZCode installation trial and grey-matter prior-art research. The 2026-11-15 board reconciliation expanded the original record without changing its scope.

## Context

- Candidate authorities are `Cargo.toml`, root `plugin.json`, and `.claude-plugin/plugin.json`; all currently require manual version agreement.
- ZCode probes the dotted plugin directories, while the vendored deployer reads `.claude-plugin/plugin.json`.
- ZCode currently collects root skills implicitly; the package should declare `./skills` explicitly.
- Release tooling must stamp versions from the release authority rather than adding another manual synchronization step.

## What to build

Audit every reader of root `plugin.json`, then delete it or generate it from `.claude-plugin/plugin.json` so one manifest owns plugin metadata. Declare skills explicitly in the dotted manifest. Add a static/pre-push version-agreement gate while duplicate authorities exist, and make release artifact generation stamp manifest versions from the coordinated release authority.

## Acceptance criteria

- [ ] A documented reader audit covers root `plugin.json`; the file is removed or generated from the dotted manifest.
- [ ] `.claude-plugin/plugin.json` explicitly declares `./skills`.
- [ ] A static/pre-push fixture fails on version disagreement among Cargo and retained manifests.
- [ ] Release artifacts derive manifest versions from the release authority rather than hand edits.
- [ ] Focused fixtures and the full maintainer gate pass.

## Out of scope

Publishing the marketplace or changing release authorization; ticket 288 owns publication separately.

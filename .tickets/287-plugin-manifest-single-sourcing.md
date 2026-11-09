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
From grey-matter proposal 23 (T1+T2+T3), informed by the Windows ZCode install trial and prior-art research (grey-matter docs/agents/research/zcode-marketplace-prior-art-20261108.md): Claude docs warn never to set version in both entry and manifest (plugin.json silently wins). Three authorities currently agree at 0.4.0 by hand.
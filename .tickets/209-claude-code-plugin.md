---
id: "209"
title: "Package tkt as a Claude Code plugin (.claude-plugin manifest + marketplace)"
status: in_progress
blocked_by: []
priority: medium
validation_criteria:
  - ".claude-plugin/plugin.json and .claude-plugin/marketplace.json pass 'claude plugin validate --strict'"
  - "claude --plugin-dir . loads all four tkt-* skills (namespaced tkt:tkt-*)"
  - "plugin.json versions (Claude + agy) are bumped by cargo-release and a cargo test fails when either drifts from Cargo.toml"
tags: ["agent-integration", "release"]
---

# Package tkt as a Claude Code plugin (.claude-plugin manifest + marketplace)

## What to build

tkt already ships its skills to every harness by copying them (`tools/deploy-skills.sh`) and to
agy as a plugin (root `plugin.json`). Claude Code has its own plugin format. Packaging tkt in
that format makes `/plugin install` an install path that updates itself, and it lets
grey-matter's Claude plugin declare tkt as a dependency (grey-matter proposal 07, ticket 22).

- `.claude-plugin/plugin.json`: name `tkt`, the same metadata as the agy manifest, and skills
  from the default `skills/` dir (no `skills` key needed).
- `.claude-plugin/marketplace.json`: marketplace `tkt` with one entry, `source: "./"`, so
  `/plugin marketplace add smileynet/tkt` then `/plugin install tkt@tkt` works.
- Version sync: Claude Code caches plugins by `version`, so a stale manifest means users never
  get updates. `release.toml` bumps both manifests with `pre-release-replacements`, and a
  cargo test pins both to `CARGO_PKG_VERSION`.

Out of scope (follow-ups): a SessionStart hook that surfaces the frontier; making
`deploy-skills.sh` skip `~/.claude/skills` when the plugin is installed, to avoid listing each
skill twice.

## Acceptance criteria

- [ ] `.claude-plugin/plugin.json` and `.claude-plugin/marketplace.json` pass `claude plugin validate --strict`
- [ ] `claude --plugin-dir .` loads all four tkt-* skills (namespaced `tkt:tkt-*`)
- [ ] Both plugin.json versions are bumped by cargo-release, and a cargo test fails when either drifts from Cargo.toml

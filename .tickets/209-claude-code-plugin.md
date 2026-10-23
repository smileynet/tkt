---
id: "209"
title: "Package tkt as a Claude Code plugin (.claude-plugin manifest + marketplace)"
status: open
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

TBD

## Acceptance criteria

- [ ] TBD

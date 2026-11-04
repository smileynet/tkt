---
id: "255"
title: "Keep import guidance within native skill loading limits"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "Core import routing and standalone reference preserve guidance within the observed native body limit"
  - "Strict native validation and refreshed workflow targets are verified with deployment limitations explicit"
tags: ["kind:bug", "stream:guidance", "triage:ready-for-agent"]
---
# Keep GitHub import guidance fully loadable

## Context

The post-86 workflow refresh observed Antigravity 1.4.12 strict validation refusing the core skill body: 24,287 bytes exceeds the 24,000-byte loaded-body limit. The CLI is deployed and Codex/Claude/Kiro/OpenCode refreshed; Antigravity remains on the prior pack until corrected. Preserve native strict validation and root ownership guards.

## Acceptance criteria

- [ ] Core skill fits the observed native loading limit and routes import details to a valid standalone reference without dropping protocol guidance
- [ ] Strict native validation and a complete workflow refresh succeed, with active/staged skill bytes compared and limitations recorded

## Verification

Run pack lint and native strict validation; check reference target/anchor and standalone skill rules. Independent critic review is required before closure. No Rust behavior or harness limit bypass is in scope.

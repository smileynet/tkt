---
id: "255"
title: "Keep import guidance within native skill loading limits"
status: done
blocked_by: []
priority: high
validation_criteria:
  - "Core import routing and standalone reference preserve guidance within the observed native body limit"
  - "Strict native validation and refreshed workflow targets are verified with deployment limitations explicit"
tags: ["kind:bug", "stream:guidance", "triage:ready-for-agent"]
assignee: "codex"
---
# Keep GitHub import guidance fully loadable

## Context

The post-86 workflow refresh observed Antigravity 1.4.12 strict validation refusing the core skill body: 24,287 bytes exceeds the 24,000-byte loaded-body limit. The CLI is deployed and Codex/Claude/Kiro/OpenCode refreshed; Antigravity remains on the prior pack until corrected. Preserve native strict validation and root ownership guards.

## Acceptance criteria

- [x] Core skill fits the observed native loading limit and routes import details to a valid standalone reference without dropping protocol guidance
- [x] Strict native validation and a complete workflow refresh succeed, with active/staged skill bytes compared and limitations recorded

## Verification

Run pack lint and native strict validation; check reference target/anchor and standalone skill rules. Independent critic review is required before closure. No Rust behavior or harness limit bypass is in scope.

## Resolution (2026-11-04)

Moved import detail behind a mandatory standalone reference while preserving the critical inline boundaries, including no writes to GitHub. Native strict validation now passes. Complete refresh and six target comparisons pass; independent critic verified the two source hashes and accepted. Rust behavior is unchanged. Evidence: docs/evidence/import-guidance-255.json.

### Verification
1. ✓ Core import routing and standalone reference preserve guidance within the observed native body limit — "Final core skill is 23655 UTF-8 bytes, with calculated body 23179 versus observed native body limit24000. Mandatory references/commands.md#github-import exists and retains the complete contract. Pack lint, Claude strict and guidance coverage45/355 pass; critic verified exact hashes."
2. ✓ Strict native validation and refreshed workflow targets are verified with deployment limitations explicit — "Final agy strict and complete deployment pass, with15skillfiles byte-identical across6active/stagedtargets. KiroCrewpayload built but CLI absent; activation remains unobserved. Source/code86remainsinstalledfea00ab; managed build/cache outputs retained and unknown ownership preserved."

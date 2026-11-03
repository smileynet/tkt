---
id: "246"
title: "Apply approved conditional guidance pointers and update coverage matrix"
status: done
blocked_by: ["245"]
spec: "docs/agents/agents-restructuring-proposal.md"
validation_criteria:
  - "Every current rule has verified coverage and source links; no policy is silently weakened"
  - "Independent critic and applicable guidance/deployment checks pass with evidence"
tags: ["guidance", "kind:enhancement", "triage:ready-for-agent", "stream:guidance"]
assignee: "codex"
---
# Apply approved conditional guidance pointers

## Intent source
Owner-approved staged proposal 226, docs/decision-record-2026-11-03.md decision 17, and the verified current coverage map from ticket 245.

## What to build
Switch AGENTS.md to the approved information hierarchy. Keep project identity, workflow, full verification command, commit/fetch/push/deploy, release authorization, close-only/body-ownership/plan synchronization and high-frequency hard constraints inline. Replace reference material with precise conditional pointers to verified homes. Update docs/agents/agent-guidance-surfaces.md in the same change as moved CLI guidance. End transition duplication from stage one.

## Acceptance criteria
- [x] Every removal is backed by stage one's authoritative home and a clear read trigger.
- [x] Global standing rules, current verification policy, standalone-skill constraints, race/push/atomic-write boundaries and release permissions are preserved.
- [x] Guidance-surface matrix/checklist and retained pointers match the final structure.
- [x] Independent critic accepts the source-to-destination coverage proof and documentation checks pass.

## Verification
This changes guidance structure, not CLI behavior, versions or release authority. Use applicable guidance/native checks and reviewed path/coverage evidence. Deploy changes by the project workflow after verification.

## Resolution (2026-11-03)

Applied approved conditional guidance structure, retained operational/safety boundaries, updated guidance matrix and ended temporary duplication. Critic accepted; evidence promoted to docs/evidence/guidance-extraction-246.md. Disposable helpers trashed; no new build output; shared target retained for final deployment.

### Verification
1. ✓ Every current rule has verified coverage and source links; no policy is silently weakened — "Every removed unit preserved at its mapped reachable home; final checker verifies45units355sourcepositions and exact human map; docs/evidence/guidance-extraction-246.md"
2. ✓ Independent critic and applicable guidance/deployment checks pass with evidence — "Independent critic accepted final runbook/map/matrix/checker hashes; negative controls and native checks pass; no gate/safety/release authority weakened"

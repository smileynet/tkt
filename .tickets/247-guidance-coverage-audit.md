---
id: "247"
title: "Verify and deploy the staged guidance restructuring"
status: open
blocked_by: ["246"]
spec: "docs/agents/agents-restructuring-proposal.md"
validation_criteria:
  - "Every current rule has verified coverage and source links; no policy is silently weakened"
  - "Independent critic and applicable guidance/deployment checks pass with evidence"
tags: ["guidance", "kind:enhancement", "triage:ready-for-agent", "stream:guidance"]
---
# Verify and deploy staged guidance restructuring

## Intent source
Owner-approved staged proposal 226 and docs/decision-record-2026-11-03.md decision 17; stages 245 and 246 establish and apply current reference homes.

## What to build
Independently compare the final AGENTS.md, reference homes and coverage matrix against the current-rule inventory. Verify source links, conditional triggers, standalone adopter behavior and installed pack refresh. Capture exact source identities and observed check results. Do not infer new release or automatic hosted-CI authorization from the restructuring.

## Acceptance criteria
- [ ] Every current rule is retained or moved with an authoritative home and reliable trigger; any deliberate retirement has explicit owner authority.
- [ ] No stale duplicate instruction or broken reference remains across affected guidance surfaces.
- [ ] Native manifest/inventory checks and applicable guidance fixtures pass; install refresh is observed and evidence records limits.
- [ ] Independent critic accepts final coverage and source identities; dashboard/plan reflect the completed stages.

## Verification
Use tools/deploy.mjs check followed by one deployment, native Claude checks and applicable guidance fixtures. Keep source acceptance distinct from rendered dashboard evidence and from unavailable historical proof.

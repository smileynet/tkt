---
id: "247"
title: "Verify and deploy the staged guidance restructuring"
status: done
blocked_by: ["246"]
spec: "docs/agents/agents-restructuring-proposal.md"
validation_criteria:
  - "Every current rule has verified coverage and source links; no policy is silently weakened"
  - "Independent critic and applicable guidance/deployment checks pass with evidence"
tags: ["guidance", "kind:enhancement", "triage:ready-for-agent", "stream:guidance"]
assignee: "codex"
---
# Verify and deploy staged guidance restructuring

## Intent source
Owner-approved staged proposal 226 and docs/decision-record-2026-11-03.md decision 17; stages 245 and 246 establish and apply current reference homes.

## What to build
Independently compare the final AGENTS.md, reference homes and coverage matrix against the current-rule inventory. Verify source links, conditional triggers, standalone adopter behavior and installed pack refresh. Capture exact source identities and observed check results. Do not infer new release or automatic hosted-CI authorization from the restructuring.

## Acceptance criteria
- [x] Every current rule is retained or moved with an authoritative home and reliable trigger; any deliberate retirement has explicit owner authority.
- [x] No stale duplicate instruction or broken reference remains across affected guidance surfaces.
- [x] Native manifest/inventory checks and applicable guidance fixtures pass; install refresh is observed and evidence records limits.
- [x] Independent critic accepts final coverage and source identities; dashboard/plan reflect the completed stages.

## Verification
Use tools/deploy.mjs check followed by one deployment, native Claude checks and applicable guidance fixtures. Keep source acceptance distinct from rendered dashboard evidence and from unavailable historical proof.

## Resolution (2026-11-03)

Completed staged guidance audit and current native deployment with renewed critic acceptance after clean upstream integration. CLI install/freshness passed at f864424 before closure metadata. Durable proof in docs/evidence/guidance-deployment-247.md and JSON; source/runtime identities recorded. Optional Kiro Crew activation skipped because CLI is absent; wider audit remains failed for unrelated unowned/legacy roots, none deleted. Helpers/raw logs trashed; native caches and scoped builds retained per evidence lifecycle. Owner-parked249 stays deferred.

### Verification
1. ✓ Every current rule has verified coverage and source links; no policy is silently weakened — "Final45-unit355-position coverage with explicit ownerCIamendment verified; native24runtime and15skill identities match; critic renewed acceptance after upstreamintegration; docs/evidence/guidance-deployment-247.md and JSON"
2. ✓ Independent critic and applicable guidance/deployment checks pass with evidence — "NativeClaude/Codex fromlocalcheckout refreshed; Kiro/OpenCode/Agy installed; cargo install and freshness at reviewedsourcef864424 passed.42Node fixtures plus6deploy guards and10dashboard tests pass; Rust399 source unchanged. Wideraudit foreignfail andKiroCrewbuild-only CLIabsence explicitly recorded."

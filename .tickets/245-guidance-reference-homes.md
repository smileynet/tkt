---
id: "245"
title: "Prepare approved guidance reference homes and current coverage map"
status: done
blocked_by: ["226"]
spec: "docs/agents/agents-restructuring-proposal.md"
validation_criteria:
  - "Every current rule has verified coverage and source links; no policy is silently weakened"
  - "Independent critic and applicable guidance/deployment checks pass with evidence"
tags: ["guidance", "kind:enhancement", "triage:ready-for-agent", "stream:guidance"]
assignee: "codex"
---
# Prepare approved guidance reference homes

## Intent source
Owner approval in docs/decision-record-2026-11-03.md, decision 17, accepts staged implementation of the proposal in docs/agents/agents-restructuring-proposal.md. Proposal ticket 226 is complete; implementation is separate work.

## What to build
Re-inventory the then-current AGENTS.md, including all CLI, query, cancellation, audit, configuration and server-era additions since the proposal baseline. Establish the approved plugin-maintenance, architecture-reference and tkt-contract references with every current rule assigned an authoritative home. Prepare an exhaustive before/after coverage map before removing always-loaded text. Temporary duplication during this transition is deliberate and ends in the next stage.

## Acceptance criteria
- [x] Every current section, rule and constraint is mapped to an exact retained or referenced home with a read trigger.
- [x] Approved reference files preserve current operational, lifecycle, parser, platform, deployment and release constraints without asserting new behavior.
- [x] Full code verification and manual-only hosted-CI policy remain unchanged; publication remains separately authorized.
- [x] Independent critic checks the current inventory against the created references; links and guidance checks pass.

## Verification
Documentation-only accuracy, links, scope and native pack checks as applicable; no Rust rebuild unless code changes. Preserve all global standing rules. Coordinate the optional-consumer CI documentation from ticket 159 without inventing a scheduling dependency.

## Resolution (2026-11-03)

Prepared approved reference homes and exhaustive frozen coverage before extraction. Critic accepted; sourceAGENTS unchanged. Intentional maintainer-tooling home refinement recorded, release authority preserved. Evidence promoted to docs/evidence/guidance-homes-245.md; disposable helpers trashed; no new build output.

### Verification
1. ✓ Every current rule has verified coverage and source links; no policy is silently weakened — "Frozen355line source partitioned into45verified units with exact retained/referenced homes and read triggers; sourceAGENTS unchanged; docs/evidence/guidance-homes-245.md"
2. ✓ Independent critic and applicable guidance/deployment checks pass with evidence — "Independent critic accepted final inventory/reference/checker identities; positive and removed-rule negative control pass; link, scope and pack prechecks pass without policy changes"

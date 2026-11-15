---
id: "293"
title: "Re-center documentation on the core workflow and refresh domain vocabulary"
status: done
blocked_by: []
priority: medium
validation_criteria:
  - "documentation checks, links, and native skill body-size gates pass"
  - "glossary and configuration wording match current CLI behavior"
  - "mise run check passes and guidance deployment succeeds"
tags: ["stream:workflow", "kind:enhancement", "triage:ready-for-agent"]
assignee: "kimi"
---
# Re-center documentation on the core workflow and refresh domain vocabulary

## Intent source

Owner-approved audit on 2026-11-13 and implementation approval on 2026-11-15. Five primary surfaces contain about two thousand lines with repeated feature contracts; this drift already produced contradictory authority, close-policy, and configuration documentation. `GLOSSARY.md` omits cancelled status and currently managed native fields.

## Context

- `README.md` is the human installation and usage entry point.
- `skills/tkt-core/SKILL.md` owns jobs-to-be-done and core invariants.
- `skills/tkt-core/references/commands.md` owns exhaustive command/recovery detail.
- `skills/tkt-work/SKILL.md` owns execution, not copies of every feature contract.
- `docs/agents/tkt-contract.md` owns contributor compatibility requirements.
- `GLOSSARY.md` must match `src/core/ticket.rs` and the native-relations contract.
- Quick-start and workflow repairs are implemented separately and must be preserved.

## What to build

Give each guidance surface one clear ownership boundary and remove redundant appended feature essays from the README/core/work entry points where a maintained reference already exists. Keep a concise core loop and links to parent/cancellation/plan/import/undo/hooks/detail contracts. Preserve all behavioral requirements in at least one authoritative maintained surface.

Refresh the glossary for all five statuses and managed native fields. Correct known wording drift such as `config --list` scope and blanket `all mutations push` language. Do not remove the new query match/tag/fields documentation.

## Acceptance criteria

- [x] README leads with a working quick start and core workflow; detailed appended contracts are represented by accurate reference links rather than duplicated essays.
- [x] `tkt-core` and `tkt-work` retain their JTBD/execution guidance while redundant feature-contract appendices are removed or reduced to links.
- [x] No behavioral contract is lost: command/reference and contributor docs retain the detailed behavior.
- [x] Glossary lists cancelled and all currently managed native fields with accurate ownership distinctions.
- [x] README correctly describes `config --list` user scope and limits push-on-write wording to ticket mutations.
- [x] Existing link/body-size/tooling checks and the full maintainer gate pass; changed guidance is deployed.

## Out of scope

Redesigning the public CLI, rewriting historical tickets, changing release authorization, or deleting standalone recovery references.

## Resolution (2026-11-15)

Consolidated duplicated feature contracts into maintained references, corrected configuration/push wording, and refreshed managed-field and cancellation vocabulary

### Verification
1. ✓ documentation checks, links, and native skill body-size gates pass — "node tools/check-tools.mjs passed 54/54 including native skill syntax/body-size gates; 38 changed-document local links and anchors resolved; CRLF-aware git diff --check passed"
2. ✓ glossary and configuration wording match current CLI behavior — "Glossary now covers all 13 managed fields and all five statuses; src/commands/config.rs confirms --list reads user configuration while --show reports resolved project values with sources"
3. ✓ mise run check passes and guidance deployment succeeds — "mise run check completed successfully and node tools/deploy.mjs refreshed the configured guidance installations"

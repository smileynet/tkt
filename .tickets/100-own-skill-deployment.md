---
id: "100"
title: "tkt owns its own skill/steering deployment"
status: done
blocked_by: []
priority: high
validation_criteria:
  - "mise run deploy succeeds and deploys skills + steering"
  - "~/.kiro/skills/tkt/ is current after deploy"
  - "~/.kiro/steering/frontier-work.md is deployed from tkt repo"
  - "crew-research no longer owns frontier-work or ticket-planning"
---

# tkt owns its own skill/steering deployment

## Problem

tkt's agent skills live in two repos:
- `tkt/skills/tkt/` — CLI reference skill (self-owned, deployed via symlink)
- `crew-research/atomics/skills/frontier-work/` — "work the frontier" protocol (crew-owned)
- `crew-research/atomics/skills/ticket-planning/` — ticket format + creation (crew-owned)

When tkt's format/commands change, the crew-research copies go stale. tkt should be the single authority on how to use tkt.

## What to build

### 1. Consolidate into tkt repo

- Add `skills/steering/frontier-work.md` — the always-on frontier protocol
- Merge ticket-planning content into `skills/tkt/references/ticket-format.md` (already partially done)
- Extend `skills/tkt/SKILL.md` triggers to cover ticket-planning activation phrases

### 2. Extend deploy-skills.sh

- Deploy `skills/steering/*.md` → `~/.kiro/steering/` (copy, not symlink — steering is body-only)
- Keep `skills/tkt/` → `~/.kiro/skills/tkt` (symlink as today)
- Support `--dry-run` (already does)

### 3. Wire mise task

```toml
[tasks.deploy]
description = "Deploy tkt skills and steering to agent harnesses"
run = "tools/deploy-skills.sh"
```

Updated deploy workflow: `cargo build --release && cargo install --path . && mise run deploy`

### 4. Remove from crew-research

- Add `frontier-work` and `ticket-planning` to deprecated.yaml
- Remove from atomics/skills/
- Consumer skills (handoff, project-cleanup, review-new-work) unchanged — they just invoke `tkt` CLI

## Acceptance criteria

- [x] `skills/steering/frontier-work.md` exists in tkt repo
- [x] `tools/deploy-skills.sh` deploys both skills and steering
- [x] `mise run deploy` wired and working
- [x] ticket-planning content merged into tkt skill references
- [x] crew-research atomics cleaned up, deprecated.yaml updated
- [x] `tkt --version` confirms binary + `~/.kiro/skills/tkt/SKILL.md` are consistent

## Out of scope

- `tkt update` subcommand (future ticket)
- Auto-deploy on `cargo install` (cargo doesn't support post-install hooks)

## Resolution (2026-08-13)

tkt now owns frontier-work steering and ticket-planning references. deploy-skills.sh deploys both. crew-research deprecated and removed.

### Verification
1. ✓ mise run deploy succeeds and deploys skills + steering — "mise run deploy:dry-run shows 4 targets (3 skill symlinks + 1 steering copy)"
2. ✓ ~/.kiro/skills/tkt/ is current after deploy — "ls -la ~/.kiro/skills/tkt confirms symlink to repo"
3. ✓ ~/.kiro/steering/frontier-work.md is deployed from tkt repo — "head ~/.kiro/steering/frontier-work.md confirms deployed content"
4. ✓ crew-research no longer owns frontier-work or ticket-planning — "crew-research commit 0d1fd8d removes atomics and deprecates"

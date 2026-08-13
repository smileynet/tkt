---
id: "100"
title: "tkt owns its own skill/steering deployment"
status: open
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

- [ ] `skills/steering/frontier-work.md` exists in tkt repo
- [ ] `tools/deploy-skills.sh` deploys both skills and steering
- [ ] `mise run deploy` wired and working
- [ ] ticket-planning content merged into tkt skill references
- [ ] crew-research atomics cleaned up, deprecated.yaml updated
- [ ] `tkt --version` confirms binary + `~/.kiro/skills/tkt/SKILL.md` are consistent

## Out of scope

- `tkt update` subcommand (future ticket)
- Auto-deploy on `cargo install` (cargo doesn't support post-install hooks)

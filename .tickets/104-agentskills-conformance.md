---
id: "104"
title: "Conform to agentskills.io and Agent Plugins 1.0.0 standards"
status: open
blocked_by: []
priority: medium
validation_criteria:
  - "plugin.json passes Agent Plugins 1.0.0 schema validation"
  - "SKILL.md has name field matching directory name"
  - "steering/ relocated outside skills/ discovery path"
---

# Conform to agentskills.io and Agent Plugins 1.0.0 standards

## Problem

tkt ships a `plugin.json` and `skills/tkt/SKILL.md` that are close to spec-conformant but have two critical issues and several minor ones:

1. **plugin.json** contains a non-standard `skills` array — Agent Plugins 1.0.0 is a closed schema that forbids undeclared top-level fields (§5.2). Skills are discovered by filesystem convention, not declared in the manifest.
2. **SKILL.md** uses `title:` instead of the required `name:` field.
3. **steering/** lives inside `skills/` — confusing for clients scanning for SKILL.md.

These were identified via research (agentskills.io spec, Agent Plugins 1.0.0 spec, 8 related standards compared).

## What to build

### 1. Fix plugin.json (critical)

Remove `skills` array. Add `author`. Keep all other valid fields.

```json
{
  "$schema": "https://agent-plugins.org/schemas/1.0.0/plugin.schema.json",
  "name": "tkt",
  "version": "0.1.0",
  "description": "Track tasks as markdown files in your git repo",
  "author": {"name": "smileynet"},
  "repository": "https://github.com/smileynet/tkt",
  "license": "MIT",
  "keywords": ["tickets", "task-management", "dependencies", "frontier"]
}
```

### 2. Fix SKILL.md frontmatter (critical)

Rename `title: tkt` → `name: tkt`. Keep `triggers` and `tools` as kiro-cli extensions (agentskills.io doesn't forbid extra frontmatter fields — only plugin.json is a closed schema). Add `compatibility` field.

```yaml
---
name: tkt
description: "Track tasks as markdown files in your git repo. Use when managing tickets, checking what's ready, claiming work, closing tasks, creating tickets, decomposing work, or validating project health."
compatibility: Requires git
triggers:
  - tkt
  - tickets
  ...
tools:
  - shell
---
```

### 3. Relocate steering/ (moderate)

Move `skills/steering/` → `steering/` at repo root. Update `deploy-skills.sh` to reference new path. The steering content is deployed separately (copy to `~/.kiro/steering/`) and shouldn't be in the skills discovery path.

### 4. Polish (minor)

- Add `compatibility: Requires git` to SKILL.md
- Ensure plugin.json `version` stays in sync with Cargo.toml (or document as manual)

## Context

- `plugin.json` — current non-conformant manifest
- `skills/tkt/SKILL.md` — main skill file
- `skills/steering/frontier-work.md` — steering file (not a skill)
- `tools/deploy-skills.sh` — references `skills/steering/`
- Research: `.scratch/research/agentskills-spec.md`, `tkt-conformance-gaps.md`, `agent-skill-prior-art.md`

## Acceptance criteria

- [ ] `plugin.json` has no `skills` field (removed)
- [ ] `plugin.json` has `author` field
- [ ] `SKILL.md` uses `name: tkt` (not `title`)
- [ ] `name` matches parent directory name (`tkt`)
- [ ] `steering/` moved out of `skills/` to repo root
- [ ] `deploy-skills.sh` updated to reference new steering path
- [ ] Deployment still works: `bash tools/deploy-skills.sh` succeeds
- [ ] kiro-cli still loads the skill correctly (triggers still fire)

## Out of scope

- MCP server declaration (tkt is a CLI, not a server)
- `allowed-tools` field (experimental, not widely supported yet)
- Automated version sync between plugin.json and Cargo.toml (separate ticket)
- OWASP Universal Skill Format conformance (proposed standard, not adopted)

---
id: "67"
title: "Adopt Agent Plugins standard — own and deploy tkt skills from this repo"
status: done
blocked_by: []
priority: normal
validation_criteria: 
  - "skills/tkt/SKILL.md with CLI usage documentation"
  - "plugin.json at repo root"
  - "tools/deploy-skills.sh deploys to kiro/claude/codex"
  - "Deployed skill readable from harness paths"
  - "tkt --version output matches plugin.json version"
---

# Adopt Agent Plugins Standard

## Context

crew-research is formalizing a skill import protocol (crew-research ticket 98) where tool
repos own and deploy their own skills. Currently crew-research owns the `ticket-planning`
skill. This ticket creates a `skills/` directory in tkt so the tool owns its own guidance.

**Decision needed:** Which skills should tkt own?
- `tkt` (CLI usage — doesn't exist yet, would document commands/workflows)
- `ticket-planning` (decompose specs into tickets — currently crew-research-owned)
- Both?

The `ticket-planning` skill is general-purpose (works without tkt via manual file editing).
A `tkt` skill would be tkt-specific. Recommendation: tkt owns a `tkt` skill (CLI usage);
`ticket-planning` stays in crew-research (it's methodology, not tool-specific).

## References to clone and review

```bash
# Agent Plugins spec
gh repo clone agentplugins/agent-plugins-spec ~/code/refs/agent-plugins-spec

# Agent Plugins example
gh repo clone agentplugins/agent-plugins-example ~/code/refs/agent-plugins-example

# crew-research (protocol design + existing skill to reference)
# ~/code/crew-research/.tickets/98-skill-import-protocol.md
# ~/code/crew-research/.scratch/research/agent-plugins-spec.md
# ~/code/crew-research/.references/agent-plugins-spec/
# ~/code/crew-research/.references/agent-plugins-example/

# archwright (deploy-skills.sh pattern to replicate)
# ~/code/archwright/tools/deploy-skills.sh
```

**Key docs:**
- Agent Plugins spec: https://agent-plugins.org/
- Agent Skills format: https://agentskills.io/specification
- Agent Plugins GitHub: https://github.com/agentplugins/agent-plugins-spec

## What to build

### 1. Create `skills/tkt/` (new tkt CLI skill)

```
skills/
  tkt/
    SKILL.md              # tkt CLI usage, workflows, frontier detection
    references/
      commands.md         # full command reference
      ticket-format.md    # .tickets/ file format contract
```

Content: document `tkt ready`, `tkt new`, `tkt batch`, `tkt close`, `tkt claim`,
`tkt edit`, `tkt sync-plan`, `tkt validate` — the commands and their workflows.
This is what an agent needs to use tkt effectively.

### 2. Add `plugin.json`

```json
{
  "$schema": "https://agent-plugins.org/schemas/1.0.0/plugin.schema.json",
  "name": "tkt",
  "version": "0.1.0",
  "description": "Ticket management CLI for .tickets/ convention — frontier detection, dependency graphs, plan sync",
  "repository": "https://github.com/smileynet/tkt",
  "license": "MIT",
  "keywords": ["tickets", "planning", "dependencies", "frontier"]
}
```

### 3. Add `SKILL_MANIFEST.yaml`

```yaml
name: tkt
version: "0.1.0"
compatibility:
  crew_research: "~> 0.9"
binary:
  name: tkt
  version_cmd: "tkt --version"
  min_version: "0.1.0"
skills:
  - name: tkt
    path: skills/tkt
deploy:
  method: symlink
  auto: true
  script: "tools/deploy-skills.sh"
```

### 4. Add `tools/deploy-skills.sh`

Follow archwright's pattern. Simpler than archwright (one skill, no steering, no extras).

## Acceptance criteria

- [x] `skills/tkt/SKILL.md` with CLI usage documentation
- [x] `plugin.json` passes Agent Plugins JSON Schema
- [x] `SKILL_MANIFEST.yaml` with version matching Cargo.toml
- [x] `tools/deploy-skills.sh` deploys to kiro/claude/codex paths
- [x] Deployed skill activates when user asks about ticket management
- [x] `tkt --version` output matches manifest version

## Out of scope

- Moving `ticket-planning` skill from crew-research (stays there — it's methodology)
- Updating crew-research's known-tools.yaml for tkt (that's crew-research ticket 98)

## Resolution (2026-08-13)

Shipped: agentskills.io format SKILL.md, plugin.json, deploy script. Deployed to 3 harnesses.

### Verification
1. ✓ skills/tkt/SKILL.md with CLI usage documentation — "skills/tkt/SKILL.md created with triggers, tools, workflows"
2. ✓ plugin.json at repo root — "plugin.json created with schema reference and skill path"
3. ✓ tools/deploy-skills.sh deploys to kiro/claude/codex — "deploy-skills.sh: 3 deployed, 0 skipped (verified with --dry-run and live)"
4. ✓ Deployed skill readable from harness paths — "ls -la ~/.kiro/skills/tkt confirms symlink to source"
5. ✓ tkt --version output matches plugin.json version — "tkt --version = 0.1.0, plugin.json version = 0.1.0"

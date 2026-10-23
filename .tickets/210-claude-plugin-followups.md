---
id: "210"
title: "Claude plugin follow-ups: SessionStart frontier hook; deploy-skills skips ~/.claude/skills when the plugin is installed"
status: open
blocked_by: ["209"]
priority: low
validation_criteria:
  - "a guarded SessionStart hook (only when .tickets/ exists and tkt is on PATH) adds 'tkt ready' output to context and passes claude plugin validate"
  - "deploy-skills.sh skips ~/.claude/skills when the tkt Claude plugin is installed, printing a note; never deletes a foreign dir"
tags: ["agent-integration"]
---

# Claude plugin follow-ups: SessionStart frontier hook; deploy-skills skips ~/.claude/skills when the plugin is installed

## What to build

TBD

## Acceptance criteria

- [ ] TBD

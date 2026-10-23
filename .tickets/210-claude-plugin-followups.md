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

Follow-ups to 209, the tkt Claude Code plugin.

- **SessionStart hook.** At session start, put `tkt ready` (and in-progress tickets) in front
  of the agent, so a session begins from the frontier without waiting for `tkt-orient` to be
  invoked. The plugin is installed for the user, so it runs everywhere: guard it to do nothing
  unless `.tickets/` exists in `$CLAUDE_PROJECT_DIR` and `tkt` is on PATH. This moved here from
  grey-matter proposal 07, because tkt owns the frontier.
- **No double listing.** `tools/deploy-skills.sh` still copies the tkt-* skills into
  `~/.claude/skills/`. With the plugin installed as well, each skill shows twice (`tkt-core` and
  `tkt:tkt-core`). Skip that root when the plugin is installed. grey-matter ticket 62 is the
  same fix for its own installer.

## Acceptance criteria

- [ ] a guarded SessionStart hook (only when .tickets/ exists and tkt is on PATH) adds 'tkt ready' output to context and passes claude plugin validate
- [ ] deploy-skills.sh skips ~/.claude/skills when the tkt Claude plugin is installed, printing a note; never deletes a foreign dir

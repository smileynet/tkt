---
id: "210"
title: "Claude plugin follow-ups: SessionStart frontier hook; deploy-skills skips ~/.claude/skills when the plugin is installed"
status: done
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

- [x] a guarded SessionStart hook (only when .tickets/ exists and tkt is on PATH) adds 'tkt ready' output to context and passes claude plugin validate
- [x] deploy-skills.sh skips ~/.claude/skills when the tkt Claude plugin is installed, printing a note; never deletes a foreign dir

## Resolution (2026-10-24)

done: guarded Claude SessionStart frontier/WIP context, private host packaging, native validation and Windows/Linux fixtures; existing ownership-safe deploy guard retained

### Verification
1. ✓ a guarded SessionStart hook (only when .tickets/ exists and tkt is on PATH) adds 'tkt ready' output to context and passes claude plugin validate — "1: source 72afcc9 adds guarded Node hook and private Claude config; source 6bb7751 fixes the native test shim; CI run 40058339724 passes four hook fixtures on Windows/Linux, 18 Node checks and 296 Rust tests; native source bundles pass all three hosts and Claude inventory reports one SessionStart hook; live launch-project context has WIP once"
2. ✓ deploy-skills.sh skips ~/.claude/skills when the tkt Claude plugin is installed, printing a note; never deletes a foreign dir — "2: canonical ownership-safe deploy guard remains byte-identical to grey-matter engine 1.5.0; hosted deploy regression tests and final corrected portfolio deployment pass; strict portfolio audit has zero findings/errors; foreign user directories retained"

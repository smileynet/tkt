---
id: "295"
title: "Add Qwen conditional rules/tickets.md, shared PreToolUse status:done guard, and <=500-char skill descriptions"
status: done
blocked_by: []
validation_criteria:
  - "rules/tickets.md exists with paths frontmatter for .tickets/**/*.md"
  - "PreToolUse hook blocks hand-editing status: done in .tickets/*.md across Claude, Qwen, and ZCode with anchored matchers (unit-tested)"
  - "tkt-reconcile and tkt-work SKILL.md descriptions are <= 500 chars with triggers front-loaded"
tags: ["stream:tooling", "kind:enhancement"]
---

# Add Qwen conditional rules/tickets.md, shared PreToolUse status:done guard, and <=500-char skill descriptions

## What to build

Harden `tkt`'s Qwen Code, Claude Code, and ZCode plugin surfaces based on the 2026-11-25 harness review:

1. Add conditional Qwen extension rule `rules/tickets.md` with `paths: [".tickets/**/*.md"]` frontmatter so Qwen loads the `never hand-edit status: done` rule at zero resident token cost.
2. Add `.claude-plugin/hooks/pre-tool-use.mjs` and register it under `PreToolUse` in `.claude-plugin/claude-hooks.json` with an anchored regex matcher (`^Edit$|^Write$|^edit$|^write_file$`) and single shell-string command (no `args`), emitting Claude/Qwen `hookSpecificOutput` JSON or ZCode top-level JSON when a direct edit tries to write `status: done` into `.tickets/*.md`. Add it to `.claude-plugin/plugin.json` `metadata.deploy.claudeFiles` and add regression tests in `tools/claude-hooks.test.mjs`.
3. Trim `skills/tkt-reconcile/SKILL.md` and `skills/tkt-work/SKILL.md` descriptions to <= 500 chars (surviving DSH's 500-char catalog truncation) with triggers front-loaded inside the first 140 chars.

## Acceptance criteria

- [x] `rules/tickets.md` exists with `paths: [".tickets/**/*.md"]` frontmatter.
- [x] `PreToolUse` hook blocks direct `status: done` edits in `.tickets/*.md` across Claude, Qwen, and ZCode with anchored matchers (`node --test tools/claude-hooks.test.mjs` passes).
- [x] `tkt-reconcile` and `tkt-work` `SKILL.md` descriptions are <= 500 chars with triggers front-loaded.

## Resolution (2026-11-25)

done: shipped Qwen conditional rules/tickets.md, shared PreToolUse status:done guard across Claude/Qwen/ZCode, and <=500-char skill descriptions

### Verification
1. ✓ rules/tickets.md exists with paths frontmatter for .tickets/**/*.md — "Added rules/tickets.md with paths: ['.tickets/**/*.md'] frontmatter for zero-resident-token Qwen enforcement"
2. ✓ PreToolUse hook blocks hand-editing status: done in .tickets/*.md across Claude, Qwen, and ZCode with anchored matchers (unit-tested) — "Added .claude-plugin/hooks/pre-tool-use.mjs, anchored matcher ^Edit$|^Write$|^edit$|^write_file$ in claude-hooks.json, and plugin.json claudeFiles entry; node --test tools/claude-hooks.test.mjs (10/10) and node tools/deploy.mjs check passed"
3. ✓ tkt-reconcile and tkt-work SKILL.md descriptions are <= 500 chars with triggers front-loaded — "Trimmed skills/tkt-reconcile/SKILL.md (392 chars) and skills/tkt-work/SKILL.md (395 chars) with triggers front-loaded; mise run check incl. check-skill-syntax passed"

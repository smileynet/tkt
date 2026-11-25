---
id: "295"
title: "Add Qwen conditional rules/tickets.md, shared PreToolUse status:done guard, and <=500-char skill descriptions"
status: open
blocked_by: []
validation_criteria:
  - "rules/tickets.md exists with paths frontmatter for .tickets/**/*.md"
  - "PreToolUse hook blocks hand-editing status: done in .tickets/*.md across Claude, Qwen, and ZCode with anchored matchers (unit-tested)"
  - "tkt-reconcile and tkt-work SKILL.md descriptions are <= 500 chars with triggers front-loaded"
tags: ["stream:tooling", "kind:enhancement"]
---

# Add Qwen conditional rules/tickets.md, shared PreToolUse status:done guard, and <=500-char skill descriptions

## What to build

Describe the change, why it is needed, and link its source (--spec or a #NN reference).

## Acceptance criteria

- [ ] Define an observable result and how to verify it.

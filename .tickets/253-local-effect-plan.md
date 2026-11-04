---
id: "253"
title: "Plan and apply local configuration context and init effects"
status: in_progress
blocked_by: []
priority: medium
validation_criteria:
  - "Concrete filesystem plan preserves explicit preview/apply semantics without opaque callbacks"
  - "Configuration context and init route persistence through atomic local effects with failure and no-write controls"
  - "Full local gate and independent critic review pass; parent all-mutator acceptance remains open"
tags: ["stream:quality", "kind:enhancement", "triage:ready-for-agent"]
parent: "191"
assignee: "codex"
---
# Plan local filesystem effects before applying them

## Intent source

Continue parent #191 and its [source audit](../docs/mutation-effects-audit.md).
The earlier children #250/#251 repaired diagnostics and migration publication;
configuration/context/init still own direct persistence and directory creation.

## What to build

Introduce one small concrete internal local-filesystem effect plan with explicit
preview/apply execution. Represent prepared text replacement, no-clobber creation,
directory creation and file removal as domain operations, not arbitrary callbacks.
Route configuration persistence, context state and init file writers through it.
Reuse core atomic replacement/publication, preserve contents and CLI output, and
keep filesystem errors actionable. A later effect failure must not erase already
published or foreign data; no batch-transaction or external-editor-lock guarantee.

Keep MutationContext and GitTransaction responsibilities intact. This child does
not complete every-mutator routing, Git publication or the full matrix in #191.
No additional dependency, hook activation, user consent change or release.

## Acceptance criteria

- [ ] Local effects are prepared as concrete data and Preview performs no filesystem effects.
- [ ] Config/context/init persistence uses the plan and atomic writes; foreign new-file arrivals are preserved.
- [ ] Tests cover preview/apply, replacement failure and preserved bytes, and real CLI no-write and normal-write controls.
- [ ] Full local gate and independent critic review accept final source before closure; parent #191 stays in progress.

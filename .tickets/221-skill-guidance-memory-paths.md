---
id: "221"
title: "Shipped tkt-core guidance prescribes .memory/adr and .memory/specs, contradicting grey-matter's no-.memory convention"
status: in_progress
blocked_by: []
validation_criteria:
  - "Each point below marked confirmed / rejected / re-scoped with evidence"
  - "If changed: tkt-core references cite convention-neutral or grey-matter paths, and the skill pack is redeployed"
tags: ["agent-integration"]
---

# Shipped tkt-core guidance prescribes .memory/adr and .memory/specs, contradicting grey-matter's no-.memory convention

> **Filed by grey-matter adoption review (2026-10-26). Confirmation status: unconfirmed.**
> Reproduce each point and mark it confirmed / rejected / re-scoped before changing anything.
> Created locally; a corporate push-policy hook blocks pushes to this remote from this host, so check for an
> id collision (`tkt renumber`) before pushing from elsewhere.

## Observed (c242a04)

1. `skills/tkt-core/references/ticket-standards.md:27` gives the ADR reference example
   "Decision in .memory/adr/NNNN-title.md".
2. Line 42 lists `.memory/specs/` first among durable citation paths.
3. grey-matter (tkt's declared skill format since 2c724a5) says never create `.memory/`;
   adopted repos keep ADRs in `docs/adr/` and specs in `docs/agents/specs/`.
4. This is shipped guidance: every tkt-only adopter reads it, so a change is user-facing.

## Verification (2026-10-28)

1. Confirmed: the shipped reference contained the exact `.memory/adr/` example.
2. Confirmed: its durable-reference rule named `.memory/specs/` twice.
3. Re-scoped: installed init-project guidance names GLOSSARY.md and docs/adr/;
   this repository still legitimately uses its existing .memory layout pending #222.
   Standalone guidance should follow each adopter's established directories.
4. Confirmed: the affected file is tracked under skills/tkt-core/references and
   shipped through the plugin's skills directory.

## What to build

Suggestion only: make the examples convention-neutral ("the project's ADR directory, e.g.
`docs/adr/`") rather than prescribing either layout; redeploy the pack.

## Acceptance criteria

- [ ] Each observed point marked confirmed / rejected / re-scoped
- [ ] If changed: tkt-core references cite convention-neutral or grey-matter paths, and the pack is redeployed

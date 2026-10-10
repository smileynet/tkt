---
id: "187"
title: "Generate a derived sync-plan projection from the ticket corpus"
status: backlog
blocked_by: []
priority: medium
validation_criteria:
  - "A projection command renders every relevant ticket from the corpus with deterministic ordering and derived status"
  - "Generated output is written atomically and respects global dry-run"
  - "The command preserves or clearly separates any non-derivable human narrative"
tags: ["planning", "sync-plan"]
---

# Generate a derived sync-plan projection from the ticket corpus

## Context

ADR 0002 makes plan-as-projection the strategic primary direction for `sync-plan`:
ticket files remain the source of truth, while derivable plan content is rendered from
the corpus. #184 deliberately implements the secondary, opt-in narrative format and
explicitly excludes this work. Without a dedicated ticket, the ADR's primary direction
has no owner or schedulable scope.

## What to build

Design and implement a command or safe `sync-plan` mode that renders a deterministic
table projection from the ticket corpus. Define where generated content lives and how
human-authored, non-derivable narrative is preserved or separated. The generated path
must be safe to preview, atomic to write, and must not make a hand-maintained plan a
second source of truth.

This is intentionally separate from #184: projection is the primary derived-plan path;
#184 is optional parsing of a human-authored narrative plan.

## Acceptance criteria

- [ ] A projection command/mode renders all relevant tickets in deterministic priority/ID order with derived status
- [ ] The generated output is atomically written and global `--dry-run` previews without writing
- [ ] The location and behavior of human-authored narrative are explicitly preserved or separated
- [ ] Table-format `sync-plan` compatibility and #184's opt-in narrative design remain intact
- [ ] Help, README, and agent guidance explain ticket files as the source of truth

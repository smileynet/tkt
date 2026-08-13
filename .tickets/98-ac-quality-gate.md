---
id: "98"
title: "Enforce acceptance criteria checked before closure across all repos"
status: in_progress
blocked_by: []
priority: high
validation_criteria: 
  - "tkt validate 0 errors 0 warnings across all repos"
  - "config.toml with enforcement in all repos"
  - "steering updated with evidence requirement"
---

# Enforce acceptance criteria checked before closure across all repos

## Problem

197 tickets across 9 repos were closed with unchecked acceptance criteria boxes. This is not cosmetic — unchecked ACs mean work was marked done without verifying it met its own stated criteria. The quality gate exists (`close.require_checked_acs` in project config, `--check-all`/`--ac`/`--force` flags) but was not enforced historically.

Current state (2026-08-13):
- tkt: 36 tickets
- godot-helper: 63 tickets
- archwright: 30 tickets
- recall: 20 tickets
- teach-me: 16 tickets
- crew-research: 13 tickets
- gdhelper-log: 11 tickets (spikes — intentional?)
- lacrosse-bosse-helper: 7 tickets
- gdhelper-cli: 1 ticket

## What to build

The gate already defaults on (`close_require_checked_acs: true` in code, configurable off via `.tickets/config.toml`). No code change needed — the problem is purely retroactive debt.

1. **Retroactive audit**: Review each repo's unchecked-AC tickets. Categorize:
   - Work actually completed but boxes never checked → check the boxes
   - Work partially done but ticket closed anyway → reopen or document gap
   - Spikes/research where ACs were aspirational → mark with `--force` note or remove ACs

2. **Steering update**: Update frontier-work steering to make the AC gate explicit in agent instructions. Close commands in AGENTS.md examples should always include `--check-all` or explicit `--ac` indices.

## Acceptance criteria

- [ ] Each repo's unchecked-AC tickets audited and categorized
- [ ] Boxes checked for tickets where work was genuinely complete
- [ ] Reopened or annotated tickets where work was incomplete
- [ ] `tkt validate --strict` passes in every repo after cleanup
- [ ] Steering/AGENTS.md updated to require AC verification before closure

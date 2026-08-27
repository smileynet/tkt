---
id: "154"
title: "Detect hand-flipped done tickets in validate + guide against status editing"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "validate surfaces missing-resolution and all-acs-unchecked-on-done findings (warning default, error under --strict)"
  - "frontier-work steering and tkt skill explicitly forbid hand-editing status to done, directing tkt close instead"
tags: ["contract"]
---

# Detect hand-flipped done tickets in validate + guide against status editing

## What to build

Agents sometimes mark work done by editing the ticket file (`status: open` → `status: done`) instead of running `tkt close`. This bypasses every close gate — AC checks, resolution text, validation evidence — and the atomic commit/push claim protocol. The frontmatter says done, but the contract was never enforced. Observed 2026-08-27: an agent hand-flipped #209/#210 to done via file rewrite.

Two complementary fixes:

**Fix 1 — Detection in `validate` (mechanical enforcement):**
The closure-quality checks already exist in `src/audit.rs` (`check_resolution_quality`, `check_ac_completeness`) but only run under `tkt audit`, which is rarely invoked. A hand-flipped done ticket has no `## Resolution` section (that's what `tkt close` appends), so `check_resolution_quality` would flag it as `missing-resolution` — but `validate` doesn't run these checks. Fold them into `validate`:
- warning severity by default (visible, non-blocking)
- error severity under `--strict` (fails CI)
- consistent with how validate treats other decay findings

**Fix 2 — Guidance (behavior nudge):**
Add an explicit rule to `steering/frontier-work.md` and the tkt skill: never set `status: done` by editing the file — always `tkt close <id>`. Hand-flipping skips AC/resolution/evidence gates and the push protocol. The #209/#210 agent knew the work was done; it just didn't know flipping status directly is wrong.

## Context

- **Relevant files:** `src/audit.rs` (check_resolution_quality, check_ac_completeness — reuse), `src/commands/validate.rs` (add the checks), `steering/frontier-work.md`, `skills/tkt/SKILL.md`
- **Do NOT** have `validate --fix` auto-append a resolution — that fabricates closure evidence. Flag only; require the agent to re-close via `tkt close <id> --force --resolution "..."`.
- **Guidance principle:** a rule violated despite existing (audit had the check, agent still hand-flipped) → promote to the routinely-run gate (validate)

## Acceptance criteria

- [ ] validate surfaces missing-resolution and all-acs-unchecked-on-done findings (warning default, error under --strict)
- [ ] validate --fix does NOT fabricate resolutions — flags only
- [ ] frontier-work steering forbids hand-editing status to done, directs tkt close
- [ ] tkt skill carries the same rule
- [ ] tests cover a hand-flipped done ticket (no Resolution) being flagged by validate
- [ ] existing validate/audit tests pass

## Out of scope

- Hard prevention via hooks (that's #155 — tkt can't stop a text editor)
- Changing audit's existing behavior

---
id: "54"
title: "Scope --check-all to acceptance criteria section (F4)"
status: done
blocked_by: []
priority: high
---

# Scope --check-all to acceptance criteria section (F4)

## Origin

Review ticket #38, finding F4.

## Problem

`tkt close --check-all` does `file.body.replace("- [ ]", "- [x]")` across the entire file body. A ticket with a design checklist, task list, or other non-AC checkboxes gets those checked too, and the AC count is inflated.

Reproduced: a ticket with 2 design checkboxes + 1 AC box reports `3/3 checked ✓` after `--check-all`.

## What to build

1. Scope `--check-all` to only check boxes in the `## Acceptance criteria` section
2. While at it, scope AC counting (used by close, validate, and audit) to the same section via a shared helper
3. Boxes outside the AC section are never touched or counted

### Implementation approach

Create a helper that identifies the AC section boundaries:
- Starts at `## Acceptance criteria` heading
- Ends at the next `## ` heading or EOF
- Only count/replace within those bounds

## Acceptance criteria

- [x] `--check-all` only checks boxes under `## Acceptance criteria`
- [x] Boxes in other sections (design checklists, task lists) are untouched
- [x] AC count in close output reflects only the AC section
- [x] `validate` and `audit` AC counting also scoped to the section
- [x] Unit test: ticket with mixed sections, verify only AC boxes affected
- [ ] Integration test: close --check-all with non-AC checkboxes, verify they're preserved

## Resolution (2026-08-05)

Added `core::ac_section_range()` helper shared by cli.rs and findings.rs. `--check-all` replaces only within the AC section range. `count_ac_boxes`, `flip_ac_boxes`, `cmd_audit`, and `check_unchecked_acs` all scope to the same boundaries. Verified in scratch repo: design checklist boxes untouched, AC count reports 2/2 not 4/4. Integration test deferred to #41.

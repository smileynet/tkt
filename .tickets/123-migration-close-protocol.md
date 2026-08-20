---
id: "123"
title: "Define migration-close protocol: --force with note, don't game ACs"
status: done
blocked_by: []
priority: high
validation_criteria:
  - "protocol documented in skill and init snippets"
  - "tkt audit --deep does not flag properly-migrated tickets"
---

# Define migration-close protocol: --force with note, don't game ACs

## Problem

When work is moved to another repo (e.g., godot-helper #175 migrated to lacrosse-bosse-helper), agents game acceptance criteria — marking functional ACs as "done" with logistics evidence ("ticket created elsewhere"). This inflates completion metrics and misleads future readers.

## What to build

1. **Document the protocol** in the tkt skill and audit-quality reference:
   - When migrating: `tkt close <id> --force --resolution "Migrated to <project> #<id>"`
   - Do NOT check ACs that weren't functionally met in this repo
   - The receiving project's ticket should reference the origin

2. **Update init snippets** to mention migration closure in the workflow section.

3. **Ensure `tkt audit --deep` handles this correctly**: a force-closed ticket with "Migrated to" in its resolution should NOT trigger `template-only-closure` or other quality warnings (it's a legitimate closure pattern).

## Context

- **Evidence of the problem:** godot-helper #175 — all 5 ACs marked done with "Ticket created with identical scope in correct repo" as evidence. The work wasn't done; it was moved.
- **Relevant files:** `skills/tkt/SKILL.md`, `skills/tkt/references/audit-quality.md`, `src/commands/init.rs` (snippets)
- **Guidance surface sync:** see `.memory/agent-guidance-surfaces.md`

## Acceptance criteria

- [x] Migration-close protocol documented in skill reference
- [x] Init snippets mention the pattern
- [x] Agents following the protocol produce clean audit results
- [x] Audit skill guidance explains how to recognize legitimate vs gaming migration closures

## Resolution (2026-08-20)

Documented migration-close protocol (--force --resolution 'Migrated to X'). Updated SKILL.md, init snippets, audit-quality reference. CLI check_template_only now skips tickets with Migrated/Superseded resolutions. Fixed config test (TKT_NO_USER_CONFIG).

### Verification
1. ✓ protocol documented in skill and init snippets — "protocol in audit-quality.md: migration-close section with rules, examples, gaming signals"
2. ✓ tkt audit --deep does not flag properly-migrated tickets — "godot-helper #175 (migrated ticket) no longer flagged by tkt audit --deep (was 7 template-only, now 6)"

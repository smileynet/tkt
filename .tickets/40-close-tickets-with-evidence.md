---
id: "40"
title: "Close tickets #30-34 properly with evidence (F2)"
status: open
blocked_by: []
priority: high
---

# Close tickets #30-34 properly with evidence (F2)

## Origin

Review ticket #38, finding F2.

## Problem

Tickets #30, #31, #32, #33, #34 are `status: done` but have zero AC boxes checked and no `## Resolution` section. They were closed by hand-editing the status field inside feature commits, bypassing `tkt close` and its guards.

## What to build

For each ticket (#30–#34, plus #28 which has the same issue):
1. Verify which ACs are genuinely met by the implementation at HEAD
2. Check the boxes that are met with a brief evidence note
3. Add a `## Resolution` section describing what was delivered
4. For any AC that is NOT met, leave it unchecked (F3 covers adding missing tests)

## Acceptance criteria

- [ ] #30 has checked ACs and a Resolution section
- [ ] #31 has checked ACs and a Resolution section
- [ ] #32 has checked ACs and a Resolution section
- [ ] #33 has checked ACs and a Resolution section
- [ ] #34 has checked ACs and a Resolution section
- [ ] #28 has checked ACs and a Resolution section
- [ ] `tkt audit --brief` no longer reports `unchecked-acs-on-done` for these tickets
- [ ] `tkt audit --brief` no longer reports `missing-resolution` for these tickets

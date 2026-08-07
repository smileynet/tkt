---
id: "59"
title: "Resolve color/symbol spec vs implementation (F9)"
status: open
blocked_by: []
---

# Resolve color/symbol spec vs implementation (F9)

## Origin

Review ticket #38, finding F9.

## Problem

Ticket #31 specifies:
- "✓ (green) for success, ✗ (red) for domain errors, ⚠ (yellow) for warnings"
- "Respect NO_COLOR=1 — degrade to plain ✓/✗/⚠ without ANSI codes"

Implementation emits plain UTF-8 glyphs with no color. The `NO_COLOR` AC passes vacuously. Additionally, ✓/✗/⚠/→ are non-ASCII — legacy Windows consoles may mangle them.

## What to build

Decide one of:
1. **Implement color**: add ANSI codes when stdout is a tty and `NO_COLOR` is unset; degrade to plain glyphs otherwise
2. **Drop the color requirement**: amend #31 to state that plain Unicode glyphs are the intended behavior, record an ADR, and optionally add ASCII fallback for `NO_COLOR`

Either way, document the decision.

Related: the error prefix changed from `tkt: <msg>` to `✗ <msg>`, dropping program identification from stderr. Consider keeping `tkt:` prefix for pipeline diagnostics.

## Acceptance criteria

- [ ] Decision documented (implement color OR drop requirement)
- [ ] If color: ANSI codes on tty, plain on NO_COLOR/non-tty
- [ ] If no color: #31 AC3 reworded or marked N/A
- [ ] Error output includes program name for pipeline diagnostics
- [ ] Behavior consistent across commands

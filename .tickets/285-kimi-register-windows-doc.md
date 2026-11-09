---
id: "285"
title: "Document Windows register_personal.py workaround in plugin-maintenance.md"
status: done
blocked_by: []
validation_criteria:
  - "plugin-maintenance.md shows Python variant as the Windows path"
  - "literal error text and date included"
  - "docs-only change"
tags: ["type:chore"]
---
## What to build

Make the Kimi plugin registration instructions in
`docs/agents/plugin-maintenance.md` platform-aware: on Windows,
`register_personal.sh` aborts (`.cmd` launcher fails the script's
executability test under Git Bash); use `python scripts/register_personal.py
<plugin-dir>` instead. Mirror of grey-matter ticket for distribution-maintenance.md;
the failure and workaround are already recorded in grey-matter's harnesses.md
(ticket 283, 2026-11-08).

## Acceptance criteria

- [x] plugin-maintenance.md Kimi section shows the Python variant as the Windows path
- [x] Literal error text and observed date are included
- [x] Docs-only change; no behavior change

## Resolution (2026-11-09)

done: plugin-maintenance.md Kimi registration block now shows the Python variant as the Windows path, with the literal .cmd executability error and observed date

### Verification
1. ✓ plugin-maintenance.md shows Python variant as the Windows path — "command block shows register_personal.py labeled Windows alongside the bash variant"
2. ✓ literal error text and date included — "literal error text and 2026-11-08 observation date included in the prose below the block"
3. ✓ docs-only change — "docs-only diff: one fenced block extended, one prose paragraph added"

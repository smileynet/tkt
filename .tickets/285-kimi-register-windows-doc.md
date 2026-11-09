---
id: "285"
title: "Document Windows register_personal.py workaround in plugin-maintenance.md"
status: open
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

- [ ] plugin-maintenance.md Kimi section shows the Python variant as the Windows path
- [ ] Literal error text and observed date are included
- [ ] Docs-only change; no behavior change

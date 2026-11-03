---
id: "250"
title: "Prevent debug logging from writing during dry-run"
status: done
blocked_by: []
priority: high
validation_criteria:
  - "Dry-run leaves absent and preexisting isolated diagnostic state byte-for-byte unchanged"
  - "Ordinary debug log and explicit human/JSON stderr behavior remain verified without user consent/config changes"
  - "Fresh public regression, project verification gate and independent critic review pass"
tags: ["stream:quality", "kind:bug", "triage:ready-for-agent"]
parent: "191"
assignee: "codex"
---
# Dry-run diagnostics must not persist state

## Source and reproduction

Parent: #191. Independent reproduction on the installed d197af2 binary:
`TKT_DEBUG=log tkt --dry-run config --set debug=true` with private XDG state/config
roots and DO_NOT_TRACK=1 exits0, creates state/tkt/debug.log, and does not create
the requested config. Debug setup emits before the global dry-run flag is stored;
the persistent debug sink has no preview guard.

## What to build

Publish the parsed preview mode before diagnostics and suppress persistent debug
log effects during previews. Preserve ordinary file logging and explicit human/
JSON stderr debug formats. Do not enable/read real user telemetry or consent.
Prove both absent and preexisting isolated log/state trees remain unchanged.

## Acceptance criteria

- [x] Preview diagnostics create no files/directories and do not append existing logs.
- [x] Ordinary file logging and human/JSON stderr modes retain their contracts.
- [x] Public child-process regression uses private roots and a positive ordinary-run control.

## Scope

This repairs a demonstrated effect leak; it does not complete the parent's
all-mutator effect-plan consolidation. No actual user configuration is changed.

## Resolution (2026-11-03)

Fixed demonstrated file-debug effects during global/local dry-run; red/green public controls, full gate and independent critic accepted. Parent191 remains in progress.

### Verification
1. ✓ Dry-run leaves absent and preexisting isolated diagnostic state byte-for-byte unchanged — "The private-root reproduction created a log during preview. The public regression went red then green; six commands preserve absent/preexisting logs and private config. The full gate passed 421 Rust tests and 42 Node fixtures, with one existing Windows skip."
2. ✓ Ordinary debug log and explicit human/JSON stderr behavior remain verified without user consent/config changes — "Ordinary file logging and human/JSON stderr controls pass. Every child uses DO_NOT_TRACK=1 and private XDG roots; actual user consent/configuration is unchanged."
3. ✓ Fresh public regression, project verification gate and independent critic review pass — "Independent critic review accepted early preview initialization and the persistent sink guard. Full gate output, scope and diff checks passed. Parent 191 remains incomplete. See docs/evidence/dry-run-diagnostics-250.md."

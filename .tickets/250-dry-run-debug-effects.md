---
id: "250"
title: "Prevent debug logging from writing during dry-run"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "Dry-run leaves absent and preexisting isolated diagnostic state byte-for-byte unchanged"
  - "Ordinary debug log and explicit human/JSON stderr behavior remain verified without user consent/config changes"
  - "Fresh public regression, project verification gate and independent critic review pass"
tags: ["stream:quality", "kind:bug", "triage:ready-for-agent"]
parent: "191"
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

- [ ] Preview diagnostics create no files/directories and do not append existing logs.
- [ ] Ordinary file logging and human/JSON stderr modes retain their contracts.
- [ ] Public child-process regression uses private roots and a positive ordinary-run control.

## Scope

This repairs a demonstrated effect leak; it does not complete the parent's
all-mutator effect-plan consolidation. No actual user configuration is changed.

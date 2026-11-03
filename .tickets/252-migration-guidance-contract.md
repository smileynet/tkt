---
id: "252"
title: "Remove unsupported migration-map guidance"
status: in_progress
blocked_by: []
priority: high
validation_criteria:
  - "Executable reference examples match current capabilities and help; unsupported map flags are not taught as runnable"
  - "Supported tk migration, source preservation and recovery are separated from proposed adapters, with accurate assignee/eligibility semantics"
  - "Standalone guidance syntax, deployment fixtures and independent critic review pass"
tags: ["stream:guidance", "kind:bug", "triage:ready-for-agent"]
assignee: "codex"
---
# Keep migration guidance within the shipped interface

Reporter: Codex
Confirmation status: confirmed
Related: #125 and #251

## Evidence

The distributed migration-assist reference tells adopters to generate a
migrate.toml and run tkt migrate --map ... --apply, then later says that interface
is unimplemented. A private DO_NOT_TRACK=1 probe of installed tkt0.4.0(d197af2)
rejects --map with exit2. It also says tkt does not track assignees, contrary to the
current typed invocation ownership contract.

## What to build

Rewrite this standalone reference around observed capabilities and help. Teach
only supported tk detection/preview/confirm operations. Unsupported adapters and
custom maps may be described as proposed designs, not executable contracts.
Preserve source evidence/unknown fields explicitly and distinguish external people
from invocation assignees and machine/agent eligibility. Align recovery guidance
with the completed publication repair; require no repository checkout or foreign
workflow skills for a tkt-only adopter.

## Acceptance criteria

- [ ] All executable command examples are supported by current capabilities/help.
- [ ] Generic maps and external adapters are clearly unimplemented; no fake flags or nonexistent ownership claims remain.
- [ ] Source preservation, preview/confirmation and recovery limits are accurate and standalone.
- [ ] Applicable guidance checks and independent critic review pass before closure.

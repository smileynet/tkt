---
id: "267"
title: "Register and validate the tkt plugin in Kimi"
status: done
blocked_by: ["266"]
validation_criteria:
  - "The validated tkt package is registered in the personal market and its ID, version, and source path read back successfully."
  - "After owner installation, live probes verify tkt-core discovery and model invocation, linked-resource loading, and a read-only ticket query."
  - "Grey Matter ticket 271 closes with evidence after tkt-core becomes available, and remaining update or platform checks are reported honestly."
tags: ["tooling"]
---
## What to build

Register the maintained tkt Kimi package through the official personal-market workflow, then ask the owner to install it. Probe actual names, model discovery and invocation, linked resources, and a read-only tkt query. Record limitations and update behavior rather than inferring them. Once tkt-core is available, close Grey Matter ticket 271 with its previously gathered evidence.

Intent: prove the installed plugin supplies the prerequisite Grey Matter needs and completes the Kimi workflow integration approved on 2026-11-08.

## Acceptance criteria

- [x] Rights scanning and native validation finish before registration; read-back verification passes.
- [x] The owner installs the plugin and live skill/resource/query probes pass.
- [x] Documentation reflects observed runtime names and behavior, with update limitations stated.
- [x] Grey Matter ticket 271 closes through tkt with one evidence item per validation criterion.

## Resolution (2026-11-08)

Registered the maintained tkt package, completed owner-installed live probes, documented observed runtime behavior and limitations, and closed Grey Matter ticket 271. Update timing and non-macOS installation remain unverified.

### Verification
1. ✓ The validated tkt package is registered in the personal market and its ID, version, and source path read back successfully. — "Rights scan clear; native validation 0 errors/0 warnings; registration read back tkt 0.4.0, source path, and four skills before installation. Link: kimi-work://plugin?id=tkt."
2. ✓ After owner installation, live probes verify tkt-core discovery and model invocation, linked-resource loading, and a read-only ticket query. — "Owner installation completed; model invocation loaded tkt:tkt-core, installed manifest/SKILL.md/references/commands.md matched package bytes, and tkt query --all-context --status in_progress -o json succeeded."
3. ✓ Grey Matter ticket 271 closes with evidence after tkt-core becomes available, and remaining update or platform checks are reported honestly. — "Grey Matter 271 closed through tkt with four criterion-matched evidence items. plugin-maintenance.md records runtime names and evidence in commit 3889d10; update timing and non-macOS installation are explicitly unverified."

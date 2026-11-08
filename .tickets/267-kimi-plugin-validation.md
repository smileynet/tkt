---
id: "267"
title: "Register and validate the tkt plugin in Kimi"
status: open
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

- [ ] Rights scanning and native validation finish before registration; read-back verification passes.
- [ ] The owner installs the plugin and live skill/resource/query probes pass.
- [ ] Documentation reflects observed runtime names and behavior, with update limitations stated.
- [ ] Grey Matter ticket 271 closes through tkt with one evidence item per validation criterion.
---
id: "244"
title: "Serve the delivery dashboard on the local network"
status: in_progress
blocked_by: []
validation_criteria:
  - "Live LAN-address HTTP response matches the current page; only explicit resources are served"
  - "HTTP regressions and lifecycle documentation are verified, with remote-device limits stated"
tags: ["dashboard", "kind:enhancement", "triage:ready-for-agent", "stream:quality"]
assignee: "codex"
---
# Serve the dashboard on the LAN

## Intent source

Owner request 2026-11-03: make the dashboard accessible via LAN.

## What to build

Run a read-only server bound to the current Ethernet IPv4 address. Serve the dashboard and explicit linked local resources without directory listings or unrelated repository files. Preserve live reload through fresh reads and document startup/stop behavior.

## Acceptance criteria

- [ ] A running scoped server responds over the LAN address with the current dashboard bytes.
- [ ] Public HTTP tests cover linked resources, rejected repository/traversal paths, method restrictions and updated content.
- [ ] Runtime lifecycle and firewall observations are documented without claiming a second-device or browser-rendering pass.

## Evidence

See docs/evidence/dashboard-lan-244.md. The supervised process responds at http://192.168.0.187:8765/docs/status.html. Native browser use remains disabled; hosting does not waive UI acceptance on the existing dashboard tickets.

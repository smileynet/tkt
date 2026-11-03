---
id: "244"
title: "Serve the delivery dashboard on the local network"
status: done
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

- [x] A running scoped server responds over the LAN address with the current dashboard bytes.
- [x] Public HTTP tests cover linked resources, rejected repository/traversal paths, method restrictions and updated content.
- [x] Runtime lifecycle and firewall observations are documented without claiming a second-device or browser-rendering pass.

## Evidence

See docs/evidence/dashboard-lan-244.md. The supervised process responds at http://192.168.0.187:8765/docs/status.html. Native browser use remains disabled; hosting does not waive UI acceptance on the existing dashboard tickets.

## Resolution (2026-11-03)

A scoped read-only Node server now serves the current dashboard and linked resources on the Ethernet LAN address. HTTP content/hash and denied-path checks pass; lifecycle, restart and clipboard fallback are documented. Source/tests/evidence promoted; runtime PID/session record retained while the service is needed, temporary publication helpers expire after the dashboard update.

### Verification
1. ✓ Live LAN-address HTTP response matches the current page; only explicit resources are served — "Live proxy-disabled HTTP GET on 192.168.0.187:8765 returns 200 and SHA256-identical current dashboard bytes; explicit resources only, unrelated paths404. Two HTTP tests pass including traversal and symlink guards. See docs/evidence/dashboard-lan-244.md."
2. ✓ HTTP regressions and lifecycle documentation are verified, with remote-device limits stated — "Nine applicable Node tests passed, no skips; hosting and lifecycle guide recorded. Existing private-network TCP rule observed, no firewall changes. Supervised process running; second-device routing and browser rendering remain unverified. Runtime state retained only until server stops."

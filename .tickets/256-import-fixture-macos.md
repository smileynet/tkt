---
id: "256"
title: "Make import failure fixtures deterministic on macOS"
status: open
blocked_by: []
validation_criteria:
  - "Git maintenance cannot race byte snapshots and missing-gh fixtures produce an execution error on macOS and Windows."
tags: ["stream:tooling"]
---
## What to build

During grey-matter sync verification, mise run check failed in tests/github_import.rs: automatic Git maintenance created transient lock files, and an empty executable was accepted by macOS as a successful shell script. Keep the fixture isolated and use an explicitly unlaunchable executable. Source: tests/github_import.rs.

## Acceptance criteria

- [ ] Import fixtures pass on macOS without host Git config changes or skipped assertions.

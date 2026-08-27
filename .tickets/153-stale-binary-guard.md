---
id: "153"
title: "Guard against stale-binary validation (prefer integration tests, add freshness check)"
status: backlog
blocked_by: []
priority: medium
validation_criteria:
  - "AGENTS.md directs validation via cargo test integration tests (freshly-built target binary), not PATH-installed tkt"
  - "tools/check-binary-fresh.sh exits 1 when installed tkt hash != HEAD, 0 when matched, with JSON status output"
tags: ["tooling"]
---

# Guard against stale-binary validation (prefer integration tests, add freshness check)

## What to build

TBD

## Acceptance criteria

- [ ] TBD

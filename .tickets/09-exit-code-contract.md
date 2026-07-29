---
id: "09"
title: "implement exit code 2 for crash-class failures"
status: done
blocked_by: ["07"]
---

# Implement exit code 2 for crash-class failures

## What to build

AGENTS.md specifies `0=success, 1=failure/drift, 2=crash` but `run()` maps every `anyhow::Error` to exit 1. Domain failures (validation drift, ticket not found, state conflict) should remain exit 1. Operational failures (I/O errors, git execution failures, parse errors, invalid UTF-8) should exit 2.

### Changes needed

1. Define an error classification enum or trait: `DomainFailure` vs `OperationalCrash`
2. In `run()`, match on error type/classification to choose exit 1 vs 2
3. Ensure Clap argument errors (already exit 2 by default) remain consistent
4. Document the contract in `--help` or a top-level doc comment

## Acceptance criteria

- [ ] Ticket parse error → exit 2
- [ ] Git subprocess cannot start → exit 2
- [ ] File I/O error → exit 2
- [ ] Ticket not found → exit 1
- [ ] Status conflict (claim non-open) → exit 1
- [ ] Validation drift detected → exit 1
- [ ] Push rejected after retries → exit 1
- [ ] Integration test verifies at least one exit-1 and one exit-2 case

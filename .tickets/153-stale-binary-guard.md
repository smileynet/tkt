---
id: "153"
title: "Guard against stale-binary validation (prefer integration tests, add freshness check)"
status: backlog
blocked_by: []
priority: high
validation_criteria:
  - "AGENTS.md directs validation via cargo test integration tests (freshly-built target binary), not PATH-installed tkt"
  - "tools/check-binary-fresh.sh exits 1 when installed tkt hash != HEAD, 0 when matched, with JSON status output"
tags: ["tooling"]
---

# Guard against stale-binary validation (prefer integration tests, add freshness check)

## What to build

The stale-binary trap bit twice (closing #131 and #132): after `cargo install --path .`, subsequent commits advance HEAD past the installed binary's baked-in `TKT_GIT_HASH`, so a later manual `tkt` run for "end-to-end validation" tests old code. The existing AGENTS.md prose constraint ("reinstall before testing") failed both times because it depends on remembering at a fuzzy moment (HEAD moves between install and test).

Root insight: integration tests already avoid this. `tkt_bin()` in `tests/integration.rs` resolves to `target/debug/tkt` — the binary `cargo test` rebuilds from current source before every run. It is never stale. The mistake was reaching for the PATH-installed binary for manual checks when the integration suite already exercises fresh code.

Two-part fix:

1. **Practice change (primary):** validation should go through `cargo test` integration tests (auto-rebuilt `target/` binary), not manual runs of the PATH-installed `tkt`. The PATH binary is for *using* tkt on real repos, not validating a fix. This eliminates the staleness class.

2. **Mechanical guard (secondary):** `tools/check-binary-fresh.sh` compares `tkt --version` hash to `git rev-parse --short HEAD`, exits 1 if stale with JSON status output. For the rare case where the PATH binary is genuinely under test (install/deploy path).

## Context

- **Relevant files:** `build.rs` (bakes TKT_GIT_HASH), `src/cli.rs` (version_string), `tests/integration.rs` (tkt_bin uses target/), `AGENTS.md` (the failing prose constraint at "After any code change...")
- **Incident evidence:** #131 closed against binary that still reproduced the bug; #132 resolution claim went stale when HEAD advanced past the installed binary
- **Guidance principle:** a correction that happened DESPITE a covering rule means the prose failed — promote to mechanical enforcement (guidance-sync P6)

## Acceptance criteria

- [ ] AGENTS.md directs validation via cargo test integration tests (freshly-built target binary), not PATH-installed tkt
- [ ] The old memory-dependent "reinstall before testing" constraint is superseded (not just appended to)
- [ ] tools/check-binary-fresh.sh exits 1 when installed tkt hash != HEAD, 0 when matched, with JSON status output
- [ ] Script follows the validation contract (JSON status, exit 0/1/2)

## Out of scope

- Blocking cargo install or adding a pre-commit hook (too heavy for the frequency)

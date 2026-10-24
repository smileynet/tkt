---
id: "216"
title: "Run identity verification on a cold CI cache"
status: in_progress
blocked_by: []
priority: high
validation_criteria:
  - "A clean hosted check job completes the offline identity fixture after fetching locked dependencies."
  - "The identity fixture reuses Cargo.lock dependency versions instead of selecting uncached newer crates."
tags: ["stream:tooling"]
---

# Run identity verification on a cold CI cache

## Intent source

Found while verifying release-audit ticket 215. At `acf27c5`, hosted run
[40054345414](https://github.com/smileynet/tkt/actions/runs/40054345414)
passed the exact Rust 1.85.0 minimum check but its full gate failed in the identity
fixture: `serde_json v1.0.154` was selected but unavailable under `--offline`.

## What to build

A clean CI runner can execute the offline source-identity fixture without relying
on an existing Cargo cache. Use the project lockfile's tested dependency versions
in the temporary fixture and fetch the locked dependencies before the full CI gate.
Keep the offline rebuild checks and production build identity behavior unchanged.

## Context

Read `tools/release-tools.test.mjs` (built identity fixture), `Cargo.lock`, and
`.github/workflows/ci.yml`. The fixture's broad serde_json requirement can select
newer metadata from the runner index; copying the project lock prevents that
independent resolution. Local checks had cached dependencies and did not expose
this cold-runner condition. No publication or release ref changes are in scope.

## Acceptance criteria

- [ ] A clean hosted full check passes its identity fixture.
- [ ] The temporary fixture starts with the project Cargo.lock.
- [ ] CI fetches locked dependencies before the offline tests.

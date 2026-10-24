---
id: "214"
title: "Limit the published crate to CLI runtime source and required notices"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "Real Cargo packaging includes required CLI source, build script, lockfile, license and user docs while excluding plugin and maintainer resources."
  - "A seeded Cargo regression fails with the previous broad package policy and passes with the explicit include policy; the full maintainer gate passes."
  - "A fresh clean crate builds and its contents, VCS identity and binary version are checked without registry publication."
tags: ["stream:tooling"]
---

# Limit the published crate to CLI runtime source and required notices

## What to build

TBD

## Acceptance criteria

- [ ] TBD

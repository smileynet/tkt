---
id: "213"
title: "Use packaged VCS metadata instead of an unrelated ancestor Git hash"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "A packaged build uses the .cargo_vcs_info.json commit, even when extracted inside another Git checkout; a non-Git build without metadata reports no fabricated hash."
  - "Ordinary and linked-worktree builds use their own commit and track the correct Git metadata for rebuilds; ambient Git repository overrides cannot redirect identity."
  - "Seeded regressions reproduce the old ancestor-hash bug and cover metadata validation; cargo fmt, clippy and all tests pass, and a rebuilt crate binary reports its sealed candidate commit."
tags: ["stream:tooling"]
---

# Use packaged VCS metadata instead of an unrelated ancestor Git hash

## What to build

TBD

## Acceptance criteria

- [ ] TBD

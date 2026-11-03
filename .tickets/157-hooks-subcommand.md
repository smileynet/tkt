---
id: "157"
title: "tkt hooks subcommand + pre-commit installer (warn-default, config block/log)"
status: backlog
blocked_by: []
priority: medium
validation_criteria:
  - "tkt hooks install writes a worktree-aware pre-commit shim that warns on staged status:done without Resolution"
  - "block and logging are opt-in via [hooks] config (both off by default); existing hooks chained via .legacy backup"
  - "Windows: LF-only shim, zero new deps, lossless tkt hooks uninstall"
tags: ["contract", "integrations", "kind:enhancement", "triage:needs-info", "stream:guidance", "gate:decision"]
---

# tkt hooks subcommand + pre-commit installer (warn-default, config block/log)

## What to build

Add a `tkt hooks` subcommand that installs a git pre-commit hook enforcing the close protocol. Decided policy (from #155): warn by default, block opt-in via config, logging opt-in (off by default).

**The hook:** a thin LF-only `#!/bin/sh` shim that delegates to `tkt hooks run pre-commit` (beads pattern — logic in the binary, so upgrades need no reinstall). `tkt hooks run pre-commit` scans staged `.tickets/*.md` for a `status: done` change lacking a `## Resolution` section (the hand-flip signature) and warns; exits 0 unless `[hooks] block = true`.

**Install (Strategy A, from research):**
- Locate the hooks dir via a NEW `git.rs` helper using `git rev-parse --git-common-dir` (worktree-aware — worktree hooks live in the shared `.git`, not the per-worktree gitdir).
- Back up any existing `pre-commit` to `pre-commit.legacy`, chain it (run it, propagate its exit code), and write a sentinel marker so tkt recognizes its own hook.
- Detect an existing `core.hooksPath` and warn rather than clobber.
- `tkt hooks uninstall` restores the `.legacy` backup losslessly.

**Config (`[hooks]` in .tickets/config.toml):**
- `block` (bool, default false) — escalate warn → hard fail (exit 1)
- `log` (bool, default false) — append warn/block outcomes to a local file (reuse the telemetry sink pattern in `src/telemetry.rs`); no writes unless enabled
- Opt-out: `TKT_SKIP_HOOKS=1`, native `git commit --no-verify`

## Context

- **Relevant files:** `src/cli.rs` (Hooks subcommand), `src/git.rs` (add `--git-common-dir` helper), `src/config.rs` (`[hooks]` section), `src/telemetry.rs` (log sink pattern)
- **Zero new deps** — const-string shim + `std::fs::write` + `git rev-parse`
- **Windows:** emit LF-only shim, pin `.gitattributes eol=lf`, copy (not symlink), keep logic in the binary
- From #155 decision record; research in `.scratch/subagent-raw/git-hook-install-patterns.md`

## Acceptance criteria

- [ ] `tkt hooks install` writes a worktree-aware pre-commit shim that warns on staged `status: done` without a Resolution section
- [ ] block and logging are opt-in via `[hooks]` config (both off by default)
- [ ] existing pre-commit hook is chained via `.legacy` backup; `core.hooksPath` collision is detected and warned
- [ ] `tkt hooks uninstall` restores the backup losslessly
- [ ] Windows: LF-only shim verified; zero new deps
- [ ] tests cover install/uninstall/chaining and the warn-vs-block config paths

## Out of scope

- Per-agent write-nudge (#158)
- CI gate docs (#159)

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:needs-info`; stream `integrations`.

Deferred opt-in product hook installer. Reconcile with #235 skipping git hooks and standing guidance against installing verification hooks; do not silently install hooks during this triage. Historical scratch research is not a dependency.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

## Planning gate

Reconcile the optional hook installer with mutation hook bypass and the standing verification policy.

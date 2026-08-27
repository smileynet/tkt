---
id: "155"
title: "Explore commit and agent hooks to enforce close protocol (beads-style)"
status: done
blocked_by: []
priority: high
validation_criteria:
  - "decision recorded: warn-default, configurable block (opt-in), configurable logging (off by default) for both outcomes"
  - "per-agent write-block mechanism documented (Claude Edit() deny/PreToolUse; Codex apply_patch hook)"
  - "follow-up implementation tickets created (hooks subcommand, init write-nudge, CI docs)"
tags: ["contract"]
---

# Explore commit and agent hooks to enforce close protocol (beads-style)

## What to build

Spike: evaluate git commit hooks and agent-lifecycle hooks/permissions as a stronger enforcement layer for the close protocol, beyond the detection-only approach in #154. Detection (validate/audit) catches a hand-flipped done ticket *after* it's committed; hooks could catch it *at* commit, *during* the agent session, or prevent the direct write entirely by routing all `.tickets/` frontmatter mutations through the `tkt` CLI.

Reference implementation: **beads** (https://github.com/gastownhall/beads) ships this model:
- `.githooks/` directory with committed hooks
- `bd setup <agent>` installs "skill, AGENTS.md guidance, AND hooks" per agent (codex, claude, factory, cursor, mux)
- `bd hooks install` as an explicit step; per-agent Claude/Codex hook integrations
- `--stealth` / `no-git-ops: true` opt-out for environments where hooks aren't wanted

## Decided policy (user, 2026-08-27)

- **Warn by default.** Every enforcement point (pre-commit hook, agent write-nudge) warns and allows the action. This matches the evidence: `.tickets/*.md` are hand-editable files, so a hard client-side block has no unbypassable boundary — it just pushes actors to raw-file edits + `--no-verify` (the documented bypass dynamic). The real gate stays CI (`tkt validate --strict` / `tkt lint --check`).
- **Block is configurable, opt-in.** A project may escalate any warn point to a hard block via config (e.g. `[hooks] block = true` or per-point keys). Off by default.
- **Logging is configurable, off by default.** Both warn and block outcomes can be logged (to a local file, reusing the telemetry sink pattern) when explicitly enabled (e.g. `[hooks] log = true` / `TKT_HOOK_LOG=1`). No logging unless turned on — no ambient writes.

## Research conclusions (spike answered, 2026-08-27)

Two research waves resolved the mechanics (full notes: `.scratch/subagent-raw/agent-hook-mechanisms.md`, `beads-hooks.md`, `git-hook-install-patterns.md`, `guardrail-warn-vs-block.md`, `codex-applypatch-hook.md`, `tkt-init-git-review.md`):

- **No unbypassable client-side boundary exists** — files are the database. Client controls are friction-that-teaches; CI `--strict` is the only real gate. (Autonoma/Notion/git semantics.)
- **Warn-decay caveat (Notion):** un-actioned warnings become noise. Each warn must be actionable, and the CI path must actually enforce — warn is a ramp to CI, not a parking lot.
- **Claude Code:** config-only path deny via `permissions.deny: ["Edit(.tickets/**)"]` — MUST be `Edit(...)`, `Write(...)` is silently ignored (v2.1.210+). Or a `PreToolUse` hook (matcher `Edit|Write`, inspect `tool_input.file_path`) for warn-mode `additionalContext`.
- **Codex:** no config path-glob deny; requires a `PreToolUse` hook (matcher `apply_patch`) that parses the patch envelope (`*** Update File:` markers) for `.tickets/` paths. VERSION-FRAGILE — interception silently didn't fire on older builds (#17794, #20204); hooks need `/hooks` trust re-review on any edit. Lower priority than Claude.
- **Install (Strategy A):** thin LF-only `#!/bin/sh` shim → `tkt hooks run <hook>` (beads delegation; upgrades need no reinstall). Back up existing hook to `.legacy`, chain it, sentinel marker, lossless uninstall. Detect existing `core.hooksPath` and warn. Add a `git rev-parse --git-common-dir` helper (worktree-aware — worktree hooks live in the shared `.git`). Windows: LF-only, `.gitattributes eol=lf`, copy not symlink, logic in the binary.
- **Zero new deps:** `serde_json` already present for `settings.json`/`hooks.json`; hook body is a const string + `std::fs::write`.
- **tkt has NO existing hooks infra** — `init.rs` writes per-agent markdown via `write_with_markers()`/`write_owned_file()`; `git.rs` shells out but has no git-dir/hooks-dir helper.

## Questions to answer

1. **Pre-commit hook:** on commit, scan staged `.tickets/*.md` for `status: done` changes that lack a `## Resolution` section (i.e., not produced by `tkt close`). Reject or warn? How does it interact with tkt's own commits (which ARE produced by close)?
2. **Agent-lifecycle hooks:** what do Claude Code / Codex hook points offer (pre-tool-use, post-edit)? Could a hook intercept a Write to a `.tickets/` file that flips status?
3. **Block direct agent writes to `.tickets/`:** should agents be prevented from writing to `.tickets/*.md` at all via their editor tool, forcing all mutations through the `tkt` CLI? Explore per-agent permission mechanisms:
   - Claude Code: `permissions.deny` / `Write(.tickets/**)` deny-rules in settings, or a `PreToolUse` hook that rejects Write/Edit tool calls targeting `.tickets/`
   - Codex / Cursor / others: equivalent deny-list or pre-tool hooks
   - **Nuance to resolve:** a blanket deny is too blunt — creating/editing ticket *bodies* by hand is legitimate and supported ("body is user-owned"). The target is specifically *frontmatter mutations* (status/blocked_by/priority) that have a CLI command. Options: (a) deny all `.tickets/` writes and require CLI for everything (simplest, most restrictive — breaks legit body edits); (b) deny only when the diff touches frontmatter fields tkt owns (precise, needs a smarter hook); (c) warn-not-block on any `.tickets/` write, nudging toward the CLI. Recommend which.
   - How does `tkt init --<agent>` install these deny-rules/hooks alongside the AGENTS.md snippet (like `bd setup`)?
4. **Installation:** should `tkt init` install hooks (like `bd setup`)? Opt-in or opt-out? How to avoid clobbering existing hooks (chaining, `core.hooksPath`)?
5. **Opt-out story:** local-only repos, CI, editors — mirror beads' `--stealth`/`no-git-ops`.
6. **Cross-platform:** hooks must work on Windows (this project's primary env) — bash hook portability.

## Deliverable

Spike is effectively answered (see conclusions above). Remaining deliverable: record the decision and split implementation into follow-up tickets:
1. `tkt hooks` subcommand + pre-commit installer (Strategy A, worktree-aware `--git-common-dir` helper, warn-default, `[hooks] block`/`log` config, LF shim)
2. `tkt init --claude/--codex` write-nudge hooks (warn-default, config to block)
3. Docs: CI recipe wiring `tkt validate --strict` / `tkt lint --check` as the real gate

## Context

- **Relevant files:** `src/commands/init.rs` (write-nudge install), `src/git.rs` (add `--git-common-dir` helper), `src/config.rs` (new `[hooks]` section), `src/telemetry.rs` (reuse local-file sink pattern for opt-in logging), `.memory/agent-guidance-surfaces.md`
- **Depends conceptually on #154** — hooks are the prevention layer; #154 is the detection layer (shipped). Do #154 first (done); this decides the prevention shape.
- **Constraint:** files are the database and hand-editing is explicitly supported — warn-default preserves legit body edits; hard block is opt-in only.

## Acceptance criteria

- [x] decision recorded: warn-default, config-to-block, opt-in logging (off by default) for both warn and block
- [x] recommendation on blocking direct agent writes to `.tickets/` (per-agent mechanism: Claude `Edit()` deny / PreToolUse; Codex apply_patch hook, version-fragile)
- [x] install strategy recorded (Strategy A shim + `--git-common-dir` worktree helper, Windows LF, zero new deps)
- [x] beads' approach documented as prior art (what to borrow, what to skip)
- [x] follow-up implementation tickets created (hooks subcommand; init write-nudge; CI docs)

## Out of scope

- Implementing the hooks (follow-up tickets above)
- Ratchet/baseline tracking (research: not justified for small `.tickets/` sets)

---
id: "158"
title: "tkt init installs per-agent write-nudge for .tickets/ (Claude, Codex)"
status: backlog
blocked_by: []
priority: low
validation_criteria:
  - "tkt init --claude writes .claude/settings.json PreToolUse warn-nudge (Edit matcher) toward tkt edit/close"
  - "tkt init --codex installs apply_patch PreToolUse hook, version-gated with clear caveat"
  - "warn-default; hard deny (Edit(.tickets/**)) only under opt-in config"
tags: ["contract"]
---

# tkt init installs per-agent write-nudge for .tickets/ (Claude, Codex)

## What to build

Extend `tkt init --<agent>` to install a per-agent nudge that steers frontmatter mutations through the `tkt` CLI instead of direct file writes. Decided policy (#155): warn by default, hard deny opt-in only.

**Claude Code** (`tkt init --claude` → `.claude/settings.json`):
- Default (warn): a `PreToolUse` hook (matcher `Edit|Write`) that, on a `.tickets/*.md` write, emits `additionalContext` nudging toward `tkt edit`/`tkt close` but ALLOWS the write (preserves legitimate body edits).
- Opt-in strict: `permissions.deny: ["Edit(.tickets/**)"]` — MUST be `Edit(...)`, not `Write(...)` (silently ignored v2.1.210+). Gated behind config/flag.
- Use managed-section markers (like init's existing marker mechanism) so re-run/`--check`/`--remove` work. `serde_json` already a dep.

**Codex** (`tkt init --codex` → `.codex/hooks.json`):
- `PreToolUse` hook (matcher `apply_patch`) parsing the patch envelope (`*** Update File:` etc.) for `.tickets/` paths, warn-mode.
- VERSION-FRAGILE: apply_patch interception silently didn't fire on older builds (#17794, #20204); hooks need `/hooks` trust re-review on edit. Document the caveat and version-gate. Lower priority than Claude.

## Context

- **Relevant files:** `src/commands/init.rs` (add .claude/.codex writers — currently writes only markdown), `src/config.rs` (block toggle)
- **Depends on #157** for the shared `[hooks]` config shape (block/log)
- Research: `.scratch/subagent-raw/agent-hook-mechanisms.md`, `codex-applypatch-hook.md`
- **Zero new deps** — serde_json already present for settings.json/hooks.json

## Acceptance criteria

- [ ] `tkt init --claude` writes a `.claude/settings.json` PreToolUse warn-nudge (Edit matcher) toward `tkt edit`/`close`
- [ ] `tkt init --codex` installs an apply_patch PreToolUse hook, version-gated with a clear caveat
- [ ] warn-default; hard deny (`Edit(.tickets/**)`) only under opt-in config
- [ ] managed-section markers allow idempotent re-run and clean `--remove`
- [ ] tests cover the settings.json/hooks.json generation

## Out of scope

- pre-commit hook + `tkt hooks` machinery (#157)
- CI gate docs (#159)

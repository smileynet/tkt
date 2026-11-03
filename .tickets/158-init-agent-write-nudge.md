---
id: "158"
title: "Generate optional advisory native write nudges"
status: open
blocked_by: []
priority: low
validation_criteria: 
  - "Explicit native config generation preserves foreign settings, owned content and dry-run; no hook or trust change happens implicitly"
  - "Claude Edit|Write path and Codex apply_patch command detection are bounded, advisory context only, with no deny or arbitrary shell/MCP coverage claim"
  - "Observed profiles, unknown minimum versions, trust and tool-coverage limits are documented; schema/merge/parser fixtures and native execution evidence limits are explicit"
tags: ["contract", "integrations", "kind:enhancement", "stream:guidance", "triage:ready-for-agent"]
---

# tkt init installs per-agent write-nudge for .tickets/ (Claude, Codex)

## Current executable contract — 2026-11-03

Current native advisory support is verified; see [the evidence](../docs/evidence/native-write-hooks-158.md). Implement explicit owned config generation for Claude Edit|Write and Codex apply_patch, with bounded file-path/patch parsing and model-visible additionalContext only. Do not change deny/permission policies, user settings, workspace trust or hook trust; emit reviewable project config only through explicit opt-in. Preserve foreign config and ownership, honor dry-run, and document interactive versus automation trust, incomplete tool coverage and unknown exact minimum versions. The locally observed profiles are Codex0.166.0 and Claude2.1.316, not claimed minimums. Git hook157 does not supply a shared native config dependency. This supersedes old nonexistent init flags, strict-deny and unsupported/silent-hook assumptions below. Native execution requires its own observed evidence or an explicit unverified limit.

## Acceptance criteria

- [ ] Explicit native config generation preserves foreign settings, owned content and dry-run; no hook or trust change happens implicitly
- [ ] Claude Edit|Write path and Codex apply_patch command detection are bounded, advisory context only, with no deny or arbitrary shell/MCP coverage claim
- [ ] Observed profiles, unknown minimum versions, trust and tool-coverage limits are documented; schema/merge/parser fixtures and native execution evidence limits are explicit

## What to build

Extend `tkt init --<agent>` to install a per-agent nudge that steers frontmatter mutations through the `tkt` CLI instead of direct file writes. Decided policy (#155): warn by default, hard deny opt-in only.

**Claude Code** (`tkt init --claude` → `.claude/settings.json`):
- Default (warn): a `PreToolUse` hook (matcher `Edit|Write`) that, on a `.tickets/*.md` write, emits `additionalContext` nudging toward `tkt edit`/`tkt close` but ALLOWS the write (preserves legitimate body edits).
- Opt-in strict: `permissions.deny: ["Edit(.tickets/**)"]` — MUST be `Edit(...)`, not `Write(...)` (silently ignored v2.1.210+). Gated behind config/flag.
- Use managed-section markers (like init's existing marker mechanism) so re-run/`--check`/`--remove` work. `serde_json` already a dep.

**Codex** (`tkt init --codex` → `.codex/hooks.json`):
- `PreToolUse` hook (matcher `apply_patch`) parsing the patch envelope (`*** Update File:` etc.) for `.tickets/` paths, warn-mode.
- VERSION-FRAGILE: apply_patch interception silently didn't fire on older builds (Codex#17794, Codex#20204); hooks need `/hooks` trust re-review on edit. Document the caveat and version-gate. Lower priority than Claude.

## Context

- **Relevant files:** `src/commands/init.rs` (add .claude/.codex writers — currently writes only markdown), `src/config.rs` (block toggle)
- **Depends on #157** for the shared `[hooks]` config shape (block/log)
- Research: `.scratch/subagent-raw/agent-hook-mechanisms.md`, `codex-applypatch-hook.md`
- **Zero new deps** — serde_json already present for settings.json/hooks.json

## Historical acceptance proposal (superseded)

- [ ] `tkt init --claude` writes a `.claude/settings.json` PreToolUse warn-nudge (Edit matcher) toward `tkt edit`/`close`
- [ ] `tkt init --codex` installs an apply_patch PreToolUse hook, version-gated with a clear caveat
- [ ] warn-default; hard deny (`Edit(.tickets/**)`) only under opt-in config
- [ ] managed-section markers allow idempotent re-run and clean `--remove`
- [ ] tests cover the settings.json/hooks.json generation

## Out of scope

- pre-commit hook + `tkt hooks` machinery (#157)
- CI gate docs (#159)

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:needs-info`; stream `integrations`.

Add missing dependency #157 for its shared hook config. Harness hook claims are version-sensitive and unverified; current Codex plugin explicitly installs no hooks. Verify official support before implementation.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

## Planning gate

Verify current harness hook support and version constraints; historical hook claims are unverified.

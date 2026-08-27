# ADR 0001: Enforcement is warn-by-default, block opt-in, logging opt-in

- Status: accepted
- Date: 2026-08-27
- Source: ticket #155 (spike), decided by user

## Context

`.tickets/*.md` are plain files (files-are-the-database). Agents/humans can always
edit a file directly and `git commit --no-verify`, so tkt has **no unbypassable
client-side enforcement boundary**. We evaluated hooks (pre-commit, agent PreToolUse)
and per-agent write-deny rules as a way to stop hand-edits that skip `tkt close`
(e.g. flipping `status: done`).

Research (`.scratch/subagent-raw/guardrail-warn-vs-block.md`): hard client-side blocks
on hand-editable files get routed around (the `--no-verify` dynamic) and train users to
abandon the tool; pure-warn streams decay to ignored noise (Notion). The only real gate
is CI running `tkt validate --strict` / `tkt lint --check`.

## Decision

- **Warn by default.** Every client-side enforcement point (pre-commit hook, agent
  write-nudge) warns and allows the action.
- **Block is opt-in** via `[hooks] block = true` project config. Off by default.
- **Logging is opt-in** (off by default) for both warn and block outcomes, via
  `[hooks] log = true` / `TKT_HOOK_LOG=1`, reusing the telemetry local-file sink. No
  ambient writes.
- **CI is the real gate.** Client controls are friction-that-teaches; `validate --strict`
  in CI is the enforcement boundary (documented in #159).

## Consequences

- Hooks/deny-rules never hard-block by default — preserves the "edit by hand anytime"
  contract for ticket bodies.
- Implementation split into #157 (hooks subcommand), #158 (init write-nudge), #159 (CI docs).
- Detection layer already shipped in #154 (validate flags hand-flipped done, warn/strict).

## Alternatives rejected

- **Block by default:** rejected — bypassed via raw file edits, abandons the tool.
- **Blanket `.tickets/` write-deny:** rejected — breaks legitimate hand-editing of ticket bodies.
- **Ratchet/baseline file:** rejected for now — overkill for small `.tickets/` sets.

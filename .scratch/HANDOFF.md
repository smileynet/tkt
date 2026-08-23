---
created_at: 2026-08-23T16:46:25-07:00
base_commit: cf9693c
handoff_key: tkt-v031-bugfix
---

# Handoff

## Objective
Ship v0.3.1 — fix data-loss parsing bugs and contract violations before next release.

## Constraints
- v0.3.0 shipped (crates.io + GitHub releases live). Post-release bugfix work in progress.
- User config enforces: require_resolution, require_validation_criteria, require_validation_evidence, allow_force=false
- `TKT_DEBUG=1` now defaults to file output (`~/.local/state/tkt/debug.log`); use `TKT_DEBUG=stderr` for terminal
- Telemetry currently disabled in consent.toml (keeps re-disabling; investigate if needed)
- Release ticket 146 blocks on 10 tickets (131-137, 139, 141, 127)

## Prior Decisions
- `tags` = what work is about (context scoping); `requires` = what machine needs (capability constraint) — two distinct fields
- Audit: CLI does only mechanical checks; judgment calls belong to agent skill (audit-quality.md)
- Debug defaults to file (not stderr) for agent-friendly quiet output
- Migration requires --dry-run preview before apply (marker file gate)
- OpenCode supported via AGENTS.md + .claude/skills/ (already works, added as init target)

## Current State
- v0.3.0 released with: context, migrate, requires, telemetry observability, audit --deep
- Architecture review (ticket 128) decomposed into 15 tickets (131-145)
- Release ticket 146 tracks v0.3.1 gate — blocked by all urgent/high/medium fixes
- Frontier: 2 urgent (131 parsing, 132 hand-edits), 6 high, 2 medium before release
- Project cleanup done: scratch cleared, AGENTS.md trimmed to 143 lines

## Next Steps
1. Fix 131 (urgent): block-style blocked_by parsed as empty + lint destroys deps
2. Fix 132 (urgent): BOM/comments/space-colon eject tickets from corpus
3. Fix 133-136 (high): unescape, renumber, JSON envelope, dry-run
4. Fix 137, 139, 141 (medium): origin/main, evidence gates, uppercase checkboxes
5. Fix 127 (high): capabilities manifest + help nudge (also blocks 138)
6. Close 146 → cut v0.3.1 release

## Fog
- Ticket 128 findings are unconfirmed hypotheses (from ox-alpha review). Each needs independent reproduction before fixing — some may be rejected.
- F6 (--dry-run ignored by 7 commands) is broad — may reveal design questions about which commands should respect global dry-run vs have local flags.
- F7 (origin/main hardcoded) may be complex if repos have non-standard remote setups.

## Evidence
- `tkt ready`: 18 frontier tickets, release blocker 146 in `tkt blocked`
- `tkt validate --brief`: pass (0 findings)
- `tkt --version`: tkt 0.3.0 (2c0c021) — installed locally
- All tests pass (56 integration + unit), 0 clippy warnings

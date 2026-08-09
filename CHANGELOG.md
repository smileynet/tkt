# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/2.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-08-09

### Added

- Frontier computation: `tkt ready` shows unblocked tickets sorted by priority
- Push-to-claim: atomic ticket allocation via git push with race detection
- Multi-level priority: urgent > high > medium > low (frontier sort order)
- Backlog status: park tickets outside the frontier with `status: backlog`
- Ticket lifecycle: `tkt new`, `tkt claim`, `tkt close`, `tkt edit`, `tkt renumber`
- Batch ticket creation: `tkt batch` for multiple tickets in one push
- Query with filters: `tkt query --status open --priority high`
- Blocked view: `tkt blocked` shows tickets with unsatisfied dependencies
- Validation: `tkt validate` checks for cycles, dangling deps, contract violations
- Audit: `tkt audit` for closure quality (unchecked ACs, missing resolutions, stale WIP)
- Plan sync: `tkt sync-plan` detects and fixes drift between tickets and plan documents
- ID collision resolution: `tkt rebase` auto-renumbers when upstream conflicts
- Agent discovery: `tkt capabilities` outputs JSON manifest of commands and workflows
- Per-project config: `.tickets/config.toml` for tunable behaviors (push, close, validate)
- User config: `~/.config/tkt/config.toml` for debug mode and format preferences
- Color support: `--color=always|never|auto`, respects `NO_COLOR` (no-color.org)
- ASCII fallback: `TKT_ASCII=1` for legacy terminals (✓→[ok], ✗→[err], ⚠→[warn])
- Spike branch awareness: auto-appends branch name to resolution on close
- Worktree support: works correctly from git worktrees
- Local-only telemetry: opt-in JSONL recording, never leaves your machine
- Debug mode: `TKT_DEBUG=1|json` for real-time diagnostics to stderr

[Unreleased]: https://github.com/smileynet/tkt/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/smileynet/tkt/releases/tag/v0.1.0

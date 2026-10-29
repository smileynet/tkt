# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Tickets can allow or exclude named agents independently of tags and machine capabilities. Identify invocations with `--agent` or `TKT_AGENT`; discovery and claims share the policy, with `claim --force` as an explicit override.
- Codex and Antigravity plugin packaging supplies the four ticket workflows: `tkt-core`, `tkt-orient`, `tkt-work` and `tkt-reconcile`. Install the tkt CLI separately.
- `audit.resolution_baseline_id` can grandfather historical resolution-quality findings while keeping acceptance-criteria and evidence checks active.
- Release maintenance can prepare and verify approved candidates without publication, then use a separately authorized reviewed CI publisher with per-channel recovery.
- Claude Code sessions show the launch project's ticket frontier and work in progress when tkt is on PATH. Requires Claude 2.1.310+ and Node 22+.
- tkt is installable as a Claude Code plugin: `/plugin marketplace add smileynet/tkt`, then
  `/plugin install tkt@tkt`

### Changed
- Claiming, closing, and editing ticket status keep existing default plan rows current in the same commit. Plan repair also adds missing tickets.
- **Breaking:** compiling tkt now requires Rust 1.85.0 or newer. Update Rust with
  `rustup update stable`, or use a pre-built binary.
- `node tools/deploy.mjs` selects roots so installed Claude/Codex plugins count as the skill copy and refreshes local plugins. Audits identify unowned skill directories and duplicate loading across configured packs. The bash wrapper remains an alternative entry point; no extra deploy step is needed.

### Fixed
- Workflow skills use supported tag-context queries and preserve the previous scope during temporary filtering.
- Upgrades from v0.4.0 can preview and remove old skill symlinks before installing the four workflow skills, while preserving unrelated files and source checkouts.
- Local Claude plugin audits use the active checkout rather than stale cache folders. Failed or foreign source checks preserve existing skill copies during deployment.
- Plugin archives retain their referenced runtime resources and license notices when extracted away from the source checkout.
- Packaged binaries report their own source identity, and published crates exclude scratch, plugin packages and maintainer files.

## [0.4.0] - 2026-10-19

### Added

- OpenCode is now supported as a project-instruction target when initializing tkt
- `TKT_DEBUG=log` records diagnostics in a local log file for later inspection
- `tkt validate` now reports placeholder ticket bodies and tickets manually changed to
  `done` outside the closure workflow

### Changed

- `TKT_DEBUG=1` now writes diagnostics to a local file; use `TKT_DEBUG=stderr` or
  `TKT_DEBUG=json` when diagnostics must be emitted to standard error
- `tkt sync-plan` reports drift without failing by default; use `--check` to enforce it
  in automation and `--fix` to apply safe derived updates
- Closure errors now show all unmet requirements together, making it easier to correct
  a ticket in one pass
- `tkt validate --fix` stops rather than applying a repair that would introduce new
  validation findings
- `tkt migrate` now requires an explicit `--confirm` after its no-write preview has
  been reviewed

### Fixed

- Hand-edited unquoted ticket fields now retain literal backslashes instead of
  interpreting sequences such as `\n` as escapes
- Tickets with a byte-order mark, comments, or block-style dependencies (including a
  one-item list) are parsed without being dropped or losing dependency information
- `tkt renumber` now refuses to change a cited ID and validates the complete local
  ticket corpus before making any change
- Frontmatter normalization preserves `blocked_by` references while correcting their
  style
- Failed ticket writes retain the existing ticket and report the underlying operating
  system error; atomic replacement is verified on Windows and Linux, and temporary
  write files use collision-resistant names and are cleaned up after failure
- A successful ticket commit followed by a failed push now reports the recoverable
  commit instead of treating the work as lost
- JSON-mode errors now end with the structured error envelope, so automation can
  reliably parse the final stderr line
- Remote race protection now follows the current tracking branch, including `master`
  and non-`origin` remotes, instead of assuming `origin/main`; when a branch has no
  tracking configuration, it pushes explicitly to the resolved remote HEAD branch
- Global `--dry-run` now consistently previews rather than changing tickets,
  configuration, context, project setup, frontmatter, renumbered IDs, or the local
  update-check cache
- Migration previews no longer leave a confirmation file behind
- Partial `--evidence` at close now honors the configured evidence policy: it warns
  under `warn`, blocks under `true`, and is ignored under `false`, instead of always
  blocking; closing a ticket that has no validation criteria now warns that supplied
  evidence was ignored rather than discarding it silently
- Hand-typed uppercase `- [X]` acceptance-criteria boxes are now counted as checked,
  so they satisfy the close gate and are no longer reported as unchecked
- `tkt doctor` now runs the same full validation as `tkt validate` — reporting
  unparseable tickets by name, stub bodies, and closure-quality issues — so it can no
  longer report a clean project that `tkt validate` fails
- Indented frontmatter comments no longer become part of the preceding ticket field
- Projects can grandfather historical closure records with a numeric resolution-check
  cutoff, keeping strict validation focused on tickets created after that policy began
- `tkt ready` now evaluates legacy `env` and modern `requires` through one capability
  model; ambiguous mixed tickets are rejected instead of being silently filtered

## [0.3.0] - 2026-08-23

### Added

- `tkt context` command — scope your session to specific tags so you only see relevant tickets and new tickets inherit those tags automatically
- `tkt migrate` command — detect and convert foreign ticket schemas (built-in `tk` adapter with ID remapping and dep rewiring)
- Machine capability matching via `requires` field — tickets only appear on frontier if the workstation has the required capabilities (`machine.capabilities` config)
- `--tags` flag on `tkt new` and `tkt batch` for explicit ticket categorization
- `--requires` flag on `tkt new` and `tkt batch` for declaring machine requirements
- `tkt audit --deep` for mechanical quality checks (evidence count mismatch, template-only closures)
- Validation criteria now enforced at ticket creation time (when configured) — agents must define success upfront, no escape hatches
- Telemetry now captures error classification on failures, which flags were used, and result counts for read commands
- Telemetry `--show` displays command distribution, error rate, workflow patterns, and per-command friction scores
- Telemetry `--show --all` displays full event history instead of last 20
- Telemetry captures parse/syntax errors when agents use wrong flag names or unknown subcommands
- Properly-migrated tickets (resolution contains "Migrated to") no longer flagged by `tkt audit --deep`

### Changed

- `tkt ready` now filters by active tag context (set via `tkt context`) in addition to env/requires
- `CREW_ENV` is now legacy — prefer `requires` field + `machine.capabilities` config for new projects
- Telemetry `--show` output restructured with summary header before event list

## [0.2.1] - 2026-08-18

### Fixed

- `tkt init` now teaches agents about backlog workflow, status transitions, and evidence-based closure
- Agent instructions generated by `tkt init --all` were missing `--status backlog`, `--evidence`, and promote/demote patterns — agents would create open tickets when intending to park work

## [0.2.0] - 2026-08-18

### Added

- `tkt init` command to set up a project and deploy AI agent instructions in one step
- `tkt doctor` command to verify your setup is correct
- `tkt doctor <path>` to scan all your projects at once and report health
- `tkt lint` command to normalize ticket frontmatter style (quoting, field order, whitespace)
- `tkt lint --check` for CI — exits 1 if any file would change
- `tkt validate --fix` to automatically repair common ticket issues
- Validation criteria on tickets — define what "done" means at creation time
- Evidence on close — link proof to each criterion so reviewers see what was verified
- Configurable enforcement for validation evidence (warn, require, or off)
- `tkt audit` flags tickets closed without evidence when criteria were defined
- `--dry-run` flag on all mutation commands — preview what would happen without writing
- Global `-o json` flag — structured JSON output for all commands
- Structured error envelopes — machine-parseable errors with kind, message, hint, and exit code
- Error kind vocabulary (8 types) declared in `tkt capabilities` with retryable flags
- Self-update check — notifies when a newer version is available (once per day, non-blocking)
- Unified config cascade — user config provides global defaults, project config overrides per-repo
- `close.allow_force` config — disable the `--force` escape hatch project-wide or globally
- Git hash in `--version` output for dev build traceability (e.g., `tkt 0.2.0 (ea047fb)`)
- Agent Plugins conformance — ships `plugin.json` and `skills/tkt/SKILL.md` per agentskills.io spec
- `tkt doctor <path>` now flags non-tkt git repos and supports `--strict` and `-o json`

### Changed

- `tkt config --show` now reports the source of each resolved value (env/project/user/default)
- `ready --json` still works but `-o json ready` is now the canonical form
- Clearer README and crate description

### Fixed

- Evidence gate: duplicate named evidence indices (e.g., `1=foo 1=bar`) no longer bypass the complete-evidence requirement

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

[Unreleased]: https://github.com/smileynet/tkt/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/smileynet/tkt/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/smileynet/tkt/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/smileynet/tkt/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/smileynet/tkt/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/smileynet/tkt/releases/tag/v0.1.0

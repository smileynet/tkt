---
id: "64"
title: "Release automation: cargo-release + git-cliff + CHANGELOG.md"
status: open
blocked_by: ["01"]
priority: high
---

# Release automation: cargo-release + git-cliff + CHANGELOG.md

## What to build

Set up the release toolchain for tkt: automated changelog generation from conventional commits, version bumping, tagging, and pushing — all gated behind a single command.

## Research basis

Studied three reference implementations (.references/):
- **cargo-release** (crate-ci) — local-first release orchestrator, dry-run by default, hook system with env vars
- **git-cliff** (orhun) — changelog generator, Tera templates, keepachangelog format, conventional commit parsing
- **release-plz** (MarcoIeni) — CI-first alternative, useful patterns but overkill for tkt's single-crate model

## Deliverables

### 1. CHANGELOG.md (keepachangelog format)

Create initial `CHANGELOG.md` covering everything shipped to date:

```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/2.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - YYYY-MM-DD

### Added
- Frontier computation with multi-level priority (urgent > high > medium > low)
- Push-to-claim atomic ticket allocation with race detection
- Dependency-aware frontier (`tkt ready`)
- Ticket lifecycle: new, claim, close, edit, renumber
- Query filters (`--status`, `--priority`) and `tkt blocked` view
- Per-project config (`.tickets/config.toml`) with 7 tunable settings
- User config (`~/.config/tkt/config.toml`) with debug/format preferences
- `tkt capabilities` machine-readable JSON manifest for agent discovery
- `tkt rebase` for resolving ID collisions with upstream
- `tkt audit` for closure quality checking
- `tkt validate` for contract/cycle/decay findings
- `tkt sync-plan` for drift detection against plan documents
- Color and symbol support (`--color`, `NO_COLOR`, `TKT_ASCII`)
- Spike branch awareness (auto-appends branch to resolution)
- Worktree support (works from git worktrees)
- Local-only telemetry (opt-in JSONL, session-aware rotation)
- Batch ticket creation (`tkt batch`)
- Debug mode (`TKT_DEBUG=1|json`)

[Unreleased]: https://github.com/smileynet/tkt/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/smileynet/tkt/releases/tag/v0.1.0
```

### 2. cliff.toml (git-cliff config)

Keepachangelog-format output with conventional commit parsing:

```toml
[changelog]
header = """
# Changelog\n
All notable changes to this project will be documented in this file.\n
The format is based on [Keep a Changelog](https://keepachangelog.com/en/2.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).\n
"""
body = """
{% if version %}\
    ## [{{ version | trim_start_matches(pat="v") }}] - {{ timestamp | date(format="%Y-%m-%d") }}
{% else %}\
    ## [Unreleased]
{% endif %}\
{% for group, commits in commits | group_by(attribute="group") %}
    ### {{ group | striptags | trim | upper_first }}
    {% for commit in commits %}
        - {{ commit.message | upper_first | trim }}\
          {% if commit.breaking %} (**Breaking**){% endif %}\
          {% if commit.scope %} ({{ commit.scope }}){% endif %}\
    {% endfor %}
{% endfor %}\n
"""
trim = true

[git]
conventional_commits = true
filter_unconventional = true
commit_parsers = [
    { message = "^feat", group = "<!-- 0 -->Added" },
    { message = "^fix", group = "<!-- 1 -->Fixed" },
    { message = "^perf", group = "<!-- 2 -->Changed" },
    { message = "^refactor", group = "<!-- 2 -->Changed" },
    { message = "^docs", skip = true },
    { message = "^chore\\(release\\)", skip = true },
    { message = "^chore\\(tickets\\)", skip = true },
    { message = "^chore\\(deps\\)", skip = true },
    { message = "^test", skip = true },
    { message = "^chore", skip = true },
]
protect_breaking_commits = true
sort_commits = "oldest"
```

### 3. release.toml (cargo-release config)

```toml
[workspace]
allow-branch = ["main"]
pre-release-hook = ["git", "cliff", "--output", "CHANGELOG.md", "--tag", "{{version}}"]
pre-release-commit-message = "chore(release): v{{version}}"
tag-message = "tkt v{{version}}"
tag-prefix = "v"
tag-name = "v{{version}}"
push = true
publish = true
verify = true
```

### 4. mise task: `mise run release`

```toml
# mise.toml
[tasks.release]
description = "Cut a release (dry-run by default, use -- --execute to ship)"
run = """
cargo fmt --check || { echo "fmt check failed"; exit 1; }
cargo clippy --all-targets -- -D warnings || { echo "clippy failed"; exit 1; }
cargo test || { echo "tests failed"; exit 1; }
cargo release ${1:-patch} "$@"
"""

[tasks."release:dry"]
description = "Preview what a release would do"
run = "cargo release ${1:-patch}"
```

### 5. Skill updates (for spellbook)

**changelog-discipline** additions:
- Add keepachangelog v2.0.0 file structure spec (header, `[Unreleased]`, version headings, link footer)
- Add Deprecated and Security categories to the decision table
- Add comparison link format: `[version]: https://...compare/vX...vY`
- Add "reverse chronological" as explicit rule
- Add anti-pattern: "Dependencies" is not a category

**release-protocol** additions:
- Add `Cargo.toml` version bump as explicit step (or note cargo-release handles it)
- Add `cargo publish` step with `--dry-run` gate
- Add pre-release version pattern (alpha/rc)
- Add `cargo-release` as recommended tool with config example
- Add `git-cliff` as changelog generation tool
- Note: fix-forward policy (never delete pushed tags) already covered ✓

### 6. Tool script: `tools/release.sh`

Fallback for environments without mise:

```bash
#!/usr/bin/env bash
set -euo pipefail

# Gate
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test

# Release (dry-run unless --execute passed)
cargo release "${1:-patch}" "${@:2}"
```

## Acceptance criteria

- [ ] CHANGELOG.md created with v0.1.0 content (keepachangelog format)
- [ ] cliff.toml configured for conventional commit → keepachangelog mapping
- [ ] release.toml configured with pre-release hook to generate changelog
- [ ] `mise run release` (or `tools/release.sh`) gates on fmt+clippy+test before releasing
- [ ] `cargo release patch` dry-run produces correct plan
- [ ] Skill updates documented (changelog-discipline + release-protocol gaps)
- [ ] Commit messages follow conventional commits going forward

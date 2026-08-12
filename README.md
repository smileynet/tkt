# tkt

Track tasks as markdown files in your git repo.

[![CI](https://github.com/smileynet/tkt/actions/workflows/ci.yml/badge.svg)](https://github.com/smileynet/tkt/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/tkt.svg)](https://crates.io/crates/tkt)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## What It Does

tkt keeps your task list in `.tickets/` — one markdown file per task, with status and dependencies in the header. It tells you what's ready to work on next.

```
.tickets/
├── 01-auth-system.md        # done
├── 02-api-endpoints.md      # open, waiting on 01
└── 03-deploy-pipeline.md    # open, waiting on 02
```

| When I'm... | I want to... | So I can... |
|---|---|---|
| Starting a work session | see what's unblocked | pick the right thing to work on |
| Grabbing a task | know nobody else took it | avoid duplicate effort |
| Finishing something | mark it done and unblock the next tasks | keep things moving |
| Working with others | create tasks without ID collisions | push tickets at the same time |
| Checking project health | find cycles or broken references | catch problems early |

## Quick Start

**You need:** `git` installed, inside a git repo.

```bash
# Install
curl -fsSL https://github.com/smileynet/tkt/releases/latest/download/tkt-installer.sh | sh

# Set up your tickets directory
mkdir .tickets && git add .tickets && git commit -m "init tickets"

# Create your first task
tkt new auth-system --title "Implement authentication"
# → ✓ created 01 auth-system (pushed)

# See what's ready
tkt ready
# → Ready (1):
# →   01  Implement authentication

# Claim it (marks in-progress, tells others it's taken)
tkt claim 01
# → ✓ claimed 01 auth-system (→ in_progress)

# Close it when done
tkt close 01 --note "JWT + refresh tokens shipped"
# → ✓ closed 01 auth-system (Resolution written)
```

## Install

Pre-built binaries (fastest):

```bash
# macOS / Linux
curl -fsSL https://github.com/smileynet/tkt/releases/latest/download/tkt-installer.sh | sh

# Windows (PowerShell)
irm https://github.com/smileynet/tkt/releases/latest/download/tkt-installer.ps1 | iex
```

From crates.io:

```bash
cargo install tkt

# Or with cargo-binstall (downloads pre-built binary, no compile)
cargo binstall tkt
```

From source:

```bash
cargo install --path .
```

Single binary, no runtime dependencies beyond `git`.

## Usage

### See what's ready

```bash
tkt ready              # human-friendly list
tkt ready --json       # machine-readable (JSON Lines)
```

Shows tasks that are open and have all their dependencies done, sorted by priority.

### Create tasks

```bash
tkt new fix-login --title "Fix login timeout" --priority high
tkt new deploy --title "Deploy to staging" --blocked-by 01,02
tkt batch "api:Build API" "docs:Write docs" --blocked-by 01
```

IDs are assigned automatically. In shared repos, tkt checks both local and remote files to avoid collisions.

### Work on tasks

```bash
tkt claim 03           # mark as in-progress (visible to others)
tkt close 03 --note "Deployed" --ac 1,2   # mark done, check acceptance criteria
```

`claim` is optional — `close` works directly on open tasks. Use `claim` in shared repos so others know what you're working on.

### Edit and maintain

```bash
tkt edit 02 --title "New title" --blocked-by 01,03 --priority high
tkt validate           # check for cycles, broken references, contract issues
tkt validate --fix     # auto-repair fixable problems
tkt sync-plan --check  # compare ticket status vs a plan document
tkt query              # dump everything as JSON Lines
tkt blocked            # show tasks stuck waiting on dependencies
```

### Flags reference

| Flag | Used by | Effect |
|------|---------|--------|
| `--json` | ready | machine-readable output |
| `--strict` | validate, sync-plan | treat warnings as errors |
| `--brief` | validate, sync-plan | short human output |
| `--blocked-by N,N` | new, batch, edit | set dependencies |
| `--priority P` | new, batch, edit | urgent, high, medium (default), low |
| `--note "..."` | close | explain what was done |
| `--ac N,N` | close, edit | check acceptance criteria boxes |
| `--check-all` | close | check all acceptance criteria at once |
| `--force` | close | close even with unchecked criteria |

## Task Format

```yaml
---
id: "01"
title: "Implement authentication"
status: open          # backlog | open | in_progress | done
blocked_by: []        # IDs that must be done first
priority: high        # optional: urgent > high > medium > low
---

# Implement authentication

## What to build
JWT-based auth with refresh token rotation...

## Acceptance criteria
- [ ] JWT tokens issued on login
- [ ] Refresh token rotation works
```

Tasks are just files. Edit them by hand anytime — tkt reads whatever's there.

## Configuration

Project config in `.tickets/config.toml` (committed with your repo):

```toml
[push]
enabled = true            # false for local-only repos (no network calls)

[close]
require_resolution = false  # require a --note when closing
require_checked_acs = false # require all acceptance criteria checked

[validate]
strict = false            # treat warnings as errors

[ready]
default_env = ""          # filter tasks by environment

[new]
default_priority = "medium"
```

Manage with `tkt config --list` or `tkt config --set push.enabled=false`.

## How It Works

- Tasks that depend on other tasks won't show up in `tkt ready` until those dependencies are done
- When you create or claim a task, tkt pushes immediately — if someone else pushed first, it retries with a new ID
- Edits only touch the specific field you changed, leaving everything else untouched
- All reads are local and fast (~50ms). Writes include a git push round-trip (~2s) — disable with `push.enabled = false` for local-only workflows

## AI Agent Integration

tkt works well with AI coding agents. Add to your AGENTS.md:

```markdown
## Tasks

tkt ready                                         # what to work on next
tkt claim <id>                                    # mark as in-progress
tkt close <id> --check-all --resolution "..."     # mark done
tkt validate --brief                              # check for problems
```

Solo workflow: `tkt ready` → pick one → `tkt close <id> --check-all --resolution "..."`.

Multi-agent workflow: `tkt ready` → `tkt claim <id>` → work → `tkt close <id>`.

## Development

```bash
cargo build            # debug build
cargo test             # all tests
cargo clippy           # lint (must be 0 warnings)
cargo fmt --check      # format check
```

## Telemetry

Optional, local-only telemetry (disabled by default). Nothing leaves your machine. See [TELEMETRY.md](TELEMETRY.md).

```bash
tkt telemetry --enable   # opt in
tkt telemetry --status   # check what's stored
tkt telemetry --disable  # opt out
```

## Contributing

Found a bug? [File a report](https://github.com/smileynet/tkt/issues/new?template=bug_report.md). Want a feature? [Request it](https://github.com/smileynet/tkt/issues/new?template=feature_request.md).

## License

MIT

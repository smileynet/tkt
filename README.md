# tkt

Track work as markdown files in git. Dependency-aware frontier, atomic push-to-claim, race detection — no server required.

[![CI](https://github.com/smileynet/tkt/actions/workflows/ci.yml/badge.svg)](https://github.com/smileynet/tkt/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/tkt.svg)](https://crates.io/crates/tkt)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## What It Does

tkt manages `.tickets/` files with YAML frontmatter inside any git repo. It computes what's ready to work on (the "frontier"), claims tickets atomically via git push, and detects when two people grab the same work.

```
.tickets/
├── 01-auth-system.md        # status: done
├── 02-api-endpoints.md      # status: open, blocked_by: [01]
└── 03-deploy-pipeline.md    # status: open, blocked_by: [02]
```

| When I'm... | I want to... | So I can... |
|---|---|---|
| Starting a work session | see what's unblocked | pick the highest-priority available task |
| Claiming a ticket | know nobody else grabbed it | avoid duplicate work across sessions |
| Finishing a task | mark it done and unblock dependents | keep the pipeline moving |
| Working on a team repo | allocate IDs without collision | push tickets concurrently |
| Reviewing project health | check for cycles and dangling refs | catch structural issues early |
| Syncing a plan document | detect status drift | keep the plan accurate |

## Quick Start

**Prerequisites:** `git` on PATH, inside a git repository.

```bash
# Install
curl -fsSL https://github.com/smileynet/tkt/releases/latest/download/tkt-installer.sh | sh

# Create the tickets directory
mkdir .tickets && git add .tickets && git commit -m "init tickets"

# Create your first ticket
tkt new auth-system --title "Implement authentication"
# → ✓ created 01 auth-system (pushed)

# See what's ready to work on
tkt ready
# → Ready (1):
# →   01  Implement authentication

# Claim it (marks in_progress, pushes to remote)
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

### Frontier — what's ready

```bash
tkt ready              # human output
tkt ready --json       # JSON Lines (one object per ticket)
```

Shows open tickets whose dependencies are all done, sorted by priority then ID.

### Create tickets

```bash
tkt new fix-login --title "Fix login timeout" --priority high
tkt new deploy --title "Deploy to staging" --blocked-by 01,02
tkt batch "api:Build API" "docs:Write docs" --blocked-by 01
```

IDs are allocated atomically — tkt scans local and remote filenames, pushes immediately, and retries on collision.

### Lifecycle

```bash
tkt claim 03           # open → in_progress (pushed)
tkt close 03 --note "Deployed" --ac 1,2   # → done, checks AC boxes 1 and 2
```

`claim` is optional — `close` works directly on `open` tickets. Use `claim` in shared repos to signal WIP and detect races.

### Edit and maintain

```bash
tkt edit 02 --title "New title" --blocked-by 01,03 --priority high
tkt validate           # check for cycles, dangling deps, contract violations
tkt validate --fix     # auto-repair fixable issues (quoting, invalid fields)
tkt sync-plan --check  # compare ticket status vs docs/plan.md table
tkt query              # dump full corpus as JSON Lines
tkt blocked            # show tickets with unsatisfied deps
```

### Common flags

| Flag | Used by | Effect |
|------|---------|--------|
| `--json` | ready | machine-readable output |
| `--strict` | validate, sync-plan | warnings become errors |
| `--brief` | validate, sync-plan | human output instead of JSON |
| `--blocked-by N,N` | new, batch, edit | set dependencies |
| `--priority P` | new, batch, edit | urgent, high, medium (default), low |
| `--env E` | new, batch, edit | corp / personal / either |
| `--note "..."` | close | resolution text |
| `--ac N,N` | close, edit | check acceptance criteria boxes |
| `--check-all` | close | check all AC boxes at once |
| `--force` | close | close even with unchecked ACs |

## Ticket Format

```yaml
---
id: "01"
title: "Implement authentication"
status: open          # backlog | open | in_progress | done
blocked_by: []        # ids that must be done first
priority: high        # optional: urgent > high > medium > low
env: corp             # optional: corp | personal | either
spec: auth-spec       # optional: links to a spec name
---

# Implement authentication

## What to build
...

## Acceptance criteria
- [ ] JWT tokens issued on login
- [ ] Refresh token rotation
```

Files are the database. Hand-edit any time — tkt reads what's there.

## Configuration

Project-level config lives in `.tickets/config.toml` (committed, shared by contributors):

```toml
[push]
enabled = true            # set false for local-only repos (skips fetch/push)

[close]
require_resolution = false  # require --note/--resolution on close
require_checked_acs = false # require all AC boxes checked on close

[validate]
strict = false            # treat warnings as errors by default

[ready]
default_env = ""          # pre-filter frontier (corp/personal)

[new]
default_priority = "medium"  # default priority for new tickets
```

Manage with `tkt config --list` or `tkt config --set push.enabled=false`.

## Design

- **Push-to-claim** — a pushed commit is a claim; race detection on push rejection
- **Remote-aware** — scans `origin/main` via `git ls-tree` before allocating IDs
- **Surgical edits** — changes one field without disturbing the rest of the file
- **Single binary** — shells out to `git` for full SSH/HTTPS auth compatibility

Read commands (`ready`, `query`, `validate`) complete in ~50-100ms. Mutation commands (`new`, `claim`, `close`) take ~2s due to git fetch + push round-trips — set `push.enabled = false` in config to skip network I/O for local-only workflows.

## Agent Integration

For AI coding agents, add to your project's AGENTS.md:

```markdown
## Tickets

tkt ready                                         # what to work on next
tkt claim <id>                                    # mark as in_progress (shared repos)
tkt close <id> --check-all --resolution "..."     # mark done
tkt validate --brief                              # check for issues
```

Single-agent workflow: `tkt ready` → `tkt close <id> --check-all --resolution "..."`.
Shared-repo workflow: `tkt ready` → `tkt claim <id>` → work → `tkt close <id>`.

Set `CREW_ENV=corp` or `CREW_ENV=personal` to filter the frontier by ticket `env` field.

## Development

```bash
cargo build            # debug build
cargo test             # all tests
cargo clippy           # must be 0 warnings
cargo fmt --check      # must produce no diff
```

## Telemetry

Optional, **local-only** telemetry (disabled by default). No data leaves your machine. See [TELEMETRY.md](TELEMETRY.md) for details.

```bash
tkt telemetry --enable   # opt in
tkt telemetry --status   # see what's stored
tkt telemetry --disable  # opt out
```

Debug mode (no persistence): `TKT_DEBUG=1 tkt ready`

## Contributing

Found a bug? [File a bug report](https://github.com/smileynet/tkt/issues/new?template=bug_report.md). Want a feature? [Request it](https://github.com/smileynet/tkt/issues/new?template=feature_request.md).

## Inspired By

**[tk](https://github.com/nicholasgasior/tk)** — proved markdown files + frontmatter is the right model for lightweight work tracking. tkt adds dependency-graph frontier computation, push-to-claim race detection, and surgical frontmatter edits.

## License

MIT

# Tkt

A git-native ticket tracker where tasks are markdown files and `git push` is the claim protocol.

- **~50ms reads** — tickets are local files, not API calls
- **Single binary** — no runtime dependencies beyond `git`
- **Race-safe** — concurrent sessions get unique IDs automatically
- **Dependency graph** — only shows you what's actually unblocked
- **AI-agent friendly** — structured output, deterministic frontier
- **Zero config** — just `tkt new` in any git repo

## Quick Start

```bash
cargo install tkt

tkt new auth --title "Implement authentication"
# → ✓ created 01 auth (pushed)

tkt new api --title "Build API" --blocked-by 01
# → ✓ created 02 api (pushed)

tkt ready
# → Ready (1):
# →   01  Implement authentication

tkt close 01 --note "JWT + refresh tokens shipped"
# → ✓ closed 01 auth

tkt ready
# → Ready (1):
# →   02  Build API
```

Dependencies resolve automatically. Close a task and its dependents appear in the frontier.

## Install

### Pre-built binaries (fastest)

```bash
# macOS / Linux
curl -fsSL https://github.com/smileynet/tkt/releases/latest/download/tkt-installer.sh | sh

# Windows (PowerShell)
irm https://github.com/smileynet/tkt/releases/latest/download/tkt-installer.ps1 | iex
```

### From crates.io

```bash
cargo install tkt
```

### With cargo-binstall (pre-built, no compile)

```bash
cargo binstall tkt
```

### From source

```bash
git clone https://github.com/smileynet/tkt.git
cargo install --path tkt
```

### Verify

```bash
tkt --version
# → tkt 0.2.1 (ea047fb)
```

**Requirement:** `git` on PATH (any version).

## Usage

### See what's ready to work on

```bash
tkt ready                # human-friendly
tkt ready --json         # machine-readable (JSON Lines)
```

Shows open tasks with all dependencies satisfied, sorted by priority then ID.

### Create tasks

```bash
tkt new fix-login --title "Fix login timeout"
tkt new fix-login --title "Fix login timeout" --priority high
tkt new deploy --title "Deploy to staging" --blocked-by 01,02
tkt new spike --title "Research caching" --status backlog
tkt new train-model --title "Train ML model" --requires gpu,linux
```

Batch creation for related work:

```bash
tkt batch "api:Build API" "docs:Write docs" "tests:Add tests" --blocked-by 01
```

### Claim and close

```bash
tkt claim 03             # mark in-progress (visible to collaborators)
tkt close 03 --note "Deployed to prod"
tkt close 03 --check-all --evidence "All tests pass" --resolution "Shipped"
```

`claim` is optional for solo work — `close` works directly on open tasks. Use `claim` in shared repos so others see what's taken.

### Edit tasks

```bash
tkt edit 02 --title "New title"
tkt edit 02 --blocked-by 01,03
tkt edit 02 --priority high
tkt edit 02 --status backlog       # pull from frontier
```

### Tags & context

```bash
tkt context frontend         # set active context (auto-tags new tickets)
tkt new bugfix --title "Fix CSS" --tags frontend,urgent
tkt context --clear          # clear active context
```

Tags categorize tickets. The active context auto-applies tags to new tickets and can scope `tkt ready` output.

### Project health

```bash
tkt validate             # check for cycles, broken refs, contract issues
tkt validate --fix       # auto-repair fixable problems
tkt lint                 # normalize frontmatter style + blocked_by id refs
tkt lint --check         # CI mode: exit 1 if lint needed
tkt doctor               # full health check
tkt blocked              # show tasks stuck on dependencies
tkt sync-plan --check    # compare ticket status vs PLAN.md
```

### Query

```bash
tkt query                        # all tickets as JSON Lines
tkt query --status open          # filter by status
tkt query --priority high        # filter by priority
```

### Migrate from other tools

```bash
tkt migrate --detect          # detect current ticket format
tkt migrate --from tk         # convert tk-format tickets to tkt
```

## Task Format

```yaml
---
id: "01"
title: "Implement authentication"
status: open
blocked_by: []
priority: high
tags: [backend, auth]
requires: [gpu]
validation_criteria:
  - "JWT tokens issue correctly (test: auth_test::jwt_issue)"
---

## What to build
JWT-based auth with refresh token rotation.

## Acceptance criteria
- [ ] JWT tokens issued on login
- [ ] Refresh token rotation works
```

Tasks are just files in `.tickets/`. Edit them by hand anytime — tkt reads whatever's there. One exception: don't hand-edit `status` to `done`. Use `tkt close` so the acceptance-criteria, resolution, and evidence gates run (`tkt validate` flags done tickets closed by hand).

## Configuration

Optional. Create `.tickets/config.toml` to customize behavior:

```toml
[push]
enabled = true              # false for local-only repos (no network on writes)

[close]
require_checked_acs = true  # require acceptance criteria checked before close
require_resolution = false  # require --note when closing

[ready]
default_env = ""            # filter frontier by environment

[new]
default_priority = "medium" # default priority for new tasks

[machine]
capabilities = "gpu,linux"   # capabilities this workstation provides (filters tkt ready)
```

User-level defaults in `~/.config/tkt/config.toml`. Project config overrides.

```bash
tkt config --list          # show project config
tkt config --show          # show resolved config with sources
tkt config --set push.enabled=false
```

## AI Agent Integration

Works with **Kiro**, **OpenCode**, **Claude Code**, **Codex**, **Cursor**, **Copilot**, and **Windsurf**.

Add to your project's AGENTS.md (or use `tkt init --write` to inject automatically):

```markdown
tkt ready              # see what's unblocked
tkt claim <id>         # mark in-progress
tkt close <id> --check-all --resolution "what was done"
```

Deploy agent instructions for all supported tools at once:

```bash
tkt init --all         # writes AGENTS.md, CLAUDE.md, .cursor/rules/, .kiro/steering/, etc.
```

Solo agent: `tkt ready` → pick first → `tkt close <id> --check-all --resolution "..."`.

Multi-agent: `tkt ready` → `tkt claim <id>` → work → `tkt close <id>`.

Structured output everywhere: `tkt ready --json`, `tkt query`, `tkt validate --brief`.

## Environment Variables

| Variable | Effect |
|----------|--------|
| `CREW_ENV` | Filter frontier by environment (legacy; prefer `machine.capabilities` config for new projects) |
| `TKT_ASCII=1` | ASCII-only symbols (✓→\[ok\], ✗→\[err\]) |
| `NO_COLOR=1` | Disable ANSI color |
| `TKT_DEBUG=1` | Debug output to stderr |

## Contributing

Found a bug? [File a report](https://github.com/smileynet/tkt/issues/new?template=bug_report.md).
Want a feature? [Request it](https://github.com/smileynet/tkt/issues/new?template=feature_request.md).

## License

[MIT](LICENSE)

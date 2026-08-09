# tkt

Track work as markdown files in git. Dependency-aware frontier, atomic push-to-claim, race detection — no server required.

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
# Build and install
cargo install --path .

# Create the tickets directory
mkdir .tickets && git add .tickets && git commit -m "init tickets"

# Create your first ticket
tkt new auth-system --title "Implement authentication"
# → allocated 01-auth-system.md (pushed — id claimed, status: open)

# See what's ready to work on
tkt ready
# → 01  Implement authentication

# Claim it (marks in_progress, pushes to remote)
tkt claim 01
# → claimed 01-auth-system.md (in_progress pushed)

# Close it when done
tkt close 01 --note "JWT + refresh tokens shipped"
# → closed 01-auth-system.md (done pushed)
```

## Install

```bash
# From source (requires Rust toolchain)
cargo install --path .

# Or build a release binary directly
cargo build --release
# Binary at target/release/tkt

# Verify
tkt --version
# → tkt 0.1.0
```

Single binary, no runtime dependencies beyond `git`.

## Usage

### Frontier — what's ready

```bash
tkt ready              # human output
tkt ready --json       # JSON Lines (one object per ticket)
```

Shows open tickets whose dependencies are all done, filtered by `CREW_ENV` if set, sorted by priority then ID.

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

**Note:** `claim` is optional. `close` works directly on `open` tickets — useful for single-agent workflows where the push round-trip adds latency without value. Use `claim` in shared repos to signal WIP and detect races.

### Edit and maintain

```bash
tkt edit 02 --title "New title" --blocked-by 01,03 --priority high
tkt renumber 05 02     # reassign ID (birth-window only)
tkt validate           # check for cycles, dangling deps, contract violations
tkt validate --strict  # promote warnings to errors
tkt sync-plan --check  # compare ticket status vs docs/plan.md table
tkt query              # dump full corpus as JSON Lines
```

### Common flags

| Flag | Used by | Effect |
|------|---------|--------|
| `--json` | ready | machine-readable output |
| `--strict` | validate, sync-plan | warnings become errors |
| `--brief` | validate, sync-plan | human output instead of JSON |
| `--blocked-by N,N` | new, batch, edit | set dependencies |
| `--priority high` | new, batch, edit | jump frontier order |
| `--env E` | new, batch, edit | corp / personal / either |
| `--note "..."` | close | resolution text |
| `--ac N,N` | close, edit | check acceptance criteria boxes |

## Ticket Format

```yaml
---
id: "01"
title: "Implement authentication"
status: open          # open | in_progress | done
blocked_by: []        # ids that must be done first
priority: high        # optional: jumps frontier order
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

## Design

- **Push-to-claim** — a pushed commit is a claim; race detection on push rejection
- **Remote-aware** — scans `origin/main` via `git ls-tree` before allocating IDs
- **Surgical edits** — changes one field without disturbing the rest of the file
- **Single binary** — shells out to `git` for full SSH/HTTPS auth compatibility
- **Worktree-aware** — works from git worktrees (`.tickets/` is part of the checked-out tree)

### Expected latency

Read commands (`ready`, `query`, `validate`) complete in ~50-100ms. Mutation commands (`new`, `claim`, `close`, `edit`) take ~2s because they include a git fetch + push round-trip — this is the cost of atomic remote operations and push-to-claim semantics. For local-only workflows, set `push.enabled = false` in `.tickets/config.toml` to skip network I/O.

### Spike branches

When closing a ticket from a `spike/*` branch, tkt auto-appends "Spike branch: spike/name" to the resolution. This documents which experimental branch validated the work.

## Agent Integration

For AI coding agents (kiro-cli, codex, etc.), add this to your project's AGENTS.md:

```markdown
## Tickets

tkt ready                                         # what to work on next
tkt claim <id>                                    # mark as in_progress (shared repos)
tkt close <id> --check-all --resolution "..."     # mark done
tkt validate --brief                              # check for issues
tkt capabilities                                  # machine-readable feature manifest
```

For single-agent workflows, `claim` is optional — `close` works directly on open tickets.

Machine-readable discovery: `tkt capabilities` outputs a JSON manifest of commands, flags, and workflows.

## Development

```bash
cargo build            # debug build
cargo test             # 96 tests (48 unit + 48 integration)
cargo clippy           # must be 0 warnings
cargo fmt --check      # must produce no diff
```

## Telemetry

tkt includes optional, **local-only** telemetry (disabled by default). No data leaves your machine. See [TELEMETRY.md](TELEMETRY.md) for full details on what's collected, where it's stored, and how to opt in/out.

```bash
tkt telemetry --enable   # opt in
tkt telemetry --status   # see what's stored
tkt telemetry --show     # print recent events
tkt telemetry --disable  # opt out
tkt telemetry --clear    # delete all local data
```

### Debug mode

For real-time diagnostics without persisting anything:

```bash
TKT_DEBUG=1 tkt ready        # human-readable trace to stderr
TKT_DEBUG=json tkt ready     # JSONL trace to stderr
```

## Contributing

Found a bug? [File a bug report](https://github.com/smileynet/tkt/issues/new?template=bug_report.md). Want a feature? [Request it](https://github.com/smileynet/tkt/issues/new?template=feature_request.md).

## Inspired By

**[tk](https://github.com/nicholasgasior/tk)** — proved markdown files + frontmatter is the right model for lightweight work tracking. tkt adds dependency-graph frontier computation, push-to-claim race detection, and surgical frontmatter edits.

## License

MIT

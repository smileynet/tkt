# tkt

Git-native ticket CLI. Track work in `.tickets/` markdown files with YAML frontmatter, managed by atomic git operations.

## What It Does

```
.tickets/
├── 01-auth-system.md        # status: done
├── 02-api-endpoints.md      # status: open, blocked_by: [01]
└── 03-deploy-pipeline.md    # status: open, blocked_by: [02]
```

tkt computes the frontier (what's unblocked), claims work atomically (git push = claim), and detects races when two sessions allocate the same id.

## Install

```bash
# From source
cargo install --path .

# Or build locally
cargo build --release
```

## Quick Start

```bash
# Show what's ready to work on
tkt ready

# Create a ticket
tkt new auth-system --title "Implement authentication"

# Claim it (marks in_progress, pushes)
tkt claim 01

# Close it when done
tkt close 01 --note "JWT + refresh tokens implemented"

# Dump all tickets as JSON
tkt query

# Check if plan.md drifted from ticket state
tkt sync-plan --check
```

## Commands

| Command | What it does |
|---------|-------------|
| `tkt ready [--json]` | Show frontier (open tickets with all deps done) |
| `tkt new <slug>` | Allocate next id, commit, push (id is yours) |
| `tkt batch <slugs...>` | Create N tickets in one commit |
| `tkt claim <id>` | Mark in_progress, push |
| `tkt close <id>` | Mark done, append resolution |
| `tkt edit <id>` | Change fields (title, blocked_by, priority, env, spec, ac) |
| `tkt renumber <old> <new>` | Move to a new id (birth-window only) |
| `tkt query` | Dump full corpus as JSON Lines |
| `tkt sync-plan --check` | Report drift between tickets and plan.md |
| `tkt sync-plan --fix` | Fix derivable columns (status) in plan.md |
| `tkt validate` | Check contract health (dangling refs, cycles, decay) |

### Common flags

- `--title`, `--spec`, `--env`, `--priority`, `--blocked-by` — metadata for new/batch/edit
- `--json` — machine-readable output (ready)
- `--strict` — promote warnings to errors (validate, sync-plan)
- `--brief` — human-readable output instead of JSON (validate, sync-plan)
- `--note` — resolution note (close)
- `--ac N,N` — mark acceptance criteria checked (close, edit)

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

Describe the work...

## Acceptance criteria

- [ ] JWT tokens issued on login
- [ ] Refresh token rotation
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Domain failure (not found, conflict, validation drift) |
| 2 | Operational crash (I/O, git, parse error) |

## Design

- **Files are the database** — `.tickets/` is git-native, hand-editable, tool-optional
- **Push-to-claim** — a pushed commit is a claim; race detection on push rejection
- **Frontier computation** — filters open tickets whose dependencies are all done, sorted by priority then id
- **Surgical edits** — changes one field without disturbing the rest of the file
- **Remote-aware allocation** — scans both local and remote filenames to prevent ID collisions
- **Single binary** — no runtime dependencies beyond `git` on PATH

## Inspired By

**[tk](https://github.com/nicholasgasior/tk)** — The original git-native ticket tool that proved markdown files + frontmatter is the right model for lightweight work tracking. tkt builds on tk's insight that files are the database, adding dependency-graph frontier computation, atomic push-to-claim race detection, and surgical frontmatter edits that preserve unknown fields.

## License

MIT

---
name: tkt
description: "Track tasks as markdown files in your git repo. Use when managing tickets, checking what's ready, claiming work, closing tasks, creating tickets, decomposing work, or validating project health."
compatibility: Requires git
triggers:
  - tkt
  - tickets
  - what to work on
  - what's ready
  - frontier
  - claim
  - close ticket
  - create ticket
  - validate tickets
  - decompose
  - break into tasks
  - work breakdown
  - task graph
  - ticket format
  - blocked by
  - backlog
  - park this
  - defer
tools:
  - shell
---

# tkt

Track tasks as markdown files in your git repo. One file per ticket in `.tickets/`, with status and dependencies in YAML frontmatter. tkt computes what's ready to work on next.

## Jobs to be done

| When I'm... | I want to... | Command |
|---|---|---|
| Starting a work session | see what's unblocked and prioritized | `tkt ready` |
| Planning future work | create tickets without them cluttering the frontier | `tkt new <slug> --title "..." --status backlog` |
| Ready to start something | promote a backlogged ticket to actionable | `tkt edit <id> --status open` |
| Grabbing a task in a shared repo | signal that I'm working on it | `tkt claim <id>` |
| Finishing a task | prove it's done with evidence | `tkt close <id> --check-all --evidence "..."` |
| Discovering work during implementation | capture it as a separate ticket without losing focus | `tkt new <slug> --title "..."` (open — or `--status backlog` only if genuinely deferred) |
| Checking project health | find cycles, broken deps, stale WIP | `tkt validate` / `tkt audit` |
| Querying the full corpus | filter by status or priority | `tkt query --status open --priority high` |
| Working in a specific environment | see only relevant work | Set `machine.capabilities` in config (or `CREW_ENV=corp` for legacy env filtering) → `tkt ready` filters by `requires` |
| Keeping a plan in sync | report drift between tickets and plan doc (advisory) | `tkt sync-plan` (add `--check` to gate CI) |
| Scoping work to a subsystem | filter frontier by tags | `tkt context backend` → `tkt ready` shows only matching |
| Returning to unscoped view | clear tag filter | `tkt context --clear` |
| Importing from another tool | convert foreign tickets | `tkt migrate --from tk` |

For full flags and options, see [commands.md](references/commands.md).

---

## Status lifecycle

```
backlog → open → in_progress → done
```

| Status | Meaning | Appears in `tkt ready`? |
|--------|---------|------------------------|
| `backlog` | Parked — not prioritized, not actionable yet | **No** |
| `open` | Available for work — dependencies satisfied = frontier | **Yes** (if deps done) |
| `in_progress` | Claimed, actively being worked | **No** |
| `done` | Completed, verified | **No** |

**Key rule:** `tkt ready` only shows `open` tickets whose `blocked_by` are all `done`. Backlogged tickets are invisible to the frontier — that's the point.

---

## Backlog workflow

Backlog is for work you've identified but are deliberately **deferring out of the current cycle**. Default new work to `open` — backlog is the exception, not the reflex.

**When NOT to backlog:** if the work could realistically be picked up once its blockers clear, it's `open` (frontier-eligible), not backlog. Discovered work found mid-task is usually actionable — create it as `open`, or `--blocked-by` if it depends on other tickets. Parking actionable work in backlog hides it from `tkt ready` and it gets forgotten. A growing backlog is a smell, not a safety net.

Legitimate uses of backlog:

- **Park future ideas** you have no plan to start yet (someday-maybe)
- **Defer low-priority items** that aren't worth doing this cycle
- **Hold work pending a decision** that hasn't been made

The test: *could this realistically be worked next once its blockers are done?* If yes → `open`. Only if it's genuinely speculative or deferred → `backlog`.

### Creating backlogged tickets

```bash
tkt new caching --title "Add response caching" --status backlog
tkt batch "cache:Response caching" "metrics:Add metrics" --status backlog
```

Default (no `--status` flag) creates as `open`.

### Promoting / demoting

```bash
tkt edit 15 --status open           # promote → now appears in tkt ready
tkt edit 07 --status backlog        # demote → disappears from frontier
```

### When to use backlog vs blocked_by

| Situation | Use |
|-----------|-----|
| "We'll do this someday, not now" | `--status backlog` |
| "We'll do this after ticket 03 is done" | `--blocked-by 03` (status stays `open`) |
| "We identified this but it's not scoped yet" | `--status backlog` |
| "This is ready but depends on another ticket" | `--blocked-by N` |

---

## Working tickets

### Solo workflow

```bash
tkt ready → pick first → work → tkt close <id> --check-all --resolution "what was done"
```

### Shared repo (multiple agents)

```bash
tkt ready → tkt claim <id> → work → tkt close <id> --check-all --resolution "what was done"
```

If `tkt claim` push is rejected → someone else got there first → pick next ticket.

---

## Closing tickets (quality gate)

Closure is a verification event, not a status flip. The close command enforces quality. Never edit a ticket file to set `status: done` — that bypasses the AC/resolution/evidence gates and the push protocol. Always use `tkt close`; `tkt validate` flags done tickets with no Resolution section.

```bash
# Standard close with evidence
tkt close 03 --check-all --resolution "JWT auth with refresh tokens" \
  --evidence "56 tests pass" --evidence "POST /login returns 200 with token"

# Force close (bypasses AC checks — use sparingly, justify)
tkt close 03 --force --resolution "Superseded by ticket 12"

# Migration close (work moved elsewhere — don't game ACs)
tkt close 05 --force --resolution "Migrated to other-project #22"
```

Project config controls what's required at close time (resolution, checked ACs, validation evidence). See enforcement config in [commands.md](references/commands.md).

---

## Validation criteria and evidence

Machine-enforceable "definition of done":

```bash
# Create with criteria
tkt new deploy --title "Deploy to staging" \
  --vc "staging URL returns 200" --vc "smoke tests pass"

# Close with positional evidence (matches criteria order)
tkt close 05 --check-all \
  --evidence "curl staging.example.com → 200 OK" \
  --evidence "make smoke → 4/4 passed" \
  --resolution "Deployed via CI pipeline"
```

---

## Health checks

| Command | Job |
|---------|-----|
| `tkt validate` | Contract integrity — cycles, broken deps, invalid fields |
| `tkt audit` | Closure quality — unchecked ACs, TBD resolutions, stale WIP |
| `tkt sync-plan` | Plan drift — ticket status vs plan document (advisory; `--check` gates CI) |
| `tkt doctor` | Setup verification — current project or multi-project scan |
| `tkt lint --check` | Style — frontmatter quoting and field order |

Use `--strict` to treat warnings as errors, `--brief` for short output, `--fix` to auto-repair (validate and sync-plan). Full options in [commands.md](references/commands.md).

---

## Key behaviors

- **Push on write:** All mutations push to remote immediately. Disable with `push.enabled = false` in `.tickets/config.toml`.
- **Race detection:** Push rejected → tkt retries with new ID (new/batch) or reports the conflict.
- **Priority sort:** `tkt ready` sorts by priority bucket (urgent > high > medium > low) then by ID.
- **Surgical edits:** `tkt edit` only touches the field you specify — everything else preserved.
- **Env filtering (legacy):** Tickets with `env: corp` only appear in `tkt ready` when `CREW_ENV=corp`. No env field = visible everywhere.
- **Requires filtering:** Tickets with `requires: [gpu, linux]` only appear if `machine.capabilities` config includes all listed values. If a ticket has `env` but no `requires`, requires is synthesized from env for backward compat. Prefer `requires` over `env` for new tickets.
- **Tag context:** `tkt context backend api` sets a session-scoped filter. `ready`, `query`, and `blocked` only show tickets whose `tags` include all context-include tags and none of the context-exclude tags. `new` and `batch` auto-inherit context tags.

---

## Creating tickets (content quality)

Before writing ticket body content, read [ticket-standards.md](references/ticket-standards.md). Every ticket needs:

1. Intent source (why does this exist?)
2. Context for a fresh agent (files to read, decisions made)
3. Behavioral outcome (what changes for the user)
4. Testable validation (acceptance criteria an agent can verify)

Set `--tags <stream>` at creation in multi-stream projects (one tag is the norm; omit in single-stream projects) — retro-tagging rarely happens, and tags are what scope `tkt ready`/`tkt context` to a subsystem.

---

## References

- [commands.md](references/commands.md) — full command reference, all flags, enforcement config
- [ticket-format.md](references/ticket-format.md) — file format, required/optional fields
- [ticket-standards.md](references/ticket-standards.md) — quality gate for ticket content
- [wide-refactors.md](references/wide-refactors.md) — expand-contract sequencing
- [migration-assist.md](references/migration-assist.md) — converting foreign ticket schemas

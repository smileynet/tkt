---
title: tkt
description: "Track tasks as markdown files in your git repo. Use when managing tickets, checking what's ready, claiming work, closing tasks, or validating project health."
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
tools:
  - shell
---

# tkt

Track tasks as markdown files in your git repo.

## When to use

- Starting a work session: see what's unblocked
- Creating tasks with dependencies
- Claiming work in shared repos
- Closing tasks with evidence of completion
- Checking project health

## Commands

```bash
tkt ready                    # show unblocked tasks, sorted by priority
tkt ready --json             # machine-readable output
tkt new <slug> --title "..." # create a task (pushes immediately)
tkt claim <id>               # mark in-progress (shared repos)
tkt close <id> --note "..."  # mark done, append resolution
tkt edit <id> --priority high --blocked-by 01,02
tkt validate                 # check for cycles, broken deps, issues
tkt validate --fix           # auto-repair fixable problems
tkt doctor                   # verify setup is correct
tkt --dry-run <command>      # preview what would happen
```

## Workflows

### Solo agent

```
tkt ready → pick first → do the work → tkt close <id> --check-all --resolution "what was done"
```

### Shared repo (multiple agents)

```
tkt ready → tkt claim <id> → do the work → tkt close <id> --check-all --resolution "what was done"
```

If a claim push is rejected, someone else got there first — pick the next ticket.

### With validation criteria

```bash
# Create with criteria
tkt new auth --title "Implement auth" --validation "tests pass" --validation "login works"

# Close with evidence
tkt close 01 --evidence "49 passed, 0 failed" --evidence "POST /login returns JWT" --resolution "Done"
```

## Key behaviors

- `tkt ready` sorts by priority (urgent > high > medium > low) then by ID
- Tasks with unsatisfied `blocked_by` dependencies don't appear in ready
- `--dry-run` on any mutation shows what would happen without writing
- All writes push to remote immediately (disable with `push.enabled = false` in config)
- `--force` bypasses validation gates when you need to override

## Task format

```yaml
---
id: "01"
title: "Implement authentication"
status: open              # backlog | open | in_progress | done
blocked_by: []            # IDs that must be done first
priority: high            # optional: urgent > high > medium > low
validation_criteria:      # optional: what "done" means
  - "tests pass"
  - "login works"
---
```

Tasks are markdown files in `.tickets/`. Edit by hand anytime.

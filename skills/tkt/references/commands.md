# tkt Command Reference

## Task lifecycle

| Command | Effect |
|---------|--------|
| `tkt new <slug> --title "..."` | Create task, assign ID, push |
| `tkt batch "slug:title" ...` | Create multiple tasks in one push |
| `tkt claim <id>` | Mark in-progress (push = visible WIP) |
| `tkt close <id> --resolution "..."` | Mark done, write resolution |
| `tkt edit <id> [--title] [--priority] [--blocked-by] [--status]` | Surgical field change |

## Viewing

| Command | Effect |
|---------|--------|
| `tkt ready` | Unblocked tasks sorted by priority |
| `tkt ready --json` | JSON Lines output |
| `tkt blocked` | Tasks with unsatisfied dependencies |
| `tkt query` | All tasks as JSON Lines |
| `tkt query --status open --priority high` | Filtered view |

## Health and maintenance

| Command | Effect |
|---------|--------|
| `tkt validate` | Contract integrity: cycles, broken deps, invalid fields, decay |
| `tkt validate --strict` | Warnings become errors |
| `tkt validate --brief` | Human-readable summary |
| `tkt validate --fix` | Auto-repair fixable problems (quoting, field cleanup) |
| `tkt validate --fix --dry-run` | Preview what --fix would change |
| `tkt audit` | Closure quality: unchecked ACs, TBD resolutions, stale WIP |
| `tkt audit --strict` | Treat warnings as errors |
| `tkt audit --brief` | Short summary |
| `tkt sync-plan --check [PLAN]` | Compare tickets vs plan document (default: docs/plan.md) |
| `tkt sync-plan --fix [PLAN]` | Update derivable columns in the plan |
| `tkt lint` | Normalize frontmatter style (quoting, field order) |
| `tkt lint --check` | CI mode: exit 1 if anything would change |
| `tkt lint 03 07` | Lint specific tickets only |
| `tkt doctor` | Verify current project setup |
| `tkt doctor <path>` | Scan all projects, flag issues |
| `tkt doctor --strict` | Strict mode |

## Other commands

| Command | Job | When to use |
|---------|-----|-------------|
| `tkt renumber <old> <new>` | Move ticket to new ID | Only during birth window (before ID is cited elsewhere) |
| `tkt rebase` | Resolve ID collisions with upstream | After `git pull` introduces conflicts |
| `tkt config --list` | Show project configuration | Checking what gates are active |
| `tkt config --set key=value` | Set config value | Changing close requirements, push behavior |
| `tkt config --show` | Show resolved values with sources | Debugging config cascade |
| `tkt telemetry --status` | Check telemetry state | Privacy audit |
| `tkt telemetry --enable` / `--disable` | Toggle telemetry consent | Opt in/out |
| `tkt capabilities` | JSON feature manifest | Agent/automation discovery |
| `tkt init` | Scaffold `.tickets/` | New project setup |

## Creation flags (new, batch)

| Flag | Effect | Default |
|------|--------|---------|
| `--title "..."` | Human-readable title | Required |
| `--status S` | `backlog`, `open`, `in_progress`, `done` | `open` |
| `--priority P` | `urgent`, `high`, `medium`, `low` | `medium` |
| `--blocked-by N,N` | IDs that must be done first | none |
| `--env E` | Environment filter (`corp`, `personal`) | none |
| `--spec S` | Originating spec slug | none |
| `--validation "..."` | Validation criteria (repeatable). Alias: `--vc` | none |

## Edit flags

| Flag | Effect |
|------|--------|
| `--title "..."` | Change title |
| `--status S` | Change status (`backlog`, `open`, `in_progress`, `done`) |
| `--priority P` | Change priority (pass `''` to clear) |
| `--blocked-by N,N` | Replace dependencies |
| `--env E` | Set environment (pass `''` to clear) |
| `--spec S` | Set spec link (pass `''` to clear) |
| `--ac N,N` | Check acceptance criteria boxes (1-based) |
| `--validation "..."` | Replace validation criteria list. Alias: `--vc` |

## Close flags

| Flag | Effect |
|------|--------|
| `--resolution "..."` | What was done (alias: `--note`) |
| `--check-all` | Check all acceptance criteria boxes |
| `--ac 1,2` | Check specific AC boxes (1-based) |
| `--evidence "..."` | Proof per validation criterion (repeatable, positional) |
| `--force` | Close even with unchecked ACs |

## Global flags (all commands)

| Flag | Effect |
|------|--------|
| `--dry-run` | Preview mutations without writing |
| `-q` / `--quiet` | Suppress confirmations, emit only essential data |
| `-o json` | Structured JSON output (data to stdout, errors to stderr) |
| `--color always\|never\|auto` | Control ANSI color (default: auto) |

## Enforcement config (.tickets/config.toml)

These settings control what `tkt close` requires:

| Section.Key | Values | Effect |
|-------------|--------|--------|
| `close.require_resolution` | `true` / `false` | Must provide `--resolution` or `--note` |
| `close.require_checked_acs` | `true` / `false` | All ACs must be checked (default: true) |
| `close.require_validation_criteria` | `true` / `false` | Ticket must have `validation_criteria` field |
| `close.require_validation_evidence` | `"true"` / `"warn"` / `"false"` | Must provide `--evidence` per criterion |
| `close.allow_force` | `true` / `false` | Whether `--force` escape hatch is allowed |
| `push.enabled` | `true` / `false` | Mutations push to remote (default: true) |
| `validate.strict` | `true` / `false` | Warnings become errors |
| `ready.default_env` | string | Default CREW_ENV filter for `tkt ready` |
| `new.default_priority` | priority enum | Default priority for new tickets |

Manage with `tkt config --set close.require_resolution=true` or edit `.tickets/config.toml` directly.

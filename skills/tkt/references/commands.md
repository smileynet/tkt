# tkt Command Reference

## Task lifecycle

| Command | Effect |
|---------|--------|
| `tkt new <slug> --title "..."` | Create task, assign ID, push |
| `tkt batch "slug:title" ...` | Create multiple tasks in one push |
| `tkt claim <id>` | Mark in-progress (push = visible WIP) |
| `tkt close <id> --note "..."` | Mark done, write resolution |
| `tkt edit <id> [--title] [--priority] [--blocked-by]` | Change fields |

## Viewing

| Command | Effect |
|---------|--------|
| `tkt ready` | Unblocked tasks sorted by priority |
| `tkt ready --json` | JSON Lines output |
| `tkt blocked` | Tasks with unsatisfied dependencies |
| `tkt query` | All tasks as JSON Lines |
| `tkt query --status open --priority high` | Filtered view |

## Maintenance

| Command | Effect |
|---------|--------|
| `tkt validate` | Check for cycles, broken deps, contract issues |
| `tkt validate --fix` | Auto-repair fixable problems |
| `tkt audit` | Closure quality (missing resolutions, unchecked ACs) |
| `tkt sync-plan --check` | Compare tickets vs plan document |
| `tkt doctor` | Verify setup is correct |
| `tkt doctor <path>` | Scan multiple projects |

## Creation flags

| Flag | Effect |
|------|--------|
| `--title "..."` | Task title (required for new) |
| `--blocked-by 01,02` | Dependencies |
| `--priority high` | urgent, high, medium (default), low |
| `--validation "..."` | Validation criteria (repeatable) |
| `--env corp` | Environment filter |

## Close flags

| Flag | Effect |
|------|--------|
| `--note "..."` | Resolution text |
| `--evidence "..."` | Proof per validation criterion (repeatable) |
| `--ac 1,2` | Check specific acceptance criteria |
| `--check-all` | Check all acceptance criteria |
| `--force` | Close even with unmet gates |

## Global flags

| Flag | Effect |
|------|--------|
| `--dry-run` | Show what would happen without writing |
| `--quiet` | Suppress confirmations |
| `--color always\|never\|auto` | Control color output |

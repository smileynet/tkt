# tkt Command Reference

For the complete machine-readable command/flag inventory, use `tkt capabilities
-o json`. See [manifest fields and semantics](capabilities.md) for schemas,
aliases, mutation modes and default output formats.

Top-level `tkt --help` points to that inventory. Normal `tkt init` output also
recommends `tkt init --write` and separate workflow skill/plugin installation.
`init --agent-only` keeps its output to the snippet; quiet writes omit setup chatter.

## Agent eligibility

Identify each invocation with global `--agent <lowercase-slug>` or `TKT_AGENT`; the flag wins and identity is never stored in config. `ready` and `claim` apply ticket agent eligibility. Missing identity sees unrestricted work only; `ready` reports hidden restricted work on stderr. Use `claim <id> --force` only as an explicit scheduling override.

Claim records optional `assignee` ownership separately from eligibility. Other
owners block claim unless forced; ready hides them. Unidentified ready hides any
owned ticket, and an anonymous forced claim preserves the existing owner.

Create or replace eligibility with mutually exclusive `--agents codex,claude` or `--exclude-agents antigravity` on `new`, `batch`, and `edit`; `edit --agents ''` or `edit --exclude-agents ''` clears it. Frontmatter stores one optional signed `agents` list: `["codex", "claude"]` allows either, `["!antigravity"]` excludes that agent, omission allows everyone. Lists must be nonempty, homogeneous in sign, duplicate-free, and contain lowercase alphanumeric/hyphen slugs. Serialization sorts names. `ready` human/JSON and `query` JSON expose the policy (`agents: []` in JSON means unrestricted). `query` honors tag context but never filters by current agent. Tags scope work streams; `requires` remains all-of machine capabilities; eligibility is separate from ownership after claim.

## Task lifecycle

| Command | Effect |
|---------|--------|
| `tkt new <slug> --title "..." [--tags STREAM]` | Create task (defaults to `open`), assign ID, push |
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
| `tkt query` | Context-scoped tasks as JSON Lines; stderr notice when context narrows results |
| `tkt query --all-context` | Bypass tag context without changing it; other explicit filters still apply |
| `tkt query --status open --priority high` | Filtered view |

Query emits typed `id`, `title`, `status`, `blocked_by`, `pending_blockers`, `tags`,
`validation_criteria`, `requires`, `agents` and `assignee`. Optional `env`, `priority`,
`spec` and `parent` retain their stored values. Unknown or unparsed stored keys appear in
`frontmatter` as raw strings rather than guessed YAML types. `blocked_by` preserves
history; `pending_blockers` includes only missing or non-done prerequisites,
including cancelled ones. Ready JSON also includes `pending_blockers` (empty for
eligible frontier rows). Human frontier output does not list completed blockers.

Query keeps tag context by default. When context actually excludes matching rows,
its notice goes to stderr (a `context_filter` JSONL warning under `-o json`), never
ticket stdout. `--all-context` bypasses context; status/priority/parent filters still apply.

## Health and maintenance

| Command | Effect |
|---------|--------|
| `tkt validate` | Contract integrity: cycles, broken deps, invalid fields, decay |
| `tkt validate --strict` | Warnings become errors |
| `tkt validate --brief` | Human-readable summary |
| `tkt validate --fix` | Auto-repair fixable problems (quoting, field cleanup, blocked_by ref normalization); aborts if a repair would introduce a new finding |
| `tkt validate --fix --dry-run` | Preview what --fix would change |
| `tkt audit` | Closure quality: unchecked ACs, TBD resolutions, stale WIP |
| `tkt audit --strict` | Treat warnings as errors |
| `tkt audit --brief` | Short summary |
| `tkt sync-plan [PLAN]` | Report ticket-vs-plan drift (advisory, exit 0; default: docs/plan.md) |
| `tkt sync-plan --check [PLAN]` | CI gate: exit 1 on drift |
| `tkt sync-plan --fix [--dry-run] [PLAN]` | Update status cells and append missing tickets under Unplanned tickets (`--dry-run` previews) |
| `tkt lint` | Normalize frontmatter style and blocked_by refs; backfill missing IDs from valid numeric-prefix filenames when the remaining ticket is valid |
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
| `tkt config --list` | Show user configuration | Checking user defaults |
| `tkt config --set key=value` | Set config value | Changing close requirements, push behavior |
| `tkt config --show` | Show resolved project values with sources | Debugging config cascade |
| `tkt telemetry --status` | Check telemetry state | Privacy audit |
| `tkt telemetry --enable` / `--disable` | Toggle telemetry consent | Opt in/out |
| `tkt capabilities` | JSON feature manifest | Agent/automation discovery |
| `tkt init` | Initialize `.tickets/` and deploy agent integration files | New project setup |
| `tkt init --write` | Write agent snippet into AGENTS.md using markers | Embedding instructions |
| `tkt init --write FILE` | Write snippet to a specific file | Custom agent file |
| `tkt init --agent-only` | Output agent snippet only (skip dir/config creation) | Existing projects |

## Context (tag scoping)

| Command | Effect |
|---------|--------|
| `tkt context` | Show active context (include/exclude tags) |
| `tkt context backend api` | Set context: include backend + api tags |
| `tkt context +backend -frontend` | Include backend, exclude frontend |
| `tkt context --clear` | Clear active context |

When a context is active:
- `tkt new` and `tkt batch` auto-apply include-tags to new tickets
- `tkt ready`, `tkt query`, and `tkt blocked` filter results to matching tickets
- Tags use AND logic: ticket must have ALL include-tags and NONE of the exclude-tags

## Migration

| Command | Effect |
|---------|--------|
| `tkt migrate --detect` | Report detected ticket format and confidence |
| `tkt migrate --from tk --dry-run` | Preview migration without writing |
| `tkt migrate --from tk --confirm` | Apply the reviewed tk-format migration |

## Creation flags (new, batch)

| Flag | Effect | Default |
|------|--------|---------|
| `--title "..."` | Human-readable title | Required |
| `--body TEXT\|@FILE\|-` | Full body text, UTF-8 file, or stdin (`new` only); conflicts with template | prompted default |
| `--template NAME` | Verbatim body from `.tickets/templates/NAME.md` (new/batch) | prompted default |
| `--status S` | `open` (default), `backlog` (deferred out of cycle), `in_progress`; `done` is rejected (use close) | `open` |
| `--priority P` | `urgent`, `high`, `medium`, `low` | `medium` |
| `--blocked-by N,N` | IDs that must be done first | none |
| `--parent ID` | Existing parent ticket (hierarchy, independent of prerequisites) | none |
| `--env E` | Environment filter (`corp`, `personal`) | none |
| `--spec S` | Originating spec slug | none |
| `--tags T,T` | Work-stream / categorization tags; also inherited from active context on creation | none |
| `--validation "..."` | Validation criteria (repeatable). Alias: `--vc` | none |
| `--requires R,R` | Machine capabilities required (e.g., `gpu,linux,corp`) | none |

**At creation:** supply a complete body with `--body @file`, `--body -`, literal
text, or a named template. An initial `@` means a file; use stdin/file input for
literal text starting with `@`. Bodies/templates replace the entire body verbatim;
templates do not interpolate fields. Fill the default prompts before validating
open work; unfilled prompts still trigger `stub-body`. Do not park actionable work
to hide that warning.

## Edit flags

| Flag | Effect |
|------|--------|
| `--title "..."` | Change title |
| `--status S` | Change status (`open`, `backlog`, `in_progress`). `done` is rejected; use `tkt close` so closure gates run |
| `--priority P` | Change priority (pass `''` to clear) |
| `--blocked-by N,N` | Replace dependencies |
| `--parent ID` | Set parent; pass `''` to clear. Missing, self and cyclic parents are rejected |
| `--tags T,T` | Replace tags (repeatable); pass `''` to clear |
| `--add-tag T` | Add unique tags after replacement (repeatable) |
| `--remove-tag T` | Remove exact tags last (repeatable; removals win) |
| `--env E` | Set environment (pass `''` to clear) |
| `--spec S` | Set spec link (pass `''` to clear) |
| `--ac N,N` | Check acceptance criteria boxes (1-based) |
| `--validation "..."` | Replace validation criteria list. Alias: `--vc` |

**Dependency edits replace:** `--blocked-by` replaces the entire list. To add a
blocker, read the current IDs, combine them with the addition, and pass the full set.
There are no `--add-blocked-by` or `--remove-blocked-by` flags.

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
| `--no-push` | Commit locally without fetching/pushing for this invocation |
| `-q` / `--quiet` | Suppress confirmations, emit only essential data |
| `-o json` | Structured JSON output (data to stdout, errors to stderr) |
| `--color always\|never\|auto` | Control ANSI color (default: auto) |

## Reference diagnostics

On collision recovery, `tkt rebase` reports possible body citations with path,
line and old/new IDs; it never rewrites prose. Review those candidates manually:
the old ID may refer to an upstream ticket rather than the renumbered local one.
Dry-run reports them without writes. Validation warns on missing unqualified
local `#NNN` / `ticket NNN` references. Use `owner/repo#NNN` or external links for
foreign issues; code/fences, URLs and headings are ignored.

## Retiring work

`tkt cancel ID --reason duplicate|superseded|obsolete|wontfix --note "why"`
records cancelled, not done. Duplicate/superseded require `--by ID`, naming an
existing non-cancelled replacement. Active dependents require `--repoint` (with
--by) or `--drop-edges`; every rewritten ticket is named. Terminal dependents'
obsolete prerequisite edges are dropped and named automatically, preserving
their bodies and closure evidence. Use dry-run to inspect changes first.

`tkt reopen ID` returns only cancelled work to open and preserves Cancellation
history. New/batch/edit cannot set cancelled; close refuses it until reopened.
Cancelled blockers do not satisfy prerequisites; validation flags such edges
and cancelled files without a record. Audit checks cancellation reason/note;
plan/query/capabilities retain the distinct status.

## Enforcement config (.tickets/config.toml)

Ticket commits skip commit hooks by default using an invocation-local empty hook
directory; `[commit] hooks = "run"` opts back in. Only ticket/plan pathspecs enter
those commits; unrelated staged files stay staged. Push hooks still run. Global
`--no-push` skips fetching/pushing for that invocation while retaining local
commits; `[push] enabled = false` remains the persistent setting. After deferred
work, run `tkt rebase` to resolve any upstream ID collision before pushing. A
rejected push reports the local commit and a retry instruction.


At close, body checkboxes use `--check-all` / `--ac`; evidence maps 1:1 by index to
frontmatter `validation_criteria`. The counts are independent.

Known effective policy values are parsed once into typed values. Invalid values
fail with key/value/source context; booleans retain true/false/1/0/yes/no aliases.
Project settings override user defaults; unknown keys remain warnings. Evidence
policy is true/warn/false, not an arbitrary truthy string.

Corpus-loading commands can skip unparseable files. Under `-o json`, each skip is
a stderr JSONL object `{ "warning": { "kind": "unparseable_ticket", "path": "...",
"message": "..." } }`; valid rows stay on stdout and successful commands retain
their exit status. Human diagnostics use the configured warning symbol/ASCII mode.

These settings control what `tkt close` requires:

| Section.Key | Values | Effect |
|-------------|--------|--------|
| `close.require_resolution` | `true` / `false` | Must provide `--resolution` or `--note` |
| `close.require_checked_acs` | `true` / `false` | All ACs must be checked (default: true) |
| `close.require_validation_criteria` | `true` / `false` | Ticket must have `validation_criteria` field |
| `close.require_validation_evidence` | `"true"` / `"warn"` / `"false"` | Must provide `--evidence` per criterion |
| `close.allow_force` | `true` / `false` | Whether `--force` escape hatch is allowed |
| `push.enabled` | `true` / `false` | Mutations push to remote (default: true) |
| `commit.hooks` | `"skip"` / `"run"` | Skip commit hooks by default; push hooks remain active |
| `validate.strict` | `true` / `false` | Warnings become errors |
| `ready.default_env` | string | Default CREW_ENV filter for `tkt ready` |
| `new.default_priority` | priority enum | Default priority for new tickets |
| `machine.capabilities` | comma-separated string | Capabilities this workstation provides (matched against ticket `requires` field) |

`tkt config --set` / `--unset` default to user configuration; add `--project` to
write `.tickets/config.toml`. Project writes validate values, use native boolean
syntax and do not commit automatically. Detached-HEAD writes warn. `--get`/`--list`
read user settings; `--show` reads effective project settings. Config output/help
show the loader's actual user path.
Project `[labels.<namespace>]` tables accept `single_select = true/false` and
`values = ["value", "other"]`. Invalid/unknown namespace fields are errors;
use a single-line JSON-style double-quoted array without a trailing comma, as the
configuration reader supports a narrow syntax rather than full TOML. Validation
enforces declared values and single-select cardinality; undeclared namespaces
remain free-form. Validation also flags dangling parents, self-parent
references and parent-chain cycles independently of blocked-by edges.

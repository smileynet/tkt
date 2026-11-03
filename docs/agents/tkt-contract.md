# Ticket and CLI compatibility contract

Read when changing stored tickets, lifecycle/close semantics, plan projection,
configuration, output/error compatibility or worktree behavior. For current
command/flag inventory use `tkt capabilities -o json`; the maintained workflow
reference is [commands.md](../../skills/tkt-core/references/commands.md). The
reference examples below preserve the contributor contract; they are not a
replacement for the complete generated manifest.

Prepared under ticket 245 and activated by ticket 246. The extracted source
now lives here; AGENTS.md keeps operational rules and conditional read pointers.

<a id="rule-10"></a>

## tkt CLI (the product)

```bash
tkt --version                                     # print version
tkt ready [--json]                                # frontier: open + deps done + env match
tkt context [TAGS...] [--clear]                   # set/show/clear active tag context for scoped work
tkt migrate [--from FORMAT] [--detect] [--confirm] # convert foreign ticket schemas (tk → tkt; preview first)
tkt new <slug> --title "..." [--body TEXT|@FILE|- | --template NAME] [--spec S] [--parent ID] [--blocked-by NN,NN] [--priority P] [--env E] [--status S] [--tags T] [--validation VC] [--requires R]
tkt batch <slug[:title]>... [--template NAME] [--spec S] [--parent ID] [--blocked-by IDS] [--priority P] [--env E] [--status S] [--tags T] [--validation VC] [--requires R]
tkt claim <id>                                    # status→in_progress, pushed
tkt cancel <id> --reason R --note "why" [--by ID] [--repoint|--drop-edges]
tkt reopen <id>                                   # cancelled→open, keeps history
tkt close <id> [--note "..."] [--resolution "..."] [--ac N,N] [--check-all] [--force] [--evidence "..." (repeatable: one per validation criterion, or N=text)]
tkt edit <id> [--title T] [--parent ID] [--tags T] [--add-tag T] [--remove-tag T] [--blocked-by IDS] [--priority P|''] [--env E|''] [--spec S|''] [--status S] [--ac N,N] [--validation VC]
tkt renumber <old> <new> [--file NAME]            # birth-window only
tkt query [--status S] [--priority P] [--parent ID] [--all-context] # context-scoped JSON Lines; bypass context explicitly
tkt blocked                                       # open tickets with unsatisfied deps
tkt capabilities                                  # machine-readable JSON feature manifest
tkt rebase [--dry-run]                            # resolve ID collisions with upstream
tkt audit [--strict] [--brief] [--deep]           # closure quality check; --deep adds evidence/template analysis
tkt sync-plan [--strict] [--brief] [plan]         # report drift (advisory, exit 0)
tkt sync-plan --check [--strict] [--brief] [plan]  # CI gate: exit 1 on drift
tkt sync-plan --fix [--dry-run] [--brief] [plan]  # fix derivable columns (default plan: docs/plan.md)
tkt validate [--strict] [--brief] [--fix [--dry-run]]  # contract + cycle + decay findings
tkt lint [--check] [IDs...]                       # normalize frontmatter style + blocked_by id refs
tkt doctor [<path>] [--strict] [--fix]                    # health check (single or cross-project)
tkt init [--target T] [--all] [--write [FILE]] [--agent-only]  # scaffold project + agent instructions
tkt config [--set K=V] [--get K] [--unset K] [--project] [--list] [--show]  # user + project config
tkt telemetry [--enable|--disable|--status|--show [--all]|--clear]  # manage local telemetry
```

`capabilities` derives command/flag inventory from Clap and adds supported domain
values, aliases, mutation modes and default output formats. See
[manifest semantics](../../skills/tkt-core/references/capabilities.md). Schemas describe
the supported workflow; use `close` for done and obey project close policy.
Top-level help and normal `init` output point to that inventory. Init recommends
project instructions and separately installed workflow skills; `--agent-only`
remains snippet-only and quiet writes omit recommendation chatter.


<a id="rule-11"></a>

### Global flags

| Flag | Effect |
|------|--------|
| `-q` / `--quiet` | Suppress confirmations, emit only essential data |
| `--no-push` | Commit ticket work locally without fetching or pushing |
| `-o json` / `--output=json` | Structured JSON output (success to stdout, errors to stderr) |
| `--dry-run` | Preview mutations without writing |
| `--color=always\|never\|auto` | Control ANSI color output (default: auto) |


<a id="rule-12"></a>

### Priority levels

`urgent` > `high` > `medium` (default) > `low`. Frontier sorts by priority bucket then ID.


<a id="rule-13"></a>

### Status values

`cancelled` is terminal retirement, separate from done, reached only through
`tkt cancel`; `tkt reopen` returns it to open. It does not satisfy prerequisites.
Duplicate/superseded require a replacement; active dependents require explicit
edge disposition. Terminal stale prerequisite edges are dropped and named.

`backlog` (parked, excluded from frontier) → `open` → `in_progress` → `done`

Reach `done` only via `tkt close`; new, batch and edit reject done. Editing a ticket's `status` to `done` by hand bypasses the close gates (ACs, resolution, evidence) and is flagged by `tkt validate`.

New tickets default to `open` (frontier-eligible); reserve `--status backlog` for work deliberately deferred out of the current cycle. Set `--tags <stream>` at creation in multi-stream projects — tags are the primary scoping mechanism for `tkt ready`/`tkt context`.


<a id="rule-14"></a>

### Query output

Query includes typed known frontmatter fields (id, title, status, blocked_by, env,
priority, spec, parent, tags, validation_criteria, requires, agents and assignee)
and unknown or unparsed stored keys as raw strings under `frontmatter`. Query and ready JSON
add `pending_blockers` for missing/non-done prerequisites; `blocked_by` retains
completed history. Cancelled prerequisites remain pending. Query preserves tag
context by default, reports actual narrowing on stderr, and accepts `--all-context`
without changing context. `-o json` remains structured with `-q`.


<a id="rule-15"></a>

### Agent eligibility

Claim records invocation identity as optional typed `assignee`. Another owner
requires `--force`; ready hides other owners and hides owned work when no identity
is supplied. Anonymous forced claims preserve the prior owner. Eligibility and
ownership are distinct; query/ready expose both.

Identify each invocation with global `--agent <lowercase-slug>` or `TKT_AGENT`; the flag wins and identity is never stored in config. `ready` and `claim` apply ticket agent eligibility. Missing identity sees unrestricted work only; `ready` reports hidden restricted work on stderr. Use `claim <id> --force` only as an explicit scheduling override.

Create or replace eligibility with mutually exclusive `--agents codex,claude` or `--exclude-agents antigravity` on `new`, `batch`, and `edit`; `edit --agents ''` or `edit --exclude-agents ''` clears it. Frontmatter stores one optional signed `agents` list: `["codex", "claude"]` allows either, `["!antigravity"]` excludes that agent, omission allows everyone. Lists must be nonempty, homogeneous in sign, duplicate-free, and contain lowercase alphanumeric/hyphen slugs. Serialization sorts names. `ready` human/JSON and `query` JSON expose the policy (`agents: []` in JSON means unrestricted). `query` honors tag context but never filters by current agent. Tags scope work streams; `requires` remains all-of machine capabilities; eligibility is separate from ownership after claim.


<a id="rule-16"></a>

### Configuration

Ticket commits skip commit hooks by default (`commit.hooks = "run"` opts in);
pre-push hooks remain active. Commits include only affected ticket/plan paths.
`--no-push` is invocation-local; push.enabled=false remains persistent. Deferred
ID collisions must be resolved with tkt rebase before a later push.

Set/unset default to user scope; `--project` writes shared project configuration
with typed values and atomic replacement, without a Git commit. Detached-HEAD
project writes warn. New bodies accept literal text, @file or stdin `-`; new/batch
templates are verbatim files under `.tickets/templates/`. Default prompts remain
detectable as unfilled work.

Known effective policy values are parsed strictly with key/value/source errors;
boolean aliases true/false/1/0/yes/no remain valid. Unknown keys retain warnings.
Corpus skip diagnostics use stderr JSONL warnings under `-o json` and `sym_warn`
otherwise; valid rows and successful exit semantics are preserved.

- **User config**: `~/.config/tkt/config.toml` — debug mode, format preferences
- **Project config**: `.tickets/config.toml` — committed to repo, shared by contributors
  - `[close]` require_resolution, require_checked_acs, require_validation_criteria, require_validation_evidence, allow_force
  - `[validate]` strict, missing_resolution_before_id (project-specific legacy ratchet)
  - `[ready]` default_env (legacy single capability; prefer `[machine] capabilities`)
  - `[priority]` warn_unknown
  - `[new]` default_priority
  - `[push]` enabled (set false for local-only repos)
  - `[machine]` capabilities (comma-separated canonical eligibility set for `requires` matching)
  - `[labels.<namespace>]` single_select boolean and values string list. The narrow config reader requires a single-line JSON-style double-quoted array without a trailing comma; malformed/unknown label fields are Validation errors. Validate enforces closed values and single-select cardinality; undeclared namespaces remain free-form.


<a id="rule-17"></a>

### Environment variables

| Var | Effect |
|-----|--------|
| `TKT_DEBUG=1\|true\|log\|file` | Debug output to the local debug log (stderr remains clean) |
| `TKT_DEBUG=stderr\|human` | Human-readable debug output to stderr |
| `TKT_DEBUG=json` | Debug output as JSON Lines on stderr |
| `TKT_ASCII=1` | ASCII-only symbols (✓→[ok], ✗→[err], ⚠→[warn]) |
| `NO_COLOR=1` | Disable ANSI color |
| `TKT_ADVICE=0` | Disable advisory hints (e.g. the batch nudge); `off`/`false` also accepted |
| `CREW_ENV` | Filter frontier by env (corp/personal) |
| `DO_NOT_TRACK=1` | Disable telemetry and update-check requests/cache refresh |
| `TKT_NO_USER_CONFIG=1` | Skip user config file (for testing — prevents ambient config leaking into child processes) |


<a id="rule-19"></a>

## Contract

The deep-audit template rule checks actual intent/acceptance sections, ignoring quoted template
examples. Empty/default prompts and real placeholder ACs remain warnings even
when checked after closure; historical evidence omissions are not waived.
Open/in-progress stub detection retains its sole-placeholder policy.

Rebase reports candidate stale body citations (path/line/old-new mapping) without
rewriting prose. Validation warns on missing unqualified local #NNN/ticket NNN
citations; qualify external issues or link them and quote code examples.

Body AC checkboxes map to `--check-all` / `--ac`; frontmatter validation_criteria
maps 1:1 by index to --evidence. The counts are independent. Fill body prompts at
creation and link the intent source (--spec or a durable body reference).

Claim, close, and status-changing edit synchronize existing `docs/plan.md` rows in
the same commit. Commit or stash plan edits first. Missing rows remain warnings
until `sync-plan --fix` appends them under Unplanned tickets. No plan means no new
file; alternate plans use explicit sync-plan.

- Files are the database: `.tickets/{NN}-{slug}.md` with YAML frontmatter
- Tool never manages specification prose (body is user-owned; close appends a Resolution section)
- Push-to-claim: pushed commit = claimed id (race detection on push rejection)
- Exit codes: 0=success, 1=domain failure (not found, conflict, drift), 2=operational crash (I/O, git, parse)
- Output: JSON by default for validate/sync-plan; human by default for ready/new/claim/close/edit (use --json for ready)
- Spike branches: closing from `spike/*` auto-appends branch name to resolution
- Worktree-aware: works from git worktrees (`.tickets/` is part of the checked-out tree)


<a id="rule-42"></a>

### Parent progress

Parents with any direct children are containers, excluded from ready. Use `tkt children ID` for full-corpus direct-child progress (done/total includes cancelled in total, with cancelled separate); `-o json` emits one object and `-q` lists child IDs. Progress never changes stored status: close parents explicitly with normal AC/evidence gates.


<a id="rule-43"></a>

### Transferring acceptance criteria

Move an unchecked body criterion with `tkt move-ac SOURCE --ac N --to DEST` (N counts all top-level single-line body ACs, including checked/moved). The source uses `[>]` plus a destination marker; the receiver gets an unchecked copy with a source back-reference. This records transfer, not completion. Validation criteria and evidence remain independent and unchanged; supply truthful proof for every retained validation criterion at close. Retry the same transfer after interruption; inspect conflicts instead of editing link identities.


<a id="rule-44"></a>

### Parent-scoped frontier

Use `tkt ready --parent ID` for ready direct children of an existing parent, preserving priority/ID order, capability matching, invocation eligibility/ownership and tag context (including its existing untagged-ticket behavior). Human in-progress context is parent-scoped too. JSON rows include stored parent when present. This is native-parent-only; no epic-tag alias is added. Legacy tag groups can continue using `tkt context epic:slug` with normal ready.


<a id="rule-45"></a>

### Plan lifecycle comparison

Plan tables compare all five lifecycle labels exactly. Both `done` and canonical `✅ done` are recognized; cancellation rows retain their reason. Unknown status cells are findings rather than silently treated as open. Plain sync-plan remains advisory for derivable drift; --check or --strict fails, while --fix regenerates cells without managing narrative prose.

Update-check cadence is user-only: `tkt config --set update_check.interval=1h` (also `24h`, `7d`, or `never`; positive whole hours/days). Default: `24h`. `TKT_UPDATE_CHECK_INTERVAL` overrides the setting; `TKT_UPDATE_CHECK=0`, `DO_NOT_TRACK=1`, CI, quiet and dry-run still suppress update-check requests/cache writes. Invalid intervals are rejected on config writes; a malformed stored/environment interval skips the advisory check with a warning.

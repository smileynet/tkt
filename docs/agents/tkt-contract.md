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

Source-reference readiness: `validate` reports `no-source-reference` at info severity for open/in-progress tickets lacking a nonempty spec, prerequisite, parent, local ticket citation, docs/specs/.memory path or HTTP(S) link. Fenced examples do not count. Backlog and terminal tickets are excluded. It remains advisory even under `--strict`; `[validate] require_source_reference = true` explicitly makes absence an error. Existing integrity rules still validate referenced targets separately.

Frontier readiness: `validate` emits info-only `frontier-readiness` for missing source reference, non-moved/nonempty acceptance checkbox or validation-criterion slots on the project structural frontier. It reuses the source predicate and native dependency/container/capability selection, with the same configured machine/legacy-env capability resolution as ready. Validation is project-wide: invocation ownership, agent eligibility, tag context and ready --parent are views and do not narrow this audit. Info remains advisory under --strict; no new claim/close gate, queue filter or sort is added. These are slot checks, not proof that the contents are sufficient. Backlog-chain diagnosis is outside this first bounded rule.

Optional Git advice: `tkt hooks install|check|remove|run` manages only the exact owned project pre-commit shim, shared by repository worktrees. Install/remove are explicit, respect global dry-run and preserve foreign/modified hooks or any configured core.hooksPath. The shim always permits committing, even when validation or tkt fails; it is not the verification gate. No hook is installed by init/deploy, no Git config or commit/push changes, and normal tkt mutation hook bypass remains unchanged.

Derived plans: `tkt sync-plan --generate [FILE]` renders the full corpus in priority/ID order (all statuses, no tag/agent narrowing). No file means Markdown stdout; -o json wraps projection/count. An explicit new/empty file or one owned unfenced tkt:projection:begin/end block is atomically written; human text outside stays exact. Global dry-run writes nothing. Unmarked nonempty files, bad markers, output links/database paths and incomplete/duplicate IDs are rejected; parent directories must exist. Generation conflicts with check/fix/strict/brief. Existing 3-column drift/fix semantics remain, with numeric and canonical slug IDs; no narrative parsing or commit/push.

Dry-run diagnostics: file debug logging is suppressed for previews, including the local validate/rebase dry-run forms. Existing logs are neither created nor appended. Explicit human/JSON stderr debug output remains available; ordinary non-preview file logging is unchanged. This repairs the documented no-write preview behavior.

Migration safety: tk preview and apply prepare readable source snapshots and reject destination or backup conflicts. New output/backups are published completely without overwriting arrivals; an original is removed only after its converted output succeeds. Backups remain .tickets.bak. This is per-file safety, not an atomic batch or external-editor lock; retain backups and inspect partial results on failure. Hard-link support is required for new-file publication.

### Local configuration, context and initialization writes

Config/context/init text writes use atomic publication, reject linked or non-regular targets and unreadable existing text, and refuse changed preparation snapshots. New files never overwrite arrivals. These are per-file operations, not a batch transaction or external-writer lock; context --clear removes the state entry itself.

Preview performs no writes; existing command previews do not inspect every filesystem precondition. Coordinate external writers. `init --write FILE` retains explicit custom paths, including paths outside the checkout; it is not a path-containment boundary. Directory effects reject linked/non-directory targets, but ancestor paths are resolved by the filesystem. New-file publication requires hard-link support on the destination filesystem.

## Controlled history reversal

Explicit recovery: tkt --dry-run undo then tkt undo reverts only a clean HEAD new/claim/close carrying a versioned receipt from this build. Legacy/other operations, root/merge/rebased history and repeated undo are refused. It adds a revert commit, never resets history; push rejection keeps a local commit without automatic rebase.

The first version supports regular non-executable ticket files and an optional already-tracked plan status update. Other effects (configuration, context, init, hooks, telemetry and caches) are outside the inverse. Unrelated untracked files are preserved; the index and tracked worktree must be clean with no active Git operation or index lock. A receipt is structural provenance, not author authentication.

Global --no-push and push.enabled apply. Preview reads without fetching or creating a temporary index. A failed revert leaves conflict/sequencer state for inspection; a failed commit leaves the staged inverse. An opted-in hook that changes the prepared tree leaves a local commit and prevents push. Inspect git status and use explicit reviewed Git recovery for unsupported or interrupted work. Do not undo a valid local commit just because publishing failed.


Append a ticket note with tkt note ID --append "text" (or @FILE / - for UTF-8 stdin, at most 64 KiB). Notes preserve lifecycle and existing body bytes; identify the caller with --agent or TKT_AGENT. The literal ## Resolution is reserved even inside examples. The target must be tracked and clean. Only cooperating note processes in the same worktree serialize; external editors, other mutations and other worktrees require coordination. Dry-run writes nothing. Every invocation appends: after a push fails, inspect the retained local commit and retry git push. If committing fails, inspect and explicitly finish or restore the retained note before retrying; do not append again. Note commits are outside undo receipt eligibility.


Import open GitHub issues with tkt import --github (current origin) or --repo OWNER/NAME; --label selects one source label. Preview first with --dry-run. gh owns authentication; only github.com is supported and nothing is written to GitHub. Successful extraction stays below 1,000 received objects and 10 MiB raw JSON, including skipped closed issues/PRs. Only import:github is generated as a local tag; preserve source labels and unknown fields in owned JSON snapshots. Never infer local acceptance or validation from provider checkboxes. Repeatable --validation (alias --vc, at most 16 nonempty values of 2,000 bytes each) applies the same explicit criteria to every new issue and satisfies projects requiring criteria at creation.

Imports are create-only and require a clean tracked worktree/index; unrelated untracked work is preserved. Valid existing immutable source identities are skipped without replacing local notes or lifecycle. Provider renames retain original provenance; divergent local/upstream ownership receipts require explicit reconciliation. A per-worktree allocation marker serializes new/batch/import only; coordinate other writers. Dry-run does not fetch, lock or persist. Interrupted markers require inspection, never automatic stealing. A pending import journal resumes only with the same repository, label, validation arguments and branch, using its recorded source before a new API read. Commit failure can retain staged files; push failure retains a local commit. Inspect the reported state and repeat that exact request to resume, or retry git push for a durable commit. Import is outside undo eligibility.


Use tkt done --week for current done work in the inclusive last 604,800 seconds (7x24h), UTC; tkt done shows all current done work. Dates come from verified same-path/same-ID single-parent Git status transitions using recorded committer time, not body dates or file mtime. Unknown dates stay visible with a reason and null window membership; known out-of-window rows are omitted only with --week. Cancelled/reopened work is excluded. Tag context applies with a narrowing notice; --all-context bypasses it. No agent/owner/machine filtering or persisted completion field is added. JSON is one object with separate dated and unknown counts; it is not proof of acceptance quality.

History attribution is bounded: 4,096 lookups/report, 256 path commits/ticket, 32 MiB/object, 8 MiB/listing, 4,096 ticket entries/tree, 16 MiB/8,192 entries each for tree and state/blob/commit metadata payload caches, and 64 MiB history payload/report. It uses local Git with no lazy fetch/replacement objects/optional locks; unsupported read options, missing/shallow/merge/rename/ID-discontinuous/unparseable history or limits produce unknowns. Existing diagnostic/telemetry settings still apply; no tickets, index or refs are changed.


Lifecycle hooks are optional advisory commands configured as JSON argv arrays in hooks.on_new/on_claim/on_close (default []). Project commands execute only after separate Git-local trust: git config --local tkt.lifecycleHooks true; this approves future hook edits and linked worktrees too. Inspect trusted code and revoke trust before untrusted updates. Hooks inherit caller credentials/environment. They run after the publication attempt, never gate it; spawn/failure/timeout warnings preserve the original result. The two-second deadline terminates only the direct child, not descendants; trusted hooks must not detach. Dry-run/failed commits emit nothing, allocation retries emit only the final retained attempt, and nested hooks are suppressed. No automatic replay: inspect the consumer and explicitly replay safely, never repeat a successful mutation as a notification retry.

See [hook configuration, trust and recovery](../lifecycle-hooks.md).

Optional extension schema #93 is design-only: see [the versioned scalar and migration contract](../extension-schema-design.md). No runtime schema loader, dynamic flags or schema migration command is available. Unknown raw frontmatter continues on the existing surgical-edit/query path; the fixture preview is not executable CLI input.


Primary/worker roles are invocation conventions: identify each caller with --agent or TKT_AGENT; use --agents/--exclude-agents for eligibility and let identified claim record assignee. Keep tags for workstreams and requires for all-of declared machine capabilities, never agent roles. Primary oversight is not a second claim; use authorized delegation and required independent review. An explicit coordinated handoff may reopen WIP with edit --status open, update eligibility, then use claim --force to override retained ownership. Force never claims WIP; direct claim does not establish machine/dependency readiness. There is no init --role or new/edit --assignee flag.

Read the [standalone role and handoff reference](../../skills/tkt-core/references/roles.md) before coordinating primary/workers or ownership overrides.


Native write nudges are explicit: tkt init --write-nudge claude|codex|both installs owned advisory config; --nudge-action check is read-only and remove removes exact owned entries while retaining inactive helper/ownership files. Ordinary init/--all leaves native hooks unchanged. Node22+ is required only for this optional helper. No trust, user settings, permissions or deny policy is changed. Detection covers Claude Edit/Write ticket paths and canonical Codex apply_patch headers only; body/spec edits remain allowed. Inspect both generated commands and helper before native trust review; helper changes may not invalidate native trust. Check proves owned config presence, not native activation. Foreign/modified/duplicate JSON is preserved by refusal; writes are atomic per-file, not a multi-file transaction.

Before setup/removal, read [native write-nudge coverage, trust and recovery](../../skills/tkt-core/references/native-hooks.md).

## Prepared effect phases

Global --dry-run writes nothing, including Git refs, locks, cadence and diagnostic caches. Remote-backed previews use cached refs; final remote-dependent allocation/publication remains unresolved. Command-owned text and owner-scoped Git/process phases share one Preview/Apply dispatcher. Existing transaction owners retain retry/recovery algorithms; lifecycle capture guards and push configuration remain in force. See [the effect inventory](../mutation-effects-audit.md).
JSON dry runs emit phase descriptions as stderr JSON Lines:
`{"preview_effects":[{"operation":"…","paths":["…"],"unresolved":false}]}`.
Command stdout contracts are preserved. Summaries include owned support cleanup
and conditional notifications; unresolved paths/outcomes are not execution receipts.
Preview Git reads disable optional locks and lazy object fetching, so missing
cached objects cause a read error rather than an implicit network mutation.

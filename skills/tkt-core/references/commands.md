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
| `tkt note <id> --append TEXT\|@FILE\|-` | Append a dated attributable note; lifecycle unchanged |
| `tkt import --github [--repo OWNER/NAME] [--label NAME] [--validation TEXT]` | Create open tickets from bounded GitHub reads; replay preserves local work |

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
| `tkt audit --deep` | Inspect closure evidence and actual template-section content; ignore quoted examples while retaining real placeholders, including checked ones |
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

Consumers may optionally run `tkt validate --strict --brief` and `tkt lint --check`
in CI, after a pinned installation and checkout. Add `tkt sync-plan --check --strict
--brief` only when maintaining a plan. These commands report mechanical failures;
host settings/permissions determine whether their results are required or bypassed.
They install no workflow/hook and do not prove evidence truth or implementation intent.

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

## Parent progress

Parents with any direct children are containers, excluded from ready. Use `tkt children ID` for full-corpus direct-child progress (done/total includes cancelled in total, with cancelled separate); `-o json` emits one object and `-q` lists child IDs. Progress never changes stored status: close parents explicitly with normal AC/evidence gates.

## Transferring acceptance criteria

Move an unchecked body criterion with `tkt move-ac SOURCE --ac N --to DEST` (N counts all top-level single-line body ACs, including checked/moved). The source uses `[>]` plus a destination marker; the receiver gets an unchecked copy with a source back-reference. This records transfer, not completion. Validation criteria and evidence remain independent and unchanged; supply truthful proof for every retained validation criterion at close. Retry the same transfer after interruption; inspect conflicts instead of editing link identities.

The receiving ticket must exist and be nonterminal. Checked criteria and already
received criteria cannot be moved. Only one criterion is transferred per call.

Source: `- [>] Requirement <!-- tkt-moved:02 -->`

Receiver: `- [ ] Requirement <!-- tkt-from:01:1 -->`

Keep marker IDs, the source criterion ordinal and matching text stable. Validation
and audit report malformed, missing, changed or duplicate links as errors; close
rejects broken links even with force. Checking all ACs leaves moved markers intact.
A receiving criterion stays unchecked until its work is independently completed.
Do not move a receiving criterion again; this initial contract supports one hop.

Each file is replaced atomically on the same filesystem. The destination is
written first, so interruption can expose a receiving copy while the source is
still unchecked. Validation reports that intermediate state; rerunning the same
command repairs it without duplicating the receiver. If both writes landed but
commit did not, retry commits the pair. This is recoverable multi-file publication,
not a simultaneous two-file filesystem rename. A successful repeated transfer is
a no-op. A push failure retains the commit; follow its instruction to push.
Normal mutation hook and push policy apply, and dry-run writes nothing.

Body ACs and frontmatter validation criteria have independent counts. The command
does not relocate validation criteria, renumber evidence, invent proof, or waive
any retained close gate. Record transfer provenance as provenance; it is not
implementation evidence. Review validation scope explicitly when work moves.

## Parent-scoped frontier

Use `tkt ready --parent ID` for ready direct children of an existing parent, preserving priority/ID order, capability matching, invocation eligibility/ownership and tag context (including its existing untagged-ticket behavior). Human in-progress context is parent-scoped too. JSON rows include stored parent when present. This is native-parent-only; no epic-tag alias is added. Legacy tag groups can continue using `tkt context epic:slug` with normal ready.

## Plan lifecycle comparison

Plan tables compare all five lifecycle labels exactly. Both `done` and canonical `✅ done` are recognized; cancellation rows retain their reason. Unknown status cells are findings rather than silently treated as open. Plain sync-plan remains advisory for derivable drift; --check or --strict fails, while --fix regenerates cells without managing narrative prose.

Update-check cadence is user-only: `tkt config --set update_check.interval=1h` (also `24h`, `7d`, or `never`; positive whole hours/days). Default: `24h`. `TKT_UPDATE_CHECK_INTERVAL` overrides the setting; `TKT_UPDATE_CHECK=0`, `DO_NOT_TRACK=1`, CI, quiet and dry-run still suppress update-check requests/cache writes. Invalid intervals are rejected on config writes; a malformed stored/environment interval skips the advisory check with a warning.

Source-reference readiness: `validate` reports `no-source-reference` at info severity for open/in-progress tickets lacking a nonempty spec, prerequisite, parent, local ticket citation, docs/specs/.memory path or HTTP(S) link. Fenced examples do not count. Backlog and terminal tickets are excluded. It remains advisory even under `--strict`; `[validate] require_source_reference = true` explicitly makes absence an error. Existing integrity rules still validate referenced targets separately.

Frontier readiness: `validate` emits info-only `frontier-readiness` for missing source reference, non-moved/nonempty acceptance checkbox or validation-criterion slots on the project structural frontier. It reuses the source predicate and native dependency/container/capability selection, with the same configured machine/legacy-env capability resolution as ready. Validation is project-wide: invocation ownership, agent eligibility, tag context and ready --parent are views and do not narrow this audit. Info remains advisory under --strict; no new claim/close gate, queue filter or sort is added. These are slot checks, not proof that the contents are sufficient. Backlog-chain diagnosis is outside this first bounded rule.

Optional Git advice: `tkt hooks install|check|remove|run` manages only the exact owned project pre-commit shim, shared by repository worktrees. Install/remove are explicit, respect global dry-run and preserve foreign/modified hooks or any configured core.hooksPath. The shim always permits committing, even when validation or tkt fails; it is not the verification gate. No hook is installed by init/deploy, no Git config or commit/push changes, and normal tkt mutation hook bypass remains unchanged.

Derived plans: `tkt sync-plan --generate [FILE]` renders the full corpus in priority/ID order (all statuses, no tag/agent narrowing). No file means Markdown stdout; -o json wraps projection/count. An explicit new/empty file or one owned unfenced tkt:projection:begin/end block is atomically written; human text outside stays exact. Global dry-run writes nothing. Unmarked nonempty files, bad markers, output links/database paths and incomplete/duplicate IDs are rejected; parent directories must exist. Generation conflicts with check/fix/strict/brief. Existing 3-column drift/fix semantics remain, with numeric and canonical slug IDs; no narrative parsing or commit/push.
Owned projection markers are standalone, unfenced lines: `<!-- tkt:projection:begin -->`
and `<!-- tkt:projection:end -->`. New-file publication requires hard-link support
and refuses a destination that appears after preview. Existing-file replacement
checks for a changed snapshot but still requires a single writer: coordinate
external edits to the destination as well as the corpus.

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

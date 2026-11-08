---
name: tkt-core
description: "Track tasks as markdown files in your git repo. Use when managing tickets, checking the frontier (what's ready), claiming work, closing tasks, creating tickets, decomposing work into a task graph, or validating project health. Trigger: tkt, tickets, what to work on, create ticket, validate tickets, decompose, break into tasks, work breakdown, task graph, ticket format, blocked by, backlog, park this, defer."
compatibility: Requires git
---

# tkt

Track tasks as markdown files in your git repo. One file per ticket in `.tickets/`, with status and dependencies in YAML frontmatter. tkt computes what's ready to work on next.

Agent setup: `tkt init` recommends project instructions (`tkt init --write`),
separately installed workflow skills/plugins, and `tkt capabilities -o json` for
command discovery. Installing the CLI alone does not install the workflow skills.

## Jobs to be done

| When I'm... | I want to... | Command |
|---|---|---|
| Discovering CLI commands and flags | read the complete structured inventory | `tkt capabilities -o json` ([schema reference](references/capabilities.md)) |
| Routing tickets to agents | allow either named agent or exclude named agents | `tkt new <slug> --agents codex,claude` / `--exclude-agents antigravity` |
| Starting a work session | see what's unblocked and prioritized | `tkt ready` |
| Planning future work | create tickets without them cluttering the frontier | `tkt new <slug> --title "..." --status backlog` |
| Ready to start something | promote a backlogged ticket to actionable | `tkt edit <id> --status open` |
| Grabbing a task in a shared repo | signal that I'm working on it | `tkt claim <id>` |
| Finishing a task | prove it's done with evidence | `tkt close <id> --check-all --evidence "..."` |
| Recording a decision or observation | append an attributable note without an editor | `tkt note <id> --append "..."` |
| Recovering an immediately mistaken mutation | preview and revert a recognized HEAD safely | `tkt --dry-run undo` then `tkt undo` |
| Discovering work during implementation | capture it as a separate ticket without losing focus | `tkt new <slug> --title "..."` (open — or `--status backlog` only if genuinely deferred) |
| Creating several related tickets at once | one commit/push instead of N; share `--tags`/`--blocked-by` | `tkt batch "slug:title" "slug:title" ... --tags STREAM` |
| Coordinating primary/workers or handing off ownership | keep identity separate from machine capabilities | Read [role examples](references/roles.md) |
| Preparing a completed-work report | inspect Git dates and explicit unknowns | `tkt done --week -o json` |
| Checking project health | find cycles, broken deps, stale WIP | `tkt validate` / `tkt audit` |
| Querying the full corpus | bypass context; optionally filter status or priority | `tkt query --all-context --status open --priority high` |
| Working in a specific environment | see only relevant work | Set `machine.capabilities` in config (or `CREW_ENV=corp` for legacy env filtering) → `tkt ready` filters by `requires` |
| Keeping a plan in sync | report drift between tickets and plan doc (advisory) | `tkt sync-plan` (add `--check` to gate CI) |
| Scoping work to a subsystem | filter frontier by tags | `tkt context backend` → `tkt ready` shows only matching |
| Returning to unscoped view | clear tag filter | `tkt context --clear` |
| Importing from another tool | preview, review, then convert foreign tickets | `tkt migrate --from tk --dry-run` → `tkt migrate --from tk --confirm` |
| Trying the GitHub issue bridge | preview a bounded create-only import | `tkt import --github --repo OWNER/NAME --dry-run` |

For full flags and options, see [commands.md](references/commands.md).

**Creation/edit traps:** `tkt new` supplies section prompts; use `--body` with text,
`@file` or stdin `-`, or new/batch `--template NAME`, to provide a complete body.
Fill intent/acceptance prompts before validating open work. Use backlog only for genuinely deferred
work. `tkt edit --blocked-by` replaces the whole list: read the existing blockers,
combine them with the new IDs, and pass the complete set.

---

## Agent eligibility

Claim, close, and status-changing edit synchronize existing `docs/plan.md` rows in
the same commit. Commit or stash plan edits first. Missing rows remain warnings
until `sync-plan --fix` appends them under Unplanned tickets. Without a plan file,
mutations create none. Alternate plans require explicit `sync-plan [PLAN]`.

Identify each invocation with global `--agent <lowercase-slug>` or `TKT_AGENT`; the flag wins and identity is never stored in config. `ready` and `claim` apply ticket agent eligibility. Missing identity sees unrestricted work only; `ready` reports hidden restricted work on stderr. Use `claim <id> --force` only as an explicit scheduling override.

Create or replace eligibility with mutually exclusive `--agents codex,claude` or `--exclude-agents antigravity` on `new`, `batch`, and `edit`; `edit --agents ''` or `edit --exclude-agents ''` clears it. Frontmatter stores one optional signed `agents` list: `["codex", "claude"]` allows either, `["!antigravity"]` excludes that agent, omission allows everyone. Lists must be nonempty, homogeneous in sign, duplicate-free, and contain lowercase alphanumeric/hyphen slugs. Serialization sorts names. `ready` human/JSON and `query` JSON expose the policy (`agents: []` in JSON means unrestricted). `query` honors tag context but never filters by current agent. Tags scope work streams; `requires` remains all-of machine capabilities; eligibility is separate from ownership after claim.

## Status lifecycle

Claim records the invocation identity as optional `assignee`; ready hides other
owners and hides owned work when unidentified. `--force` overrides ownership
checks; anonymous forced claims preserve the recorded owner. This is independent
of allow/exclude eligibility.

```
backlog → open → in_progress → done
```

| Status | Meaning | Appears in `tkt ready`? |
|--------|---------|------------------------|
| `backlog` | Parked — not prioritized, not actionable yet | **No** |
| `open` | Available work or a parent container | **Yes** (if deps done and not a container) |
| `in_progress` | Claimed, actively being worked | **No** |
| `done` | Completed, verified | **No** |

**Key rule:** `tkt ready` only shows `open` tickets whose `blocked_by` are all `done`, excluding any ticket with direct children. Backlogged tickets are invisible to the frontier.

Only `tkt close` reaches `done`; new, batch and edit reject that status so closure gates run.

Retire abandoned work with `tkt cancel ID --reason REASON --note "why"`, not close.
Duplicate/superseded require `--by ID`; active dependents need explicit repoint/drop.
`tkt reopen ID` reopens cancelled work, preserving its history. Cancelled is distinct
from done and cannot satisfy prerequisites. See the command reference for edge handling.

Body AC checkboxes use `--check-all` / `--ac`; frontmatter `validation_criteria`
maps 1:1 by index to `--evidence`. Their counts are independent.

Hierarchy uses optional `--parent ID` on new/batch/edit, separate from prerequisites;
edit `--parent ''` clears it, and query `--parent ID` selects children. Invalid
parent targets and introduced cycles are rejected.

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
| `tkt audit --deep` | Evidence quality and actual unfilled template sections; quoted examples are ignored |
| `tkt sync-plan` | Plan drift — ticket status vs plan document (advisory; `--check` gates CI) |
| `tkt doctor` | Setup verification — current project or multi-project scan |
| `tkt lint --check` | Preview style and missing numeric-ID repairs without writing; bare `tkt lint` applies them |

Validation checks parent targets/cycles separately from prerequisites. Project
label namespaces declare `single_select` and a single-line double-quoted `values`
array; malformed declarations fail configuration loading. Validation enforces
declared values and single-select cardinality; undeclared tags remain free-form.
See the command reference for the narrow configuration syntax.

Invalid known policy values fail with key/value/source context instead of silently
weakening gates. Corpus skip warnings are nonfatal; `-o json` puts one structured
warning per stderr line, so distinguish an empty corpus from skipped invalid files.

Use `--strict` to treat warnings as errors, `--brief` for short output, `--fix` to auto-repair (validate and sync-plan). Full options in [commands.md](references/commands.md).

---

## Key behaviors

Rebase reports potentially stale body citations instead of rewriting prose.
Validation treats unqualified `#NNN`/`ticket NNN` as local references and warns
when missing; qualify external issues or link them, and use code formatting for examples.

- **Push on write:** All mutations push to remote immediately. Disable with `push.enabled = false` in `.tickets/config.toml`.
- **Race detection:** Push rejected → tkt retries with new ID (new/batch) or reports the conflict.
- **Priority sort:** `tkt ready` sorts by priority bucket (urgent > high > medium > low) then by ID.
- **Surgical edits:** `tkt edit` only touches the field you specify — everything else preserved.
- **Env filtering (legacy):** Tickets with `env: corp` only appear in `tkt ready` when `CREW_ENV=corp`. No env field = visible everywhere.
- **Requires filtering:** Tickets with `requires: [gpu, linux]` only appear if `machine.capabilities` config includes all listed values. If a ticket has `env` but no `requires`, requires is synthesized from env for backward compat. Prefer `requires` over `env` for new tickets.
- **Tag context:** `tkt context backend api` sets a session-scoped filter. `ready`, `query`, and `blocked` only show tickets whose `tags` include all context-include tags and none of the context-exclude tags. `new` and `batch` auto-inherit context tags.
- **Scripted/non-interactive use — advice routing:** `tkt`'s advisory hints (e.g. the batch nudge) print to **stderr** in human mode. `-q`/`--quiet` keeps text output minimal; `-o json` selects structured output and remains JSON when combined with `-q`. Creation JSON exposes final `id`/`path` fields (`tickets[]` for batch), including after allocation retry; read those instead of parsing `result`. Set `TKT_ADVICE=0` (also `off`/`false`) to silence the nudge entirely.

Query retains the active tag context by default and reports actual narrowing on
stderr. Use `tkt query --all-context` to bypass context for a whole-board audit
without changing stored context or the current environment.
Query includes tags, validation criteria, requires, ownership and eligibility;
unknown or unparsed stored fields are raw strings under `frontmatter`. Query and ready JSON
retain historical `blocked_by` alongside unfinished/missing `pending_blockers`.

---

## Creating tickets (content quality)

Before writing ticket body content, read [ticket-standards.md](references/ticket-standards.md). Every ticket needs:

1. Intent source (why does this exist?)
2. Context for a fresh agent (files to read, decisions made)
3. Behavioral outcome (what changes for the user)
4. Testable validation (acceptance criteria an agent can verify)

Set `--tags <stream>` at creation in multi-stream projects (one tag is the norm; omit in single-stream projects) — retro-tagging rarely happens, and tags are what scope `tkt ready`/`tkt context` to a subsystem.

Retag existing work with edit: `--tags a --tags b` replaces, `--tags ''` clears,
`--add-tag x` adds unique values, and `--remove-tag y` removes last. Edit does not
inherit active context tags or change that context.

---

## References

- [commands.md](references/commands.md) — full command reference, all flags, enforcement config
- [ticket-format.md](references/ticket-format.md) — file format, required/optional fields
- [ticket-standards.md](references/ticket-standards.md) — quality gate for ticket content
- [wide-refactors.md](references/wide-refactors.md) — expand-contract sequencing
- [migration-assist.md](references/migration-assist.md) — converting foreign ticket schemas

## Optional consumer enforcement

Optional consumer CI can run strict validation and check-only lint after a pinned
installation. Host settings and permissions determine whether results are required;
see [enforcement configuration](references/commands.md#enforcement-config-ticketsconfigtoml).

## Parent progress

Parents with any direct children are containers, excluded from ready. Use `tkt children ID` for full-corpus direct-child progress (done/total includes cancelled in total, with cancelled separate); `-o json` emits one object and `-q` lists child IDs. Progress never changes stored status: close parents explicitly with normal AC/evidence gates.

## Transferring acceptance criteria

Move an unchecked body criterion with `tkt move-ac SOURCE --ac N --to DEST` (N counts all top-level single-line body ACs, including checked/moved). The source uses `[>]` plus a destination marker; the receiver gets an unchecked copy with a source back-reference. This records transfer, not completion. Validation criteria and evidence remain independent and unchanged; supply truthful proof for every retained validation criterion at close. Retry the same transfer after interruption; inspect conflicts instead of editing link identities.

## Parent-scoped frontier

Use `tkt ready --parent ID` for ready direct children of an existing parent, preserving priority/ID order, capability matching, invocation eligibility/ownership and tag context (including its existing untagged-ticket behavior). Human in-progress context is parent-scoped too. JSON rows include stored parent when present. This is native-parent-only; no epic-tag alias is added. Legacy tag groups can continue using `tkt context epic:slug` with normal ready.

## Plan lifecycle comparison

Plan tables compare all five lifecycle labels exactly. Both `done` and canonical `✅ done` are recognized; cancellation rows retain their reason. Unknown status cells are findings rather than silently treated as open. Plain sync-plan remains advisory for derivable drift; --check or --strict fails, while --fix regenerates cells without managing narrative prose.

Update-check cadence is a user preference: `tkt config --set update_check.interval=7d` (default `24h`; positive hours/days or `never`). It does not change ticket readiness or verification. Existing quiet/dry-run, CI, `DO_NOT_TRACK=1` and `TKT_UPDATE_CHECK=0` suppression remains effective.

Source-reference readiness: `validate` reports `no-source-reference` at info severity for open/in-progress tickets lacking a nonempty spec, prerequisite, parent, local ticket citation, docs/specs/.memory path or HTTP(S) link. Fenced examples do not count. Backlog and terminal tickets are excluded. It remains advisory even under `--strict`; `[validate] require_source_reference = true` explicitly makes absence an error. Existing integrity rules still validate referenced targets separately.

Frontier readiness: `validate` emits info-only `frontier-readiness` for missing source reference, non-moved/nonempty acceptance checkbox or validation-criterion slots on the project structural frontier. It reuses the source predicate and native dependency/container/capability selection, with the same configured machine/legacy-env capability resolution as ready. Validation is project-wide: invocation ownership, agent eligibility, tag context and ready --parent are views and do not narrow this audit. Info remains advisory under --strict; no new claim/close gate, queue filter or sort is added. These are slot checks, not proof that the contents are sufficient. Backlog-chain diagnosis is outside this first bounded rule.

Optional Git advice: `tkt hooks install|check|remove|run` manages only the exact owned project pre-commit shim, shared by repository worktrees. Install/remove are explicit, respect global dry-run and preserve foreign/modified hooks or any configured core.hooksPath. The shim always permits committing, even when validation or tkt fails; it is not the verification gate. No hook is installed by init/deploy, no Git config or commit/push changes, and normal tkt mutation hook bypass remains unchanged.

Derived plans: `tkt sync-plan --generate [FILE]` renders the full corpus in priority/ID order (all statuses, no tag/agent narrowing). No file means Markdown stdout; -o json wraps projection/count. An explicit new/empty file or one owned unfenced tkt:projection:begin/end block is atomically written; human text outside stays exact. Global dry-run writes nothing. Unmarked nonempty files, bad markers, output links/database paths and incomplete/duplicate IDs are rejected; parent directories must exist. Generation conflicts with check/fix/strict/brief. Existing 3-column drift/fix semantics remain, with numeric and canonical slug IDs; no narrative parsing or commit/push.

### Local configuration, context and initialization writes

Config/context/init text writes use atomic publication, reject linked or non-regular targets and unreadable existing text, and refuse changed preparation snapshots. New files never overwrite arrivals. These are per-file operations, not a batch transaction or external-writer lock; context --clear removes the state entry itself.

Preview performs no writes; existing command previews do not inspect every filesystem precondition. Coordinate external writers. `init --write FILE` retains explicit custom paths, including paths outside the checkout; it is not a path-containment boundary. Directory effects reject linked/non-directory targets, but ancestor paths are resolved by the filesystem. New-file publication requires hard-link support on the destination filesystem.

## Controlled history reversal

Explicit recovery: tkt --dry-run undo then tkt undo reverts only a clean HEAD new/claim/close carrying a versioned receipt from this build. Legacy/other operations, root/merge/rebased history and repeated undo are refused. It adds a revert commit, never resets history; push rejection keeps a local commit without automatic rebase.

The first version supports regular non-executable ticket files and an optional already-tracked plan status update. Other effects (configuration, context, init, hooks, telemetry and caches) are outside the inverse. Unrelated untracked files are preserved; the index and tracked worktree must be clean with no active Git operation or index lock. A receipt is structural provenance, not author authentication.

Global --no-push and push.enabled apply. Preview reads without fetching or creating a temporary index. A failed revert leaves conflict/sequencer state for inspection; a failed commit leaves the staged inverse. An opted-in hook that changes the prepared tree leaves a local commit and prevents push. Inspect git status and use explicit reviewed Git recovery for unsupported or interrupted work. Do not undo a valid local commit just because publishing failed.


Append a ticket note with tkt note ID --append "text" (or @FILE / - for UTF-8 stdin, at most 64 KiB). Notes preserve lifecycle and existing body bytes; identify the caller with --agent or TKT_AGENT. The literal ## Resolution is reserved even inside examples. The target must be tracked and clean. Only cooperating note processes in the same worktree serialize; external editors, other mutations and other worktrees require coordination. Dry-run writes nothing. Every invocation appends: after a push fails, inspect the retained local commit and retry git push. If committing fails, inspect and explicitly finish or restore the retained note before retrying; do not append again. Note commits are outside undo receipt eligibility.


## GitHub import

Before importing, read [the import and recovery contract](references/commands.md#github-import).
Preview `tkt import --github --dry-run`; `--repo OWNER/NAME` selects the source.
Bounded extraction never writes to GitHub. Owned snapshots and create-only replay
preserve source and local progress. Keep the tracked worktree/index clean.
Resume a pending journal with the same repository, label, validation arguments and
branch. Shared per-worktree `new`/`batch`/import markers require interruption inspection.


## Completed work reports

Use `tkt done --week` for a rolling UTC window; unknown dates remain visible.
Read [date attribution and limits](references/commands.md#completed-work-reports)
before using its counts. The report adds no stored completion field.

Before configuring lifecycle hooks, read [trust and retry rules](references/commands.md#advisory-lifecycle-hooks).

Native write nudges: read [setup/trust](references/native-hooks.md) before configuration.

Remote previews use cached refs; IDs/publication remain unresolved.

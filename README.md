# Tkt

A git-native ticket tracker where tasks are markdown files and `git push` is the claim protocol.

- **~50ms reads** — tickets are local files, not API calls
- **Single binary** — no runtime dependencies beyond `git`
- **Race-safe** — concurrent sessions get unique IDs automatically
- **Dependency graph** — only shows you what's actually unblocked
- **AI-agent friendly** — structured output, deterministic frontier
- **Zero config** — just `tkt new` in any git repo

## Quick Start

Agents and scripts can discover every command and flag with `tkt capabilities
-o json`, including input schemas, aliases, mutation modes and output formats.
See the [manifest reference](skills/tkt-core/references/capabilities.md).

```bash
cargo install tkt

tkt new auth --title "Implement authentication"
# → ✓ created 01 auth (pushed)

tkt new api --title "Build API" --blocked-by 01
# → ✓ created 02 api (pushed)

tkt ready
# → Ready (1):
# →   01  Implement authentication

tkt close 01 --note "JWT + refresh tokens shipped"
# → ✓ closed 01 auth

tkt ready
# → Ready (1):
# →   02  Build API
```

Dependencies resolve automatically. Close a task and its dependents appear in the frontier.

## Install

### Pre-built binaries (fastest)

```bash
# macOS / Linux
curl -fsSL https://github.com/smileynet/tkt/releases/latest/download/tkt-installer.sh | sh

# Windows (PowerShell)
irm https://github.com/smileynet/tkt/releases/latest/download/tkt-installer.ps1 | iex
```

### From crates.io

Compiling tkt requires Rust 1.85.0 or newer.

```bash
cargo install tkt
```

### With cargo-binstall (pre-built, no compile)

```bash
cargo binstall tkt
```

### From source

Requires Rust 1.85.0 or newer.

```bash
git clone https://github.com/smileynet/tkt.git
cargo install --path tkt
```

### Verify

```bash
tkt --version
# → tkt <version> (<commit>)
```

**Requirement:** `git` on PATH.

### Upgrading skills from v0.4.0

The v0.4.0 installer linked one skill named `tkt` into the Kiro, Claude and Codex
skill roots. The new plugins provide four `tkt-*` skills. Before installing a
plugin or deploying the new skills, remove the old links from a current source
checkout (Node 22+):

```sh
node tools/migrate-legacy-skills.mjs --source /absolute/path/to/old-tkt/skills/tkt
# Inspect the JSON plan, then apply the same source:
node tools/migrate-legacy-skills.mjs --source /absolute/path/to/old-tkt/skills/tkt --execute
```

Use the old link target, even if you upgraded that checkout and `skills/tkt` no
longer exists. The command defaults to a preview and removes only matching symlinks;
it preserves their targets. A regular directory, a link to another source, or a root
outside your home stops cleanup before any changes. Review those cases manually.
Then install the plugin below or run `node tools/deploy.mjs` once for bare skills.
Restart chats to discard previously loaded skills. This migration does not change
legacy steering files or install the Rust CLI.

### Codex plugin

Install the workflow skills from the Codex marketplace:

```sh
codex plugin marketplace add smileynet/tkt
codex plugin add tkt@tkt
```

The plugin supplies four workflow skills; install the `tkt` CLI separately using
one of the methods above.

For local development, replace `smileynet/tkt` with the checkout's absolute path.
After edits, run `node tools/deploy.mjs` once to refresh the cached plugin and harness roots,
then start a new chat. The skill installer skips Codex scan roots when this plugin
is enabled, removing only copies it previously installed. `--allow-dual` overrides
that guard.

### Claude Code and Antigravity

```sh
claude plugin marketplace add smileynet/tkt
claude plugin install tkt@tkt
```

The Claude plugin supplies the four workflow skills and session context from the
launch project's tickets. Its hook requires Claude 2.1.310+ and Node 22+. Install
the CLI separately; plugins do not supply the Rust binary. Codex and Antigravity
receive no Claude hooks.

For a source checkout, `node tools/deploy.mjs` deploys the skills once per supported
harness and builds the Antigravity plugin when configured. Node 22+ is a maintainer
prerequisite, separate from the installed tkt binary's Git requirement.

GitHub marketplace and source-clone examples follow the development default branch.
Stable plugins use the release-only `stable` ref after verified publication; version
fields do not pin a default-branch install. Prebuilt/registry commands install published
versions, not staged candidates. See [release maintenance](docs/releasing.md) for
channels and update boundaries, and restart chats after plugin refresh.

## Usage

### See what's ready to work on

```bash
tkt ready                # human-friendly
tkt ready --json         # machine-readable (JSON Lines)
```

Shows open tasks with all dependencies satisfied, sorted by priority then ID.

### Create tasks

```bash
tkt new fix-login --title "Fix login timeout"
tkt new fix-login --title "Fix login timeout" --priority high
tkt new fix-css --title "Fix CSS overflow" --tags frontend
tkt new deploy --title "Deploy to staging" --blocked-by 01,02
tkt new spike --title "Research caching" --status backlog
tkt new train-model --title "Train ML model" --requires gpu,linux
```

New tasks default to `open` (frontier-eligible). Use `--status backlog` only for work you're deliberately deferring — like the research spike above — not for actionable work you just discovered.

Batch creation for related work:

```bash
tkt batch "api:Build API" "docs:Write docs" "tests:Add tests" --blocked-by 01
```

### Claim and close

Body acceptance checkboxes use `--check-all` / `--ac`; frontmatter validation
criteria use one `--evidence` per index. Those counts are independent.

Create a complete ticket body with `tkt new example --body @brief.md`, `--body -`
for stdin, or literal text. New/batch accept `--template NAME` to read the whole
body verbatim from `.tickets/templates/NAME.md`; no interpolation is applied.
Default section prompts remain detectable as unfilled work by validation.

Claims record the invocation's agent identity as `assignee` when provided.
Ownership is separate from eligibility: another owner blocks claiming unless
`--force`, and ready hides other owners (or all owned tickets when unidentified).
An anonymous forced claim preserves the recorded owner. Query/ready expose ownership.

```bash
tkt claim 03             # mark in-progress (visible to collaborators)
tkt close 03 --note "Deployed to prod"
# --evidence is repeatable — supply one per validation criterion:
tkt close 03 --check-all --evidence "unit tests pass" --evidence "deployed to staging" --resolution "Shipped"
```

`claim` is optional for solo work — `close` works directly on open tasks. Use `claim` in shared repos so others see what's taken. When a ticket has N `validation_criteria`, pass one `--evidence` per criterion (positional in order, or `N=text` to target criterion N).

### Edit tasks

Retire abandoned work with `tkt cancel ID --reason obsolete --note "why"`.
Duplicate/superseded reasons require `--by ID`. Active dependents require explicit
`--repoint` or `--drop-edges`; terminal dependents' obsolete edges are dropped and
named without changing their body/evidence. Inspect `--dry-run` first. Cancelled
is distinct from done and does not satisfy prerequisites; `tkt reopen ID` returns
cancelled work to open with its history retained. Close refuses cancelled work.

Ticket commits skip commit hooks by default; `[commit] hooks = "run"` opts in.
Only affected ticket/plan paths are committed, preserving unrelated staged files.
Push hooks still run. Use global `--no-push` for local filing without fetch/push,
or `[push] enabled = false` persistently; run `tkt rebase` before later pushing
when IDs collide. Push failures name the durable local commit and retry action.

Rebase reports possible stale body references with file/line and ID mapping while
leaving prose unchanged, including in dry-run. Review ambiguous references manually.
Validation warns about missing local `#NNN` / `ticket NNN` citations; qualify external
issues as `owner/repo#NNN` or link them. Quoted code, URLs and headings are excluded.

```bash
tkt edit 02 --title "New title"
tkt edit 02 --blocked-by 01,03
tkt edit 02 --priority high
tkt edit 02 --parent 01           # hierarchy, independent of prerequisites
tkt edit 02 --parent ''           # clear parent
tkt edit 02 --tags backend,auth   # replace tags (empty clears)
tkt edit 02 --add-tag urgent --remove-tag someday
tkt edit 02 --status backlog       # pull from frontier
```

New and batch also accept `--parent ID`. Query emits the stored parent and
`tkt query --parent ID` filters children. Invalid parent targets and introduced
parent cycles are rejected before writing.

Parent, assignee and label-namespace declarations are optional, additive extensions;
existing tickets without them remain valid. Follow the [combined native relations
walkthrough](docs/native-relations.md) to create an effort with children, claim work
under an invocation identity, inspect progress and validate a single-select label
namespace. Parents close explicitly with their own acceptance and evidence.

### Tags & context

```bash
tkt context frontend         # set active context (auto-tags new tickets)
tkt new bugfix --title "Fix CSS" --tags frontend,urgent
tkt context --clear          # clear active context
```

Tags are the primary way to scope work to a subsystem. Set `--tags` when you create a ticket (retro-tagging rarely happens); the active context auto-applies tags to new tickets and scopes `tkt ready` output. One tag per ticket is the norm in multi-stream projects.

### Project health

```bash
tkt validate             # check for cycles, broken refs, contract issues
tkt validate --fix       # auto-repair fixable problems
tkt lint                 # normalize frontmatter; backfill missing numeric IDs from filenames
tkt lint --check         # CI mode: exit 1 if lint needed
tkt doctor               # full health check
tkt blocked              # show tasks stuck on dependencies
tkt sync-plan            # report ticket-vs-plan drift (advisory, exit 0)
tkt sync-plan --check    # CI gate: exit 1 on drift (plan default: docs/plan.md)
```

Consumers can opt into [ticket-policy checks in their own CI](docs/agents/enforcement-model.md).
The recipe is optional; enforcement depends on host settings and permissions.
It installs no workflow or hook and does not replace project-specific tests.

### Query

Claim, close, and status edits update existing `docs/plan.md` rows in their own
commit. Commit or stash plan changes first. Missing rows remain warnings until
`tkt sync-plan --fix` appends them under Unplanned tickets. Without a plan, ticket
mutations create none. Use explicit sync-plan for alternate plan paths.

```bash
tkt query                        # context-scoped tickets as JSON Lines
tkt query --all-context          # bypass context without changing it
tkt query --status open          # filter by status
tkt query --priority high        # filter by priority
```

Query preserves tag context and reports actual narrowing on stderr. `--all-context`
bypasses context while keeping explicit status, priority and parent filters.
JSON includes tags, validation criteria, requirements, ownership and eligibility;
unknown or unparsed stored fields appear as raw strings under `frontmatter`. Query and ready
JSON keep historical `blocked_by` and add `pending_blockers` for missing or non-done
prerequisites. Combining `-q` with `-o json` keeps structured JSON output.

The deep-audit template rule inspects actual intent and acceptance sections. Quoted template
examples are ignored; real empty/default prompts and placeholder criteria still
warn, including checked placeholders. Missing historical evidence remains visible.

### Migrate from other tools

```bash
tkt migrate --detect          # detect current ticket format
tkt migrate --from tk --dry-run  # preview tk-format conversion without writing
tkt migrate --from tk --confirm  # apply the reviewed conversion
```
## Agent eligibility

Identify each invocation with global `--agent <lowercase-slug>` or `TKT_AGENT`; the flag wins and identity is never stored in config. `ready` and `claim` apply ticket agent eligibility. Missing identity sees unrestricted work only; `ready` reports hidden restricted work on stderr. Use `claim <id> --force` only as an explicit scheduling override.

Create or replace eligibility with mutually exclusive `--agents codex,claude` or `--exclude-agents antigravity` on `new`, `batch`, and `edit`; `edit --agents ''` or `edit --exclude-agents ''` clears it. Frontmatter stores one optional signed `agents` list: `["codex", "claude"]` allows either, `["!antigravity"]` excludes that agent, omission allows everyone. Lists must be nonempty, homogeneous in sign, duplicate-free, and contain lowercase alphanumeric/hyphen slugs. Serialization sorts names. `ready` human/JSON and `query` JSON expose the policy (`agents: []` in JSON means unrestricted). `query` honors tag context but never filters by current agent. Tags scope work streams; `requires` remains all-of machine capabilities; eligibility is separate from ownership after claim.


## Task Format

```yaml
---
id: "01"
title: "Implement authentication"
status: open
blocked_by: []
priority: high
tags: [backend, auth]
requires: [gpu]
validation_criteria:
  - "JWT tokens issue correctly (test: auth_test::jwt_issue)"
---

## What to build
JWT-based auth with refresh token rotation.

## Acceptance criteria
- [ ] JWT tokens issued on login
- [ ] Refresh token rotation works
```

Tasks are just files in `.tickets/`. Edit them by hand anytime — tkt reads whatever's there. One exception: don't hand-edit `status` to `done`. Use `tkt close` so the acceptance-criteria, resolution, and evidence gates run (`tkt validate` flags done tickets closed by hand). Generic creation and status-edit commands reject `done`.

## Configuration

Optional. Create `.tickets/config.toml` to customize behavior:

```toml
[push]
enabled = true              # false for local-only repos (no network on writes)

[close]
require_checked_acs = true  # require acceptance criteria checked before close
require_resolution = false  # require --note when closing

[validate]
# Project-specific ratchet: grandfather historical closures below this numeric ID.
missing_resolution_before_id = "36"

[audit]
# Separate resolution-quality cutoff; acceptance/evidence checks still apply.
resolution_baseline_id = "36"

[ready]
default_env = ""            # filter frontier by environment

[new]
default_priority = "medium" # default priority for new tasks

[machine]
capabilities = "gpu,linux"   # capabilities this workstation provides (filters tkt ready)

[labels.triage]
single_select = true
values = ["needs-info", "ready-for-agent"]
```

Label namespace declarations are parsed as project configuration; malformed or
unknown namespace fields are configuration errors. Validation checks declared
tag values and rejects multiple tags in single-select namespaces. The existing narrow configuration syntax requires a single-line,
double-quoted JSON-style string array for `values` (no trailing comma); it is not
a full TOML parser. `tkt validate` also checks parent references for missing targets and cycles.
`tkt config` and `tkt config --help` show the actual user configuration path,
respecting `XDG_CONFIG_HOME` and the operating system's configuration directory.

Use `tkt config --project --set close.require_resolution=true` or
`--project --unset close.require_resolution` for shared settings. Writes validate
known values, serialize native booleans and replace atomically; they do not create
a Git commit. Detached-HEAD project writes warn. Without `--project`, writes keep
the user scope; `--get`/`--list` inspect user values and `--show` inspects project values.

`requires` is the canonical eligibility field. Existing `env: corp` or `env: personal`
tickets remain compatible: tkt treats that value as one required capability. Do not combine
a non-`either` `env` with `requires`; use `requires: [corp, gpu]` when both are needed.
`CREW_ENV` and `ready.default_env` remain legacy ways to supply one capability.

User-level defaults in `~/.config/tkt/config.toml`. Project config overrides.

```bash
tkt config --list          # show project config
tkt config --show          # show resolved config with sources
tkt config --set push.enabled=false
```

## AI Agent Integration

`tkt --help` points to the complete command-schema inventory. `tkt init` prints
agent setup recommendations: persist project instructions with `tkt init --write`,
then install workflow skills separately through the [Codex plugin](#codex-plugin),
[Claude/Antigravity plugins](#claude-code-and-antigravity), or `node tools/deploy.mjs`
from a source checkout. Installing the CLI alone does not install workflow skills.

Works with **Kiro**, **OpenCode**, **Claude Code**, **Codex**, **Cursor**, **Copilot**, and **Windsurf**.

Add to your project's AGENTS.md (or use `tkt init --write` to inject automatically):

```markdown
tkt ready              # see what's unblocked
tkt claim <id>         # mark in-progress
tkt close <id> --check-all --resolution "what was done"
```

Deploy agent instructions for all supported tools at once:

```bash
tkt init --all         # writes AGENTS.md, CLAUDE.md, .cursor/rules/, .kiro/steering/, etc.
```

Solo agent: `tkt ready` → pick first → `tkt close <id> --check-all --resolution "..."`.

Multi-agent: `tkt ready` → `tkt claim <id>` → work → `tkt close <id>`.

Structured output everywhere: `tkt ready --json`, `tkt query`, `tkt validate --brief`.

## Environment Variables

| Variable | Effect |
|----------|--------|
| `CREW_ENV` | Filter frontier by environment (legacy; prefer `machine.capabilities` config for new projects) |
| `TKT_ASCII=1` | ASCII-only symbols (✓→\[ok\], ✗→\[err\]) |
| `NO_COLOR=1` | Disable ANSI color |
| `TKT_ADVICE=0` | Disable advisory hints (e.g. the batch nudge) |
| `TKT_DEBUG=1\|true\|log\|file` | Debug output to the local debug log |
| `TKT_DEBUG=stderr\|human` | Human-readable debug output to stderr |
| `TKT_DEBUG=json` | Debug output as JSON Lines on stderr |
| `DO_NOT_TRACK=1` | Disable telemetry and update-check requests/cache refresh |

User configuration `debug = true` also defaults to the log file. Set
`debug.format = "human"` or `"json"` explicitly for stderr output.

Known policy values are parsed strictly: booleans accept true/false/1/0/yes/no,
evidence accepts true/warn/false, and priority/environment defaults use their
declared values. Invalid effective settings report the key, value and source;
project values override user defaults. Unknown keys retain warning compatibility.

Skipped unparseable tickets remain nonfatal diagnostics. With `-o json`, each
warning is a JSON object on its own stderr line (`warning.kind`, `path`, `message`),
separate from result stdout. Human warnings honor ASCII mode. Empty results with
skip warnings differ from an empty corpus; exit 0 does not establish corpus health.

## Local dashboard

Share the delivery dashboard on an explicit LAN interface with
`node tools/serve-dashboard.mjs --host 192.168.0.187 --port 8765`.
See [LAN hosting](docs/dashboard-lan.md) for the address, served-file scope and
process lifecycle. The default interface is loopback when `--host` is omitted.

## Contributing

Found a bug? [File a report](https://github.com/smileynet/tkt/issues/new?template=bug_report.md).
Want a feature? [Request it](https://github.com/smileynet/tkt/issues/new?template=feature_request.md).

Before each code commit, run `mise run check`. The former automatic push/PR CI
workflow has been removed; remaining hosted workflows are manual-only. Local
pre-push checks are an opt-in proposal tracked in ticket 249, not installed by default.

## Release maintenance

[Release staging](docs/release-staging.md) covers the pinned generator and manual
artifact builds. [Release maintenance](docs/releasing.md) covers preparation,
verification, the sole reviewed CI publisher and recovery. Maintainers need Node
22+, locked npm dependencies, mise, Rust and cargo-dist 0.32.0. Publication requires
separate authorization after candidate, install and final audit gates.

## License

[MIT](LICENSE)

## Parent progress

Parents with any direct children are containers, excluded from ready. Use `tkt children ID` for full-corpus direct-child progress (done/total includes cancelled in total, with cancelled separate); `-o json` emits one object and `-q` lists child IDs. Progress never changes stored status: close parents explicitly with normal AC/evidence gates.

## Transferring acceptance criteria

Move an unchecked body criterion with `tkt move-ac SOURCE --ac N --to DEST` (N counts all top-level single-line body ACs, including checked/moved). The source uses `[>]` plus a destination marker; the receiver gets an unchecked copy with a source back-reference. This records transfer, not completion. Validation criteria and evidence remain independent and unchanged; supply truthful proof for every retained validation criterion at close. Retry the same transfer after interruption; inspect conflicts instead of editing link identities.

## Parent-scoped frontier

Use `tkt ready --parent ID` for ready direct children of an existing parent, preserving priority/ID order, capability matching, invocation eligibility/ownership and tag context (including its existing untagged-ticket behavior). Human in-progress context is parent-scoped too. JSON rows include stored parent when present. This is native-parent-only; no epic-tag alias is added. Legacy tag groups can continue using `tkt context epic:slug` with normal ready.

## Plan lifecycle comparison

Plan tables compare all five lifecycle labels exactly. Both `done` and canonical `✅ done` are recognized; cancellation rows retain their reason. Unknown status cells are findings rather than silently treated as open. Plain sync-plan remains advisory for derivable drift; --check or --strict fails, while --fix regenerates cells without managing narrative prose.

Update-check cadence is user-only: `tkt config --set update_check.interval=1h` (also `24h`, `7d`, or `never`; positive whole hours/days). Default: `24h`. `TKT_UPDATE_CHECK_INTERVAL` overrides the setting; `TKT_UPDATE_CHECK=0`, `DO_NOT_TRACK=1`, CI, quiet and dry-run still suppress update-check requests/cache writes. Invalid intervals are rejected on config writes; a malformed stored/environment interval skips the advisory check with a warning.

Source-reference readiness: `validate` reports `no-source-reference` at info severity for open/in-progress tickets lacking a nonempty spec, prerequisite, parent, local ticket citation, docs/specs/.memory path or HTTP(S) link. Fenced examples do not count. Backlog and terminal tickets are excluded. It remains advisory even under `--strict`; `[validate] require_source_reference = true` explicitly makes absence an error. Existing integrity rules still validate referenced targets separately.

Frontier readiness: `validate` emits info-only `frontier-readiness` for missing source reference, non-moved/nonempty acceptance checkbox or validation-criterion slots on the project structural frontier. It reuses the source predicate and native dependency/container/capability selection, with the same configured machine/legacy-env capability resolution as ready. Validation is project-wide: invocation ownership, agent eligibility, tag context and ready --parent are views and do not narrow this audit. Info remains advisory under --strict; no new claim/close gate, queue filter or sort is added. These are slot checks, not proof that the contents are sufficient. Backlog-chain diagnosis is outside this first bounded rule.

Optional Git advice: `tkt hooks install|check|remove|run` manages only the exact owned project pre-commit shim, shared by repository worktrees. Install/remove are explicit, respect global dry-run and preserve foreign/modified hooks or any configured core.hooksPath. The shim always permits committing, even when validation or tkt fails; it is not the verification gate. No hook is installed by init/deploy, no Git config or commit/push changes, and normal tkt mutation hook bypass remains unchanged.

See [optional hook installation and recovery](docs/optional-hooks.md).

Derived plans: `tkt sync-plan --generate [FILE]` renders the full corpus in priority/ID order (all statuses, no tag/agent narrowing). No file means Markdown stdout; -o json wraps projection/count. An explicit new/empty file or one owned unfenced tkt:projection:begin/end block is atomically written; human text outside stays exact. Global dry-run writes nothing. Unmarked nonempty files, bad markers, output links/database paths and incomplete/duplicate IDs are rejected; parent directories must exist. Generation conflicts with check/fix/strict/brief. Existing 3-column drift/fix semantics remain, with numeric and canonical slug IDs; no narrative parsing or commit/push.

See [derived plan ownership and preview](docs/plan-projection.md).

Dry-run diagnostics: file debug logging is suppressed for previews, including the local validate/rebase dry-run forms. Existing logs are neither created nor appended. Explicit human/JSON stderr debug output remains available; ordinary non-preview file logging is unchanged. This repairs the documented no-write preview behavior.

Migration safety: tk preview and apply prepare readable source snapshots and reject destination or backup conflicts. New output/backups are published completely without overwriting arrivals; an original is removed only after its converted output succeeds. Backups remain .tickets.bak. This is per-file safety, not an atomic batch or external-editor lock; retain backups and inspect partial results on failure. Hard-link support is required for new-file publication.

See [migration publication and recovery](docs/migration-publication.md).

### Local configuration, context and initialization writes

Config/context/init text writes use atomic publication, reject linked or non-regular targets and unreadable existing text, and refuse changed preparation snapshots. New files never overwrite arrivals. These are per-file operations, not a batch transaction or external-writer lock; context --clear removes the state entry itself.

Preview performs no writes; existing command previews do not inspect every filesystem precondition. Coordinate external writers. `init --write FILE` retains explicit custom paths, including paths outside the checkout; it is not a path-containment boundary. Directory effects reject linked/non-directory targets, but ancestor paths are resolved by the filesystem. New-file publication requires hard-link support on the destination filesystem.

## Controlled history reversal

Explicit recovery: tkt --dry-run undo then tkt undo reverts only a clean HEAD new/claim/close carrying a versioned receipt from this build. Legacy/other operations, root/merge/rebased history and repeated undo are refused. It adds a revert commit, never resets history; push rejection keeps a local commit without automatic rebase.

The first version supports regular non-executable ticket files and an optional already-tracked plan status update. Other effects (configuration, context, init, hooks, telemetry and caches) are outside the inverse. Unrelated untracked files are preserved; the index and tracked worktree must be clean with no active Git operation or index lock. A receipt is structural provenance, not author authentication.

Global --no-push and push.enabled apply. Preview reads without fetching or creating a temporary index. A failed revert leaves conflict/sequencer state for inspection; a failed commit leaves the staged inverse. An opted-in hook that changes the prepared tree leaves a local commit and prevents push. Inspect git status and use explicit reviewed Git recovery for unsupported or interrupted work. Do not undo a valid local commit just because publishing failed.

See [undo eligibility and recovery](docs/undo.md).

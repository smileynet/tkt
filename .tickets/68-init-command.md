---
id: "68"
title: "Add tkt init command with per-agent scaffolding"
status: open
blocked_by: ["01"]
priority: medium
---

# Add tkt init command with per-agent scaffolding

## What to build

A `tkt init` subcommand that scaffolds both the project structure and agent integration files. The command handles two concerns:

1. **Project bootstrapping** — create `.tickets/` and `.tickets/config.toml`
2. **Agent instruction deployment** — emit/write a marker-wrapped snippet for AI coding tools

### Core command

```
tkt init [FLAGS]
```

| Flag | Effect |
|------|--------|
| (none) | Create .tickets/ + config.toml + print agent snippet to stdout |
| `--agent-only` | Skip directory/config creation, only output agent snippet |
| `--write [FILE]` | Write snippet into FILE (default: AGENTS.md) using markers |
| `--target <tool>` | Generate for a specific tool (see per-agent specs below) |
| `--all` | Generate for all known tool locations |

### Marker format

```markdown
<!-- tkt:begin -->
## Tickets
...content...
<!-- tkt:end -->
```

On re-run with `--write`: find markers, replace between them. If no markers found, append. Never touch content outside markers.

### Canonical snippet (tool-agnostic)

```markdown
## Tickets

This project uses [tkt](https://github.com/smileynet/tkt) for work tracking. Tickets live in `.tickets/`.

tkt ready                                         # what to work on next
tkt claim <id>                                    # mark as in_progress (shared repos)
tkt close <id> --check-all --resolution "..."     # mark done
tkt validate --brief                              # check for issues

### Workflow

Single-agent: `tkt ready` → `tkt close <id> --check-all --resolution "..."`
Shared-repo: `tkt ready` → `tkt claim <id>` → work → `tkt close <id> --check-all --resolution "..."`

If a claim push is rejected, someone else got there first — pick the next frontier ticket.
```

---

## Per-Agent Specifics

### AGENTS.md (default, `--target agents`)

**File:** `AGENTS.md` (root)
**Read by:** Codex, Cursor, Copilot, Gemini CLI, Kiro CLI, Aider, Warp, Zed, Junie
**Format:** Plain markdown, marker-wrapped
**Content:** The canonical snippet above

This is the primary target. Widest compatibility, simplest format.

### Claude Code (`--target claude`)

**File:** `CLAUDE.md` (root)
**Read by:** Claude Code only
**Why separate:** Claude Code does NOT natively read AGENTS.md (requires explicit @-import). Users who primarily use Claude Code need this.
**Format:** Plain markdown, marker-wrapped
**Content:** Same canonical snippet, but add a note:

```markdown
<!-- tkt:begin -->
## Tickets

This project uses tkt for work tracking. Run `tkt ready` to see what's available.

Commands: ready, claim <id>, close <id> --check-all --resolution "...", validate --brief

Workflow: tkt ready → close <id> --check-all --resolution "done: what was shipped"
For shared repos: tkt ready → claim <id> → work → close <id> --check-all --resolution "..."
<!-- tkt:end -->
```

Shorter because Claude Code users typically have richer project context already loaded.

### Cursor (`--target cursor`)

**File:** `.cursor/rules/tkt.mdc`
**Read by:** Cursor only
**Why separate:** Cursor's modern format uses `.mdc` files with frontmatter for activation conditions.
**Format:**

```markdown
---
description: tkt ticket management workflow
alwaysApply: true
---

# tkt Workflow

This project uses tkt for work tracking (.tickets/ directory).

## Commands
- `tkt ready` — see unblocked tickets (frontier)
- `tkt claim <id>` — mark in_progress (shared repos only)
- `tkt close <id> --check-all --resolution "..."` — mark done
- `tkt validate --brief` — check for issues

## Workflow
1. `tkt ready` → pick the first listed ticket
2. Read the ticket file completely
3. Do the work described
4. Verify acceptance criteria
5. `tkt close <id> --check-all --resolution "what was done"`
```

### GitHub Copilot (`--target copilot`)

**File:** `.github/copilot-instructions.md`
**Read by:** GitHub Copilot only
**Why separate:** Copilot reads this path for repo-level instructions. Some teams already have this file; tkt appends with markers.
**Format:** Plain markdown, marker-wrapped (same as AGENTS.md canonical)

### Kiro CLI (`--target kiro`)

**File:** `.kiro/steering/tkt.md`
**Read by:** Kiro CLI only
**Why separate:** Kiro uses steering files in `.kiro/steering/` as persistent guidance. This is already how tkt integrates with kiro in practice (via the frontier-work steering).
**Format:** Plain markdown (no markers needed — tkt owns the whole file)
**Content:**

```markdown
# tkt Integration

When .tickets/ exists, work the frontier.

## Commands
tkt ready              # frontier (open + deps done + env match)
tkt claim <id>         # status → in_progress, pushed
tkt close <id> --check-all --resolution "..."  # mark done
tkt validate --brief   # check for issues

## Workflow
Single-agent: tkt ready → close <id> --check-all --resolution "..."
Shared-repo: tkt ready → claim <id> → work → close <id>

## Frontier Rule
Pick the first ticket `tkt ready` lists — it already applies priority sorting
(urgent > high > medium > low) then lowest-number-first.
```

### Codex (`--target codex`)

**File:** `AGENTS.md` (same as default — Codex originated the AGENTS.md convention)
**Read by:** OpenAI Codex
**Why no separate file:** Codex reads AGENTS.md natively and walks subdirectories. The default target IS the Codex target.
**Note:** If a `.codex/` directory exists, could additionally write `.codex/AGENTS.md` for directory-scoped instructions, but this is unnecessary for v1.

### Windsurf (`--target windsurf`)

**File:** `.windsurf/rules/tkt.md`
**Read by:** Windsurf/Codeium
**Format:** Markdown with frontmatter:

```markdown
---
trigger: always_on
---

# tkt Workflow

This project uses tkt for work tracking. Tickets in .tickets/.

Commands: ready, claim <id>, close <id> --check-all --resolution "...", validate --brief
Workflow: tkt ready → close <id> --check-all --resolution "done: description"
```

### `--all` flag

Generates all of the above in one shot:
- AGENTS.md (with markers)
- CLAUDE.md (with markers)
- .cursor/rules/tkt.mdc
- .github/copilot-instructions.md (with markers, only if .github/ exists)
- .kiro/steering/tkt.md
- .windsurf/rules/tkt.md

Creates parent directories as needed. Reports what was written.

---

## Implementation notes

- The canonical snippet is a const string embedded in the binary (no templates needed — tkt is too simple for Go-style template rendering)
- Per-agent variants are minimal wrappers around the canonical content
- `--write` without a path defaults to AGENTS.md
- Marker parsing: find `<!-- tkt:begin -->`, find `<!-- tkt:end -->`, replace between (inclusive of markers). If not found, append with a blank line separator.
- Idempotent: safe to run multiple times. Creates directories only if missing, skips config if exists, updates markers in-place.
- Print a summary of actions taken (created/updated/skipped for each file)

### Detection strategy: promiscuous install (no detection)

**Decision:** `tkt init --all` writes ALL tool-specific files unconditionally. No agent detection.

**Rationale (from beads reference study):**

Beads uses a two-layer model:
1. `bd init` — always installs Claude/Codex/Cursor files with NO detection (non-fatal on failure)
2. `bd doctor` — detects active agents via env vars, PATH, config files (diagnostic only)
3. `bd setup <tool>` — explicit per-tool install for 14 supported tools

Their detection signals include `CLAUDECODE=1` env var, `which claude` PATH check, `~/.claude/` existence, JSON config parsing for plugins/MCP/hooks. This is expensive, fragile (env vars can be set by other tools), and only used for diagnostics — never for deciding what to install.

**Why tkt doesn't need detection:**
- Our per-agent files are 10-20 lines each (vs beads' hooks/JSON/plugins)
- Dormant files have zero cost if the tool isn't installed
- Detection adds complexity (env vars, PATH checks, file sniffing) for zero practical benefit
- Users who run `--all` explicitly accept all files; users who run `--target cursor` know what they want

**The only detection tkt needs:** check if target files already exist (for idempotent marker updates). This is file-existence, not agent-presence detection.

### `--all` behavior

Writes all targets unconditionally. Each write is non-fatal — if a directory can't be created (permissions), warn and continue. Report a summary:

```
✓ created .tickets/
✓ created .tickets/config.toml
✓ updated AGENTS.md (tkt section)
✓ created CLAUDE.md (tkt section)
✓ created .cursor/rules/tkt.mdc
✓ created .kiro/steering/tkt.md
✓ skipped .github/copilot-instructions.md (.github/ doesn't exist)
✓ created .windsurf/rules/tkt.md
```

Note: for `.github/copilot-instructions.md`, only write if `.github/` already exists (don't create a `.github/` directory just for this — it implies a GitHub repo structure the user may not want).


## Acceptance criteria

- [ ] `tkt init` creates `.tickets/` and `.tickets/config.toml` when missing
- [ ] `tkt init` prints canonical agent snippet to stdout
- [ ] `tkt init --write` appends/updates AGENTS.md with marker-wrapped section
- [ ] `tkt init --write` is idempotent (re-run updates, doesn't duplicate)
- [ ] `tkt init --agent-only` skips directory/config creation
- [ ] `tkt init --target claude` writes CLAUDE.md
- [ ] `tkt init --target cursor` writes .cursor/rules/tkt.mdc with frontmatter
- [ ] `tkt init --target kiro` writes .kiro/steering/tkt.md
- [ ] `tkt init --target copilot` writes .github/copilot-instructions.md (with markers)
- [ ] `tkt init --target windsurf` writes .windsurf/rules/tkt.md with frontmatter
- [ ] `tkt init --all` generates all targets
- [ ] Parent directories created automatically for tool-specific paths
- [ ] Existing user content outside markers preserved on re-run
- [ ] Integration tests cover create, update, and idempotency scenarios

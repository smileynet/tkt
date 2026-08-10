---
id: "69"
title: "Explore agent/harness-specific integrations (hooks, MCP, skills)"
status: backlog
blocked_by: ["68"]
priority: low
---

# Explore agent/harness-specific integrations (hooks, MCP, skills)

## What to build

Explore deeper integrations with specific AI coding agents beyond the static markdown snippets that `tkt init` provides. The question: does tkt benefit from hooks, MCP servers, plugins, or skills that go beyond "here are the commands"?

### Areas to explore

#### 1. Session-start hooks (Claude Code, Codex, Cursor)

Could tkt inject frontier context automatically when an agent starts a session?

```
# Hypothetical hook output
tkt ready --json | head -3   # inject top-3 frontier tickets as context
```

Tools that support session-start hooks:
- **Claude Code**: `.claude/settings.json` → `hooks.SessionStart`
- **Codex**: `.codex/hooks.json` → `SessionStart`
- **Cursor**: `.cursor/hooks.json` → `sessionStart`

**Question:** Is this valuable? The agent can already run `tkt ready` itself. A hook saves one round-trip but adds complexity. Beads does this because `bd prime` outputs 50-2000 tokens of workflow context — tkt's frontier is typically <10 lines.

#### 2. MCP server

An MCP server exposing tkt operations as tools:
- `tkt_ready` → returns frontier as structured data
- `tkt_close` → marks ticket done
- `tkt_show` → reads a ticket file with metadata

**Question:** What does MCP buy over shell-out? Agents already shell out to `tkt`. MCP would add structured responses (no parsing needed) and tool discovery. But it also adds a Python/Node dependency for the server, or requires tkt to embed an MCP server (scope creep).

#### 3. Agent skills / plugins

Beads publishes a `.agents/skills/beads/SKILL.md` that defines an autonomous workflow loop. tkt could similarly provide a skill:

```markdown
---
name: tkt-workflow
description: Work the frontier — pick, claim, implement, close tickets
trigger: always
---

# tkt Workflow Skill

1. Run `tkt ready` to see the frontier
2. Pick the first ticket listed
3. Read the ticket file completely
4. Implement what's described
5. Verify acceptance criteria
6. Run `tkt close <id> --check-all --resolution "..."`
```

**Question:** Is this a tkt concern or a consumer concern? The spellbook adapter already defines this workflow. Shipping it from tkt itself would make it available to users who don't use spellbook.

#### 4. Completion/compaction hooks

Some agents fire hooks before context compaction. tkt could use this to persist state:
- Save current ticket ID being worked on
- Emit a "resume point" that survives compaction

**Question:** Probably over-engineering. The ticket file itself IS the resume point.

#### 5. Native tool integration patterns

| Agent | Native integration | What tkt could provide |
|-------|-------------------|----------------------|
| Claude Code | Plugin (marketplace) | Slash commands: `/tkt ready`, `/tkt close` |
| Codex | Native hooks + AGENTS.md | SessionStart hook injecting frontier |
| Cursor | .mdc rules + hooks | Rule that activates when .tickets/ is present |
| Kiro CLI | Skills (SKILL.md) | A deployable skill defining the workflow |
| Copilot | Extensions | VS Code extension (very heavy) |

### Evaluation criteria

For each integration explored:
1. **Does it save the agent meaningful work?** (vs just running `tkt ready`)
2. **Does it require a runtime dependency?** (MCP server = Python/Node process)
3. **Is it tkt's job or the consumer's?** (spellbook adapter vs tkt-native)
4. **Maintenance burden** — each agent's hook/plugin format changes over time

### Prior art

- **beads**: Full MCP server (Python, PyPI), Claude plugin (marketplace), session hooks for 3 agents, agent skill. Justified by beads' complex state (Dolt DB, sync, memories).
- **spellbook adapter**: Already defines tkt's workflow as a wayfinder adapter — mapping operations to CLI commands.
- **frontier-work steering**: Already provides the "how to work tickets" guidance for kiro-cli.

### Recommendation (preliminary)

Start with the **skill file** (lowest cost, highest portability). A single `SKILL.md` shipped by `tkt init` covers Claude Code, Cursor, Codex, and Kiro. Hooks and MCP are probably not worth it until tkt has state that benefits from automatic injection (e.g., "you were working on ticket 03" context after compaction).

## Acceptance criteria

- [ ] Research spike completed: document which integrations are worth building
- [ ] Decision recorded (ADR or ticket update) with rationale
- [ ] If skill approach chosen: ship SKILL.md generation in `tkt init`
- [ ] If hooks chosen: implement for at least one agent as proof of concept

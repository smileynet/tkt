---
id: "36"
title: "agent command discovery: guidance for projects adopting tkt"
status: in_progress
blocked_by: []
---

# Agent command discovery: guidance for projects adopting tkt

## Problem

When AI agents (kiro-cli, codex, agy) encounter a project with `.tickets/`, they need to know:
1. That `tkt` is available on PATH
2. What commands are relevant to their current task
3. How to integrate tkt into their workflow (frontier → claim → work → close)
4. What flags to use (--check-all, --resolution, -q for scripting)

Currently this knowledge lives in:
- Global steering (`frontier-work.md`) — tells agents to use `tkt ready`
- Per-project AGENTS.md — lists available commands
- `tkt --help` — full command reference

But agents don't always read AGENTS.md before acting, and `tkt --help` dumps everything at once without workflow context.

## Research questions

1. **How do other CLI tools support agent/automation discovery?** (e.g., `gh` has `gh api`, structured JSON output, completion scripts)
2. **Should tkt provide a machine-readable capability manifest?** (e.g., `tkt capabilities --json` that lists commands, flags, and typical workflows)
3. **What steering/skill updates would help agents use tkt effectively?** (e.g., a tkt-workflow skill that activates when .tickets/ is detected)
4. **Should there be a per-project `.tickets/config.toml`** that declares project-specific defaults (required resolution text, AC enforcement level)?

## Proposed deliverables

### A. Skill: tkt-workflow

A deployable skill (for `~/.kiro/skills/`) that activates when `.tickets/` is detected. Provides:
- The JTBD-aware close workflow (--check-all --resolution)
- Frontier-first work selection
- Debug/telemetry awareness
- Common flag combinations for agent use

### B. Agent guidance in AGENTS.md template

A recommended AGENTS.md section for projects adopting tkt:

```markdown
## Tickets
tkt ready                           # what to work on next
tkt claim <id>                      # mark as in_progress
tkt close <id> --check-all --resolution "what was done"  # mark done
tkt validate --brief                # check for issues
TKT_DEBUG=1 tkt <cmd>              # diagnose problems
```

### C. Machine-readable discovery (optional)

```bash
tkt capabilities --json
# → {"commands":["ready","new","claim","close",...], "flags":{"close":["--check-all","--resolution","--force"]}, "version":"0.1.0"}
```

This enables agents to introspect available features without parsing --help text.

### D. Per-project config (`.tickets/config.toml`)

```toml
[close]
require_resolution = true    # error if --resolution not provided
check_all_default = false    # don't auto-check (require explicit --check-all)

[validate]
strict = false               # default strictness for CI
```

## Acceptance criteria

- [ ] Research: how do agents currently discover and use tkt (observe 3+ sessions)
- [ ] Decision: which deliverables (A/B/C/D) to implement
- [ ] Skill authored (if A selected)
- [ ] AGENTS.md template documented (if B selected)
- [ ] Implementation complete for selected deliverables
- [ ] Tested in at least one agent session

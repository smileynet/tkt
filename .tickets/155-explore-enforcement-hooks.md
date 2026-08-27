---
id: "155"
title: "Explore commit and agent hooks to enforce close protocol (beads-style)"
status: backlog
blocked_by: []
priority: high
validation_criteria:
  - "spike output: recommended hook architecture (pre-commit + agent lifecycle) with tradeoffs vs current detection-only approach"
  - "decision recorded: which hooks tkt should ship, how tkt init installs them, opt-out story"
tags: ["contract"]
---

# Explore commit and agent hooks to enforce close protocol (beads-style)

## What to build

Spike: evaluate git commit hooks and agent-lifecycle hooks as a stronger enforcement layer for the close protocol, beyond the detection-only approach in #154. Detection (validate/audit) catches a hand-flipped done ticket *after* it's committed; hooks could catch it *at* commit or *during* the agent session.

Reference implementation: **beads** (https://github.com/gastownhall/beads) ships this model:
- `.githooks/` directory with committed hooks
- `bd setup <agent>` installs "skill, AGENTS.md guidance, AND hooks" per agent (codex, claude, factory, cursor, mux)
- `bd hooks install` as an explicit step; per-agent Claude/Codex hook integrations
- `--stealth` / `no-git-ops: true` opt-out for environments where hooks aren't wanted

## Questions to answer

1. **Pre-commit hook:** on commit, scan staged `.tickets/*.md` for `status: done` changes that lack a `## Resolution` section (i.e., not produced by `tkt close`). Reject or warn? How does it interact with tkt's own commits (which ARE produced by close)?
2. **Agent-lifecycle hooks:** what do Claude Code / Codex hook points offer (pre-tool-use, post-edit)? Could a hook intercept a Write to a `.tickets/` file that flips status?
3. **Installation:** should `tkt init` install hooks (like `bd setup`)? Opt-in or opt-out? How to avoid clobbering existing hooks (chaining, `core.hooksPath`)?
4. **Opt-out story:** local-only repos, CI, editors — mirror beads' `--stealth`/`no-git-ops`.
5. **Cross-platform:** hooks must work on Windows (this project's primary env) — bash hook portability.

## Deliverable

Spike write-up in `.scratch/` or a follow-up spec: recommended hook architecture, tradeoffs vs #154's detection-only approach, and a decision on whether/what tkt ships.

## Context

- **Relevant files:** `src/commands/init.rs` (where hook install would live), `tools/deploy-skills.sh` (existing deploy pattern), `.memory/agent-guidance-surfaces.md`
- **Depends conceptually on #154** — hooks are the prevention layer; #154 is the detection layer. Do #154 first (cheap, high-value); this spike decides if prevention is worth the complexity.
- **Constraint:** files are the database and hand-editing is explicitly supported — hooks must not break legitimate manual edits, only catch the specific done-without-close pattern

## Acceptance criteria

- [ ] spike output: recommended hook architecture (pre-commit + agent lifecycle) with tradeoffs vs detection-only
- [ ] decision recorded: which hooks tkt should ship, how tkt init installs them, opt-out story
- [ ] beads' approach documented as prior art (what to borrow, what to skip)
- [ ] Windows hook portability assessed

## Out of scope

- Implementing the hooks (this is a spike — implementation is a follow-up ticket)

---
id: "155"
title: "Explore commit and agent hooks to enforce close protocol (beads-style)"
status: backlog
blocked_by: []
priority: high
validation_criteria:
  - "spike output: recommended hook architecture (pre-commit + agent lifecycle) with tradeoffs vs current detection-only approach"
  - "recommendation on whether/how to block direct agent writes to .tickets/ (deny-rules vs frontmatter-only vs warn), per agent"
  - "decision recorded: which hooks tkt should ship, how tkt init installs them, opt-out story"
tags: ["contract"]
---

# Explore commit and agent hooks to enforce close protocol (beads-style)

## What to build

Spike: evaluate git commit hooks and agent-lifecycle hooks/permissions as a stronger enforcement layer for the close protocol, beyond the detection-only approach in #154. Detection (validate/audit) catches a hand-flipped done ticket *after* it's committed; hooks could catch it *at* commit, *during* the agent session, or prevent the direct write entirely by routing all `.tickets/` frontmatter mutations through the `tkt` CLI.

Reference implementation: **beads** (https://github.com/gastownhall/beads) ships this model:
- `.githooks/` directory with committed hooks
- `bd setup <agent>` installs "skill, AGENTS.md guidance, AND hooks" per agent (codex, claude, factory, cursor, mux)
- `bd hooks install` as an explicit step; per-agent Claude/Codex hook integrations
- `--stealth` / `no-git-ops: true` opt-out for environments where hooks aren't wanted

## Questions to answer

1. **Pre-commit hook:** on commit, scan staged `.tickets/*.md` for `status: done` changes that lack a `## Resolution` section (i.e., not produced by `tkt close`). Reject or warn? How does it interact with tkt's own commits (which ARE produced by close)?
2. **Agent-lifecycle hooks:** what do Claude Code / Codex hook points offer (pre-tool-use, post-edit)? Could a hook intercept a Write to a `.tickets/` file that flips status?
3. **Block direct agent writes to `.tickets/`:** should agents be prevented from writing to `.tickets/*.md` at all via their editor tool, forcing all mutations through the `tkt` CLI? Explore per-agent permission mechanisms:
   - Claude Code: `permissions.deny` / `Write(.tickets/**)` deny-rules in settings, or a `PreToolUse` hook that rejects Write/Edit tool calls targeting `.tickets/`
   - Codex / Cursor / others: equivalent deny-list or pre-tool hooks
   - **Nuance to resolve:** a blanket deny is too blunt — creating/editing ticket *bodies* by hand is legitimate and supported ("body is user-owned"). The target is specifically *frontmatter mutations* (status/blocked_by/priority) that have a CLI command. Options: (a) deny all `.tickets/` writes and require CLI for everything (simplest, most restrictive — breaks legit body edits); (b) deny only when the diff touches frontmatter fields tkt owns (precise, needs a smarter hook); (c) warn-not-block on any `.tickets/` write, nudging toward the CLI. Recommend which.
   - How does `tkt init --<agent>` install these deny-rules/hooks alongside the AGENTS.md snippet (like `bd setup`)?
4. **Installation:** should `tkt init` install hooks (like `bd setup`)? Opt-in or opt-out? How to avoid clobbering existing hooks (chaining, `core.hooksPath`)?
5. **Opt-out story:** local-only repos, CI, editors — mirror beads' `--stealth`/`no-git-ops`.
6. **Cross-platform:** hooks must work on Windows (this project's primary env) — bash hook portability.

## Deliverable

Spike write-up in `.scratch/` or a follow-up spec: recommended hook architecture, tradeoffs vs #154's detection-only approach, and a decision on whether/what tkt ships.

## Context

- **Relevant files:** `src/commands/init.rs` (where hook install would live), `tools/deploy-skills.sh` (existing deploy pattern), `.memory/agent-guidance-surfaces.md`
- **Depends conceptually on #154** — hooks are the prevention layer; #154 is the detection layer. Do #154 first (cheap, high-value); this spike decides if prevention is worth the complexity.
- **Constraint:** files are the database and hand-editing is explicitly supported — hooks must not break legitimate manual edits, only catch the specific done-without-close pattern

## Acceptance criteria

- [ ] spike output: recommended hook architecture (pre-commit + agent lifecycle) with tradeoffs vs detection-only
- [ ] recommendation on blocking direct agent writes to `.tickets/` (blanket deny vs frontmatter-only vs warn-only), with per-agent mechanism (Claude deny-rules/PreToolUse, Codex/Cursor equivalents)
- [ ] decision recorded: which hooks tkt should ship, how tkt init installs them, opt-out story
- [ ] beads' approach documented as prior art (what to borrow, what to skip)
- [ ] Windows hook portability assessed

## Out of scope

- Implementing the hooks (this is a spike — implementation is a follow-up ticket)

# Agent Guidance Surfaces

tkt teaches agents how to use it through multiple documents. When CLI behavior changes (new flags, status semantics, close requirements), **all surfaces must be updated together**.

## Surfaces (sync required)

| Surface | Location | Audience | Loaded when |
|---------|----------|----------|-------------|
| **Init snippets** | `src/commands/init.rs` (embedded constants) | External users' agents | `tkt init` / `tkt init --all` |
| **Skill (SKILL.md)** | `skills/tkt-core/SKILL.md` | tkt's own agents (kiro, claude, codex, agy) | Skill activation via triggers |
| **Skill references** | `skills/tkt-core/references/*.md` | tkt's own agents (lazy-loaded) | Agent reads reference link |
| **Workflow skills** | `skills/tkt-{orient,work,reconcile}/SKILL.md` | tkt's own agents (cross-harness + agy plugin) | Skill activation via triggers |
| **Plugin manifest** | `plugin.json` | Antigravity (`agy`) & Agent Plugins consumers | Packaged by `tools/deploy-skills.sh` |
| **AGENTS.md (tkt CLI section)** | `AGENTS.md` | Contributors to tkt itself | Codex/agent reads project |
| **README (Usage section)** | `README.md` | Humans browsing crate/GitHub | Manual reading |

## What each surface covers

| Topic | Init snippets | SKILL.md | commands.md ref | tkt-work | AGENTS.md | README |
|-------|:---:|:---:|:---:|:---:|:---:|:---:|
| Status lifecycle | ✓ | ✓ | — | — | ✓ | ✓ |
| Backlog workflow | ✓ | ✓ | — | ✓ | — | — |
| Close with evidence | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| All flags/options | — | — | ✓ | — | ✓ | — |
| Enforcement config | — | — | ✓ | — | ✓ | ✓ |
| Priority sort | ✓ | ✓ | — | ✓ | ✓ | ✓ |
| Env filtering | — | ✓ | ✓ | — | ✓ | — |
| Requires / capabilities | — | ✓ | ✓ | — | ✓ | ✓ |
| Race detection | ✓ | ✓ | — | ✓ | ✓ | ✓ |
| Ticket creation quality | — | link | — | — | — | — |
| Health commands | — | ✓ | ✓ | — | ✓ | ✓ |
| Tag context | — | — | ✓ | ✓ | ✓ | ✓ |
| Migration | — | — | ✓ | — | ✓ | ✓ |
| Validation criteria | — | — | ✓ | — | ✓ | ✓ |
| Batch creation | — | ✓ | ✓ | — | ✓ | ✓ |
| Advisory hints (TKT_ADVICE, batch nudge) | — | — | — | — | ✓ | ✓ |

## Update checklist

When changing CLI behavior that affects agent usage:

1. ☐ Update `src/commands/init.rs` snippets (all 6: AGENTS, CLAUDE, COPILOT, CURSOR, KIRO, WINDSURF)
2. ☐ Update `skills/tkt-core/SKILL.md` (JTBD table + relevant section)
3. ☐ Update `skills/tkt-core/references/commands.md` (if flags/options changed)
4. ☐ Update `skills/tkt-work/SKILL.md` (if it affects ticket workflow)
5. ☐ Update `AGENTS.md` tkt CLI section (if flag surface changed)
6. ☐ Update `README.md` (if user-facing behavior changed)
7. ☐ Run `bash tools/deploy-skills.sh` (deploys skills to harness scan roots and refreshes the Antigravity `agy` plugin in `~/.gemini/config/plugins/tkt/`)
8. ☐ Bump version in `Cargo.toml` and `plugin.json` if init snippets changed (binary carries stale guidance otherwise)

## Why this matters

Init snippets are **baked into the binary**. Users who installed from crates.io or GitHub releases get whatever was compiled at release time. If snippets drift from actual CLI behavior, agents in those users' projects operate on stale instructions. A version bump + release is required when snippets change meaningfully.

The skills are deployed locally via `deploy-skills.sh` and take effect immediately — but only for this project's maintainer. Other tkt users don't get the skills; they get the init snippets. Skills (not steering) are the agent-guidance surface because steering is kiro-only, while skills are scanned cross-harness (kiro, codex, claude, agents) and packaged as a global plugin for Antigravity (`agy`).

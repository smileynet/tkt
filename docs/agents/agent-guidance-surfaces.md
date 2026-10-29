# Agent Guidance Surfaces

tkt teaches agents how to use it through multiple documents. When CLI behavior changes (new flags, status semantics, close requirements), **all surfaces must be updated together**.

## Surfaces (sync required)

| Surface | Location | Audience | Loaded when |
|---------|----------|----------|-------------|
| **Init snippets** | `src/commands/init.rs` (embedded constants) | External users' agents | `tkt init` / `tkt init --all` |
| **Skill (SKILL.md)** | `skills/tkt-core/SKILL.md` | Users’ agents (kiro, claude, codex, agy) | Skill activation via triggers |
| **Skill references** | `skills/tkt-core/references/*.md` | Users’ agents (lazy-loaded) | Agent reads reference link |
| **Workflow skills** | `skills/tkt-{orient,work,reconcile}/SKILL.md` | Users’ agents (cross-harness + agy plugin) | Skill activation via triggers |
| **Plugin manifests** | `.claude-plugin/plugin.json`, `.codex-plugin/plugin.json`, `plugin.json` | Claude Code, Codex, and Antigravity (`agy`) users | Plugin installation; Antigravity packaged by `tools/deploy-skills.sh` |
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
8. ☐ Follow [release maintenance](../releasing.md) when preparing delivery of changed init snippets. Update versions together through the coordinated release process; never edit versions ad hoc.

## Why this matters

Init snippets are **baked into the binary**. Users who installed from crates.io or GitHub releases get whatever was compiled at release time. If snippets drift from actual CLI behavior, agents in those users' projects operate on stale instructions. Changed snippets reach installed binaries through a release.

Users can install the workflow skills through the Claude Code, Codex, and Antigravity plugins, separately from the Rust CLI. Local maintainers refresh skills and plugins with `node tools/deploy.mjs` (or the bash wrapper), then start a new chat. Keep relevant skill and init-snippet guidance synchronized; the plugins distribute skills while the binary supplies init snippets.

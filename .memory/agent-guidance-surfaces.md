# Agent Guidance Surfaces

tkt teaches agents how to use it through multiple documents. When CLI behavior changes (new flags, status semantics, close requirements), **all surfaces must be updated together**.

## Surfaces (sync required)

| Surface | Location | Audience | Loaded when |
|---------|----------|----------|-------------|
| **Init snippets** | `src/commands/init.rs` (embedded constants) | External users' agents | `tkt init` / `tkt init --all` |
| **Skill (SKILL.md)** | `skills/tkt/SKILL.md` | tkt's own agents (kiro, claude, codex) | Skill activation via triggers |
| **Skill references** | `skills/tkt/references/*.md` | tkt's own agents (lazy-loaded) | Agent reads reference link |
| **Steering (frontier-work)** | `steering/frontier-work.md` | tkt's own agents (always-on) | Every session |
| **AGENTS.md (tkt CLI section)** | `AGENTS.md` | Contributors to tkt itself | Codex/agent reads project |
| **README (Usage section)** | `README.md` | Humans browsing crate/GitHub | Manual reading |

## What each surface covers

| Topic | Init snippets | SKILL.md | commands.md ref | frontier-work | AGENTS.md | README |
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
2. ☐ Update `skills/tkt/SKILL.md` (JTBD table + relevant section)
3. ☐ Update `skills/tkt/references/commands.md` (if flags/options changed)
4. ☐ Update `steering/frontier-work.md` (if it affects ticket workflow)
5. ☐ Update `AGENTS.md` tkt CLI section (if flag surface changed)
6. ☐ Update `README.md` (if user-facing behavior changed)
7. ☐ Run `bash tools/deploy-skills.sh` (deploys skill + steering to agent directories)
8. ☐ Bump version if init snippets changed (binary carries stale guidance otherwise)

## Why this matters

Init snippets are **baked into the binary**. Users who installed from crates.io or GitHub releases get whatever was compiled at release time. If snippets drift from actual CLI behavior, agents in those users' projects operate on stale instructions. A version bump + release is required when snippets change meaningfully.

The skill and steering files are deployed locally via `deploy-skills.sh` and take effect immediately — but only for this project's maintainer. Other tkt users don't get the skill; they get the init snippets.

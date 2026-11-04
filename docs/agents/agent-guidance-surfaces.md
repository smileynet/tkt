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
| **Contributor runbook and contract** | `AGENTS.md` → `docs/agents/tkt-contract.md`, architecture/plugin/platform references | Contributors to tkt itself | Runbook always; reference pointers trigger by task |
| **README (Usage section)** | `README.md` | Humans browsing crate/GitHub | Manual reading |

## What each surface covers

| Topic | Init snippets | SKILL.md | commands.md ref | tkt-work | Contributor runbook/reference | README |
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
| Global preview effect boundary and cached remote uncertainty | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Ticket creation quality | — | link | — | — | — | — |
| Health commands | — | ✓ | ✓ | — | ✓ | ✓ |
| Tag context | — | — | ✓ | ✓ | ✓ | ✓ |
| Migration | — | — | ✓ | — | ✓ | ✓ |
| Validation criteria | — | — | ✓ | — | ✓ | ✓ |
| Batch creation | — | ✓ | ✓ | — | ✓ | ✓ |
| Advisory hints (TKT_ADVICE, batch nudge) | — | — | — | — | ✓ | ✓ |
| Parent containers and explicit progress/closure | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Primary/worker coordination and explicit handoffs | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Invocation assignee versus eligibility | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Optional single-select label namespace policy | ✓ | ✓ | ✓ | — | ✓ | ✓ |

## Update checklist

When changing CLI behavior that affects agent usage:

1. ☐ Update `src/commands/init.rs` snippets (all 6: AGENTS, CLAUDE, COPILOT, CURSOR, KIRO, WINDSURF)
2. ☐ Update `skills/tkt-core/SKILL.md` (JTBD table + relevant section)
3. ☐ Update `skills/tkt-core/references/commands.md` (if flags/options changed)
4. ☐ Update `skills/tkt-work/SKILL.md` (if it affects ticket workflow)
5. ☐ Update the contributor runbook's conditional pointers and `docs/agents/tkt-contract.md` (if CLI compatibility or semantics changed); architecture/plugin/Windows references own their stated branches
6. ☐ Update `README.md` (if user-facing behavior changed)
7. ☐ Run `bash tools/deploy-skills.sh` (deploys skills to harness scan roots and refreshes the Antigravity `agy` plugin in `~/.gemini/config/plugins/tkt/`)
8. ☐ Follow [release maintenance](../releasing.md) when preparing delivery of changed init snippets. Update versions together through the coordinated release process; never edit versions ad hoc.
9. ☐ When moving contributor guidance, update [the exhaustive coverage map](guidance-coverage.md), run `node tools/check-guidance-coverage.mjs --final`, and review retained rules and read triggers against the frozen source. The map covers that migration baseline, not unreviewed future behavior.

## Why this matters

Init snippets are **baked into the binary**. Users who installed from crates.io or GitHub releases get whatever was compiled at release time. If snippets drift from actual CLI behavior, agents in those users' projects operate on stale instructions. Changed snippets reach installed binaries through a release.

Users can install the workflow skills through the Claude Code, Codex, and Antigravity plugins, separately from the Rust CLI. Local maintainers refresh skills and plugins with `node tools/deploy.mjs` (or the bash wrapper), then start a new chat. Keep relevant skill and init-snippet guidance synchronized; the plugins distribute skills while the binary supplies init snippets.

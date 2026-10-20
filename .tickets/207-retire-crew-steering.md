---
id: "207"
title: "Retire crew-research steering surface: fold frontier-work into a skill/AGENTS.md"
status: done
blocked_by: []
priority: medium
validation_criteria:
  - "frontier-work guidance lives in a grey-matter-convention skill or AGENTS.md, not steering/"
  - "tkt-reconcile and any skill referencing frontier-work point at the new home"
  - "deploy-skills.sh no longer deploys steering/ (or steering/ is removed)"
  - "no dangling references to steering/frontier-work.md across skills, AGENTS.md, README"
---

# Retire crew-research steering surface: fold frontier-work into a skill/AGENTS.md

## What to build

grey-matter is now the canonical skill format and crew-research is deprecated (session
decision, 2026-10-19). grey-matter has no "steering" concept — it folds that guidance
into skills + AGENTS.md. One crew-research artifact remains in this repo:
`steering/frontier-work.md` (the "work the frontier" rule, loaded as a user-rule). It
needs a grey-matter home.

Decide the destination and migrate:

- **Option A — a skill.** `frontier-work` is do-on-cue guidance with a clear trigger
  ("what should I work on?", "pick the next ticket"), which the promotion test
  (`grey-matter/docs/agents/doc-conventions.md`) says is a *skill*, not reference. This
  mirrors the crew `frontier-work` skill's own shape. Likely the right call; it would
  live in tkt's committed `skills/` set (e.g. `skills/frontier-work/`) and deploy via
  `tools/deploy-skills.sh` like `tkt` and `tkt-reconcile`.
- **Option B — AGENTS.md.** If it's better treated as a standing operational rule rather
  than a triggered procedure, fold the essentials into AGENTS.md. Riskier: AGENTS.md is
  paid every session (context rot), and the frontier rule is long.

Then:

- Repoint referrers. `skills/tkt-reconcile/SKILL.md` "Does NOT" section names
  `frontier-work`; any other skill that cites it must point at the new home.
- Stop deploying steering. `tools/deploy-skills.sh` currently copies `steering/*.md` to
  `~/.kiro/steering` (kiro-only). Remove that block once the guidance has moved, and
  delete `steering/` (or leave a stub README noting the migration).
- Clean up the global surface: the stale `~/.kiro/steering/frontier-work.md` copy on
  dev machines should be removed on next deploy (the installer doesn't manage steering
  with a manifest, so this may need a one-time manual sweep or an installer prune step).

## Context

- **Relevant files:** `steering/frontier-work.md` (the surface to retire),
  `tools/deploy-skills.sh` (the steering-copy block near the end),
  `skills/tkt-reconcile/SKILL.md` (referrer), `AGENTS.md`.
- **Conventions:** `grey-matter/docs/agents/doc-conventions.md` (promotion test:
  do-on-cue → skill), `grey-matter/docs/agents/skill-frontmatter.md` (safe-core
  frontmatter), `grey-matter/AGENTS.md` (no steering concept).
- **History:** #100 (tkt owns skill/steering deploy), #110 (steering-symlink decision —
  now superseded; the grey-matter rewrite copies instead), #67/#104 (Agent Plugins /
  agentskills conformance).
- **Related, not blocking:** #202 (deploy tkt guidance + skill safely across harnesses)
  is the broader deploy ticket; this one is the narrow steering-retirement slice. They
  can proceed independently.

## Acceptance criteria

- [x] `frontier-work` guidance lives in a grey-matter-convention skill (preferred) or
      AGENTS.md — not in `steering/`
- [x] `tkt-reconcile` and any other skill referencing `frontier-work` point at the new home
- [x] `tools/deploy-skills.sh` no longer deploys `steering/`; `steering/` is removed or stubbed
- [x] No dangling references to `steering/frontier-work.md` across skills, AGENTS.md, README
- [x] Stale `~/.kiro/steering/frontier-work.md` is cleaned up (sweep or installer prune)

## Out of scope

- The broader multi-harness guidance deploy (#202)
- Any change to the frontier *rule content* itself — this is a relocation, not a rewrite

## Resolution (2026-10-20)

Migrated steering/frontier-work.md to skills/tkt-work/ (cross-harness skill, name==dir). Repointed tkt-orient + tkt-reconcile referrers, AGENTS.md constraint, and guidance-surfaces/workspace-layout docs. Removed steering block + STEERING_SRC var from deploy-skills.sh, deleted steering/ dir, swept stale ~/.kiro/steering/frontier-work.md. Deployed tkt-work to all four harness roots.

### Verification
1. ✓ frontier-work guidance lives in a grey-matter-convention skill or AGENTS.md, not steering/ — "skills/tkt-work/SKILL.md created with frontmatter name:tkt-work + triggers; deploy dry-run lint passes name==dir"
2. ✓ tkt-reconcile and any skill referencing frontier-work point at the new home — "grep across skills/,AGENTS.md,README.md for frontier-work returns no matches; tkt-orient + tkt-reconcile now cite tkt-work"
3. ✓ deploy-skills.sh no longer deploys steering/ (or steering/ is removed) — "deploy-skills.sh steering block removed (bash -n ok); dry-run shows no steering copy; steering/ dir deleted"
4. ✓ no dangling references to steering/frontier-work.md across skills, AGENTS.md, README — "~/.kiro/skills/tkt-work/SKILL.md present in all 4 roots; ~/.kiro/steering/frontier-work.md confirmed removed"

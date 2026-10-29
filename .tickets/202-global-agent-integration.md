---
id: "202"
title: "Deploy tkt guidance and skill safely across supported harnesses"
status: backlog
blocked_by: []
priority: high
validation_criteria:
  - "a dry-run reports every supported harness target and performs no filesystem changes"
  - "installation preserves unmanaged user content and records ownership before replacing a managed integration"
  - "an OpenCode-only environment receives both the project instruction path and the global tkt skill"
tags: ["guidance", "agent-integration"]
---

# Deploy tkt guidance and skill safely across supported harnesses

## What to build

Provide a single safe installation path for the two complementary integration layers:

1. project-local, marker-owned tkt instructions (`AGENTS.md` or the harness-native
   equivalent); and
2. a globally discoverable `tkt` skill for harnesses that support skills.

`tkt init --all` already writes project instructions, while
`tools/deploy-skills.sh` installs a maintainer-local skill only for Kiro, Claude, and
Codex. That split leaves an OpenCode-only environment without a global skill despite
OpenCode being advertised, skips harnesses whose root does not already exist, and
unconditionally removes a same-named skill without recording ownership.

Define a supported-harness registry with verified project-instruction and global-skill
paths. The installer must offer `--tool`, `--all`, and `--dry-run`; create missing
managed roots; and replace only tkt-owned paths (or an existing tkt symlink). Record
managed ownership and source/version so an unmanaged same-named directory is reported
as a conflict, never deleted. Keep per-project instructions idempotent and
marker-owned; do not invent a global AGENTS.md convention where a harness does not
have one.

The independent Kiro review correctly identifies that a `cargo install` user cannot
use repository-only skill assets. Package the canonical skill/instruction templates
with the executable (for example via compile-time embedded assets) and provide a
dedicated deployment command rather than overloading project-local `tkt init` with
global behavior. Exact command spelling and harness paths remain design work and must
be verified against each harness's current official documentation. Codex's user-level
instruction discovery is a useful supported case; it is not permission to generalize
that convention to other harnesses.

Source: 2026-10-19 review of `tkt init`, `tools/deploy-skills.sh`, README agent
claims, and the existing `skills/tkt` package.

## Context

- Read `src/commands/init.rs` for current project-local targets and markers.
- Read `tools/deploy-skills.sh` for current local symlink behavior.
- Read `docs/agents/agent-guidance-surfaces.md`; a CLI behavior change still requires all
  listed guidance surfaces to remain synchronized.
- Verify each harness path against its current official documentation before adding it
  to the registry. A claimed but unverified target is worse than an explicit skip.

## Acceptance criteria

- [ ] `--dry-run` reports the instruction and skill action for every verified target
      and makes no filesystem changes
- [ ] Installation creates missing managed roots and updates only tkt-owned paths;
      an unmanaged target is left untouched with an actionable conflict message
- [ ] A verified OpenCode-only fixture receives both its project instruction and its
      global tkt skill without requiring a Claude installation
- [ ] Temporary-home tests cover first install, idempotent update, unmanaged conflict,
      and dry run; all guidance surfaces describe the same supported harness set
- [ ] A binary installed without the repository can deploy the packaged tkt skill and
      only the verified harness-specific instruction target for each selected tool

## Out of scope

- Teaching every harness to consume skills it does not natively support
- Changing the ticket workflow itself

## Rescope (2026-10-29)

Most of this shipped in 0.5.0 through the shared deploy engine (`tools/deploy.mjs`,
vendored from grey-matter; `tools/deploy-skills.sh` now forwards to it):

- `--dry-run` plans every action without writing.
- An ownership manifest records what the deployer installed; unowned same-named
  directories are refused, not deleted.
- Missing managed roots are created; OpenCode is covered (grey-matter's deploy tests
  include an OpenCode-only home, served through `~/.claude/skills` or its own root).
- Claude, Codex and Antigravity users can install the plugins without a checkout.

Remaining gap: a user who installs only the binary (`cargo install tkt`) and uses
OpenCode or Kiro still needs a repository checkout to get the four workflow skills.
Narrow the work to that: package the skills with the binary and give it a deploy
command. Check off the delivered criteria above with evidence when this ticket is
worked.

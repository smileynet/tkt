---
id: "222"
title: "Adopt grey-matter repo layout: relocate .memory/ to GLOSSARY.md, docs/adr/, docs/agents/"
status: in_progress
blocked_by: []
validation_criteria:
  - "Relocation leaves 0 new broken links and cargo build, clippy and tests match the pre-change baseline"
  - "No tracked .memory/ remains and AGENTS.md points at the new homes"
tags: ["stream:tooling"]
---

# Adopt grey-matter repo layout: relocate .memory/ to GLOSSARY.md, docs/adr/, docs/agents/

Created locally; a corporate push-policy hook blocks pushes to this remote from this host, so check for
an id collision (`tkt renumber`) before pushing from elsewhere.

## Observed (c242a04)

- Tracked `.memory/`: `CONTEXT.md`, `adr/0001..0003`, `specs/workspace-layout.md`,
  `agent-guidance-surfaces.md`, `proposed-skill-release-management.md`. No `GLOSSARY.md`,
  `docs/adr/` or `CLAUDE.md`.
- Live references: `AGENTS.md:11` and `:230`, a doc comment in `src/commands/init.rs:5`.
  `tools/release-tools.test.mjs:84` mentions `.memory/` only as fixture data.
- Packaging is unaffected: `Cargo.toml` uses an allowlist `include` (src, build.rs,
  Cargo.*, README, CHANGELOG, LICENSE), so `docs/` never enters the crate.
- tkt's code does not read `.memory/` anywhere.
- `.tickets/config.toml` has `audit.resolution_baseline_id`, unknown to the 0.4.0 release
  ("unknown config key" warning on every command); main is 145 commits past v0.4.0.

## What to build

Run grey-matter `adopt-project` (subsume): `relocate-knowledge.mjs` with the default map
plus the two loose notes into `docs/agents/`, update AGENTS.md and the `init.rs` comment, add
a `CLAUDE.md` pointer, verify against a `cargo build`/clippy/test + link-check baseline.

## Acceptance criteria

- [ ] Relocation leaves 0 new broken links; cargo build, clippy and tests match the baseline
- [ ] No tracked `.memory/` remains and AGENTS.md points at the new homes

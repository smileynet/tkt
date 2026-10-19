---
id: "201"
title: "Remediate small agent-facing help and release metadata drift"
status: open
blocked_by: []
validation_criteria:
  - "tkt --help has one accurate Close description and AGENTS.md documents every relevant public audit/debug option"
  - "plugin.json version and changelog header match the release metadata policy"
  - "the release process produces curated user-facing notes without internal ticket or documentation noise"
tags: ["guidance", "release"]
---

# Remediate small agent-facing help and release metadata drift

## What to build

Repair the small, independently safe consumer-surface drifts found in the
2026-10-19 release-readiness review. Keep this ticket limited to help text,
maintainer guidance, release metadata, and release-note generation; it does not own
the larger capability-manifest work (#138) or cross-harness installation work (#202).

- Remove the duplicate `Close` description in `tkt --help`.
- Align AGENTS.md with the actual public audit and debug surface, including
  `tkt audit --deep` and all supported `TKT_DEBUG` values.
- Make plugin metadata use the release version, and align the changelog header with
  the project's Keep a Changelog 1.1 guidance.
- Preserve a curated, user-facing CHANGELOG.md at release time. git-cliff may produce
  a review candidate, but it must not overwrite the curated file or publish internal
  ticket, documentation, CI, test, or refactoring entries.

Source: release-readiness review requested 2026-10-19; see #146 for release scope and
the crew-research changelog-discipline guidance.

The independent Kiro review confirms the two help defects and supplies a candidate
summary of post-v0.3.0 user-visible changes. Incorporate that research only after
checking each assertion against the repository: populate `Unreleased` first, do not
pre-label it as v0.4.0 or claim that OpenCode receives a global skill before #202
delivers one. Describe atomic-write behavior with the #181 qualification until its
cross-platform investigation is closed.

## Context

- Read `src/cli.rs` for the duplicate command description and `AGENTS.md` for the
  contributor-facing command table.
- Read `src/telemetry.rs` for the canonical debug-mode values.
- Read `plugin.json`, `CHANGELOG.md`, `cliff.toml`, and `release.toml` together:
  cargo-release currently invokes git-cliff directly on CHANGELOG.md, while the
  catch-all cliff parser can surface internal ticket commits.
- Preserve the current command and release compatibility contracts; #138 owns
  machine-readable capability completeness.

## Acceptance criteria

- [x] `tkt --help` prints the Close command description once, and AGENTS.md documents
      `tkt audit --deep` plus every supported `TKT_DEBUG` spelling and destination
- [x] `plugin.json` is versioned with the released package and CHANGELOG.md declares
      the agreed Keep a Changelog version
- [x] A release-note dry run proves that curated user-facing entries survive and that
      ticket/docs/CI/test/refactor commit noise is excluded
- [x] The populated `Unreleased` section contains only verified user-facing changes;
      it makes no premature release-version, OpenCode-skill, or unqualified
      cross-platform atomic-write claim

## Out of scope

- Completing the capabilities manifest (#138)
- Adding or changing agent-harness installation targets (#202)

## Resolution (2026-10-19)

Corrected agent-facing help/guidance and made the release process preserve curated notes.

### Verification
1. ✓ tkt --help has one accurate Close description and AGENTS.md documents every relevant public audit/debug option — "Installed tkt 0.4.0 (78c3f24) shows one close summary; AGENTS.md documents audit --deep and every TKT_DEBUG spelling/destination."
2. ✓ plugin.json version and changelog header match the release metadata policy — "plugin.json and Cargo metadata both declare 0.4.0; CHANGELOG.md and git-cliff use Keep a Changelog 1.1."
3. ✓ the release process produces curated user-facing notes without internal ticket or documentation noise — "git cliff --unreleased was run as a review candidate and excludes configured ticket/docs/CI/test/refactor noise; release.toml no longer lets it overwrite CHANGELOG.md, and Unreleased remains curated and unlabelled until the release cut."

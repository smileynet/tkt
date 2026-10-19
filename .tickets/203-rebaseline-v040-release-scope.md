---
id: "203"
title: "Decide v0.4.0 release scope before moving planned work to v0.5.0"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "docs/plan.md states the approved v0.4.0 and v0.5.0 scopes"
  - "release ticket blockers match the approved v0.4.0 gate"
tags: ["release", "planning"]
---

# Decide v0.4.0 release scope before moving planned work to v0.5.0

## What to build

Make the release-train decision explicit before changing milestone headings, package
versions, or changelog release links. The preferred decision is to skip an untagged
v0.3.1 and make v0.4.0 the release of the already-merged post-v0.3.0 user-facing
changes, subject to the existing correctness/release gate. Move the unimplemented
agent-experience extensions and native-relations/label-namespace work now grouped
under v0.4.0 to v0.5.0.

This prevents a release of existing behavior from being delayed by net-new feature
work while preserving the non-negotiable safety gate. v0.4.0 would retain #146's
integrity, validation-baseline, CI, packaging, and release-verification blockers and
add #201 for accurate user-facing release metadata. It would not make #202's new
cross-harness global installer a v0.4.0 blocker. The planned #127, #138, #161, #168,
#79, #83, #86, #87, #88, #89, #90, #93, #192–#200, and #202 work belongs to v0.5.0
unless a separate approved scope decision says otherwise.

Source: user request on 2026-10-19 following the release-readiness and independent
Kiro reviews. The Kiro proposal recommended v0.4.0 but understated the established
#146 gate; this ticket preserves that gate rather than silently deferring it.

## Context

- Read `docs/plan.md` for the two current v0.4.0 sections and its v0.3.1 corrective
  release table.
- Read #146, #150, #160, #181, #185, #188, #189, and #201 before editing the release
  gate. Open tickets cannot be removed from it without an explicit deferral decision.
- Read `CHANGELOG.md`, `release.toml`, and `Cargo.toml` only after the plan decision;
  version changes must follow, not lead, the approved scope.

## Out of scope

- Implementing any deferred v0.5.0 feature
- Bumping package or plugin versions, tagging, or publishing a release

## Acceptance criteria

- [ ] The approved plan distinguishes the v0.4.0 shipped-change release train from
      deferred v0.5.0 feature work, with every moved ticket represented exactly once
- [ ] #146 names the approved release version and lists every required v0.4.0 blocker,
      including #201 if the release requires curated notes and metadata
- [ ] The decision records whether v0.3.1 is skipped and which currently open tickets
      are explicitly deferred rather than implying that an open-ticket release is safe

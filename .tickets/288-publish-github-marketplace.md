---
id: "288"
title: "publish a GitHub-source marketplace so both dev machines converge via plugins update"
status: backlog
blocked_by: []
priority: low
validation_criteria:
  - "a published marketplace source (GitHub, or archive pinned by sha256 to the release bundle) is the canonical install; local-directory adds remain a dev-machine affordance"
  - "macOS (0.1.0 spike snapshot) and Windows (0.2.0) installs converge through plugins update from the published source"
tags: ["kind:enhancement", "stream:tooling"]
---
# Publish a canonical GitHub-source marketplace

## Intent source

Grey-matter proposal 23 (T4), paired with the external grey-matter project's ticket 292 (`packed-marketplace-archive-source`). The 2026-11-15 board reconciliation clarified that this is not a local tkt ticket reference.

## Context

- The macOS installation is at the 0.1.0 spike snapshot and the Windows installation at 0.2.0.
- Local-directory plugin sources are development conveniences, not a convergence mechanism.
- Anthropic's official marketplace uses published sources rather than development checkouts; release archives pinned by SHA-256 are the lightweight alternative.

## What to build

Publish a canonical GitHub-source marketplace, or an archive source pinned by SHA-256 to the reviewed release bundle, so both development machines converge through the normal plugin update path. Retain local-directory sources only for development.

## Acceptance criteria

- [ ] A published marketplace or pinned release-archive source is documented as the canonical installation path.
- [ ] The macOS 0.1.0 snapshot and Windows 0.2.0 installation converge through plugin update from that source.
- [ ] Local-directory adds remain documented as development-only.
- [ ] Evidence names the published source and both observed client identities.

## Out of scope

Manifest single-sourcing (ticket 287) and any release/publication step without the separately required owner authorization.

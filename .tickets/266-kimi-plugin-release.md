---
id: "266"
title: "Integrate the Kimi plugin with tkt maintenance and releases"
status: done
blocked_by: ["265"]
validation_criteria:
  - "Cargo-owned version preparation and manifest checks include the Kimi manifest without drift."
  - "Release artifacts build and verify a Kimi host bundle with the same identity and four skills without publishing anything."
  - "README, plugin maintenance, release guidance, and applicable tooling gates document and check installation, prerequisites, updates, and validation."
tags: ["tooling"]
---
## What to build

Add the Kimi manifest to Cargo-owned version surfaces and plugin manifest tests. Extend release artifact construction and verification with a Kimi host bundle. Add package checks to source-selected tooling gates and document build, registration, owner installation, CLI and Git prerequisites, updates, and native validation. No publication is authorized.

Intent: make Kimi a maintained tkt distribution rather than an untracked local conversion, following the package contract established in ticket 265.

## Acceptance criteria

- [x] Release version edits and Rust manifest tests include the Kimi source manifest.
- [x] Release fixtures build and verify a same-version Kimi archive with all four skills and no Claude hooks.
- [x] User and maintainer documentation covers installation, prerequisites, updates, validation, and publication boundaries.
- [x] Applicable tooling and Rust checks pass; native or platform gaps are reported.

## Resolution (2026-11-08)

Integrated the Kimi manifest with Cargo-owned versions, built and verified a fourth self-contained host archive, added always-run package tests, and documented installation, prerequisites, updates, validation, and publication boundaries. Nothing was published.

### Verification
1. ✓ Cargo-owned version preparation and manifest checks include the Kimi manifest without drift. — "Cargo test --test plugin_manifests passed 5/5, including .kimi-plugin/kimi.plugin.json; canonical release-version tests cover candidate updates and drift."
2. ✓ Release artifacts build and verify a Kimi host bundle with the same identity and four skills without publishing anything. — "tools/release-artifacts.mjs build and verify passed from commit 799c75057068f3f710c4d36f662a34b884ba8fc9: 4 assets, 4 skills, version 0.4.0; native verification not run and no publication performed."
3. ✓ README, plugin maintenance, release guidance, and applicable tooling gates document and check installation, prerequisites, updates, and validation. — "README, plugin-maintenance.md, and releasing.md document the complete lifecycle. Full mise run check passed 48/48 tooling tests, then stopped at the pre-existing src/commands/done.rs:202 Clippy byte-string error before the full Rust suite; focused manifest tests passed separately."

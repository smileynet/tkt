---
id: "266"
title: "Integrate the Kimi plugin with tkt maintenance and releases"
status: in_progress
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

- [ ] Release version edits and Rust manifest tests include the Kimi source manifest.
- [ ] Release fixtures build and verify a same-version Kimi archive with all four skills and no Claude hooks.
- [ ] User and maintainer documentation covers installation, prerequisites, updates, validation, and publication boundaries.
- [ ] Applicable tooling and Rust checks pass; native or platform gaps are reported.
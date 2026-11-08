---
id: "270"
title: "Validate tkt Kimi installation beyond macOS"
status: backlog
blocked_by: []
validation_criteria:
  - "test -s docs/evidence/kimi-plugin-platform-validation.md"
  - "node --test tools/kimi-plugin.test.mjs"
tags: ["tooling", "gate:evidence"]
---
# Intent source

Sam requested follow-up tickets for remaining Kimi plugin limitations on 2026-11-08. Closed ticket 267 verified live installation only on macOS.

# What to learn and deliver

Install and exercise the maintained tkt Kimi package on at least one real non-macOS client, preferably Windows. Verify native validation, registration, owner installation, all four skills, linked command references, and the separately installed tkt/Git prerequisites. Exercise a read-only ticket query against a real repository.

Record the client OS and package identity in `docs/evidence/kimi-plugin-platform-validation.md`. Name any untested operating systems explicitly instead of claiming broad platform support.

# Out of scope

Fixing unreproduced platform defects, release publication, and replacing live installation with static archive inspection.

## Acceptance criteria

- [ ] A real non-macOS installation loads `tkt-core` and its linked command reference from packaged bytes.
- [ ] A read-only ticket query returns current repository data using the separately installed prerequisites.
- [ ] Evidence names every tested and untested platform.
- [ ] Existing Kimi package tests pass in the applicable maintained environment.

## Note — 2026-11-08

By: anonymous

<!-- tkt-note:v1 99564-1794172689034443000 -->
2026-11-08 audit: no real non-macOS Kimi client is available in this environment. Keep backlog with gate:evidence; static package inspection is not platform validation.

## Note — 2026-11-08

By: anonymous

<!-- tkt-note:v1 9747-1794173499010986000 -->
## Planning gate\n\nA real non-macOS Kimi client is required; no platform result may be inferred from macOS or static package checks.

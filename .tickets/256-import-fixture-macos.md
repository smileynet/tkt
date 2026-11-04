---
id: "256"
title: "Make import and note fixtures deterministic on macOS"
status: in_progress
blocked_by: []
validation_criteria: 
  - "Import and note failure fixtures pass on this macOS host without host configuration edits or skipped assertions; Windows coverage is reported separately."
tags: ["stream:tooling"]
assignee: "codex"
---
## What to build

During grey-matter sync verification, mise run check failed in tests/github_import.rs: automatic Git maintenance created transient lock files, and an empty executable was accepted by macOS as a successful shell script. Keep the fixture isolated and use an explicitly unlaunchable executable. Source: tests/github_import.rs.

Note-fixture follow-up: tests/note.rs had the same background-maintenance race and
created a non-executable pre-commit hook on POSIX. Disable maintenance in its disposable
repository and use the existing executable-hook helper. Missing-gh coverage also
isolates PATH so a broken fixture cannot fall through to the owner's real CLI.
Windows execution remains unverified on this Mac. No product code or host Git settings change.

## Acceptance criteria

- [x] Import fixtures pass on macOS without host Git config changes or skipped assertions.

## Resolution (2026-11-04)

done: isolated import and note fixture Git maintenance, executable hook creation and missing-gh PATH. No product code or host configuration changes. macOS full gate passed using a disposable Git template; Windows native execution remains unverified. Small verification logs are retained in the coordinating sync session until compact evidence is promoted.

### Verification
1. ✓ Import and note failure fixtures pass on this macOS host without host configuration edits or skipped assertions; Windows coverage is reported separately. — "Full mise run check with disposable GIT_TEMPLATE_DIR passed: 43 Node tests, cargo fmt --check, warning-free clippy and 467 Rust tests across 41 suites; zero failures or ignored Rust tests. Focused import19/19 and note7/7 passed without the template. Windows was not run on this Mac."

## Windows follow-up — 2026-11-03

Reopened after integrating the upstream fixture repair: the missing-gh test failed
before reaching gh because moving Git for Windows through a symlink broke runtime
lookup (rev-parse failed with empty stderr). Keep the real Git executable directory
on the isolated Windows PATH, assert it contains no gh.exe, and keep the invalid
fixture gh first. Unix retains its symlink-only PATH. The focused Windows check
passed 1/1 and the fresh full gate passed 482 Rust tests, 42 Node tests, one
existing Windows relative-link skip, clean formatting and warning-free clippy.
Independent critic accepted this test-only change conditional on that gate.
Historical macOS evidence above remains unchanged; macOS was not rerun here.

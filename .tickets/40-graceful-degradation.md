---
id: "40"
title: "graceful degradation: read commands skip unparseable files instead of crashing"
status: done
blocked_by: ["38"]
---

# Graceful degradation: read commands skip unparseable files instead of crashing

## Problem

`load_corpus()` currently fails on the first unparseable ticket file, causing all commands that use it to crash (exit 2). This means a single malformed file makes the entire tool unusable in that project.

`tkt validate` already handles this — it collects parse errors as findings and continues. But `ready`, `query`, `close`, `claim`, `edit` all use `load_corpus()` which propagates the first error.

## Proposed fix

Add a `load_corpus_tolerant()` variant (or add a mode flag) that:
1. Skips unparseable files
2. Emits a warning to stderr: `⚠ skipping 03-broken.md: missing required field: id`
3. Returns the successfully-parsed tickets
4. Returns the list of parse errors (for commands that want them)

Use tolerant loading for read commands (`ready`, `query`) and mutation preflight (`claim`, `close`, `edit`). Keep strict loading for `validate` (where finding parse errors is the point).

## Design notes

- Mutation commands need the corpus to find the target ticket — if the *target* ticket is the one that can't parse, that's a domain error (exit 1), not a crash
- Read commands should always show what they can, even if some files are broken
- The warning must go to stderr (not stdout) to avoid polluting JSON output

## Acceptance criteria

- [x] `tkt ready` succeeds even if one .tickets/ file is malformed
- [x] Warning printed to stderr for each skipped file
- [x] `tkt query` shows all parseable tickets, skips broken ones with warning
- [x] Mutation commands error clearly if their target ticket is unparseable
- [x] Integration test: corpus with one good + one bad file → ready shows the good one

## Resolution (2026-08-09)

Fixed in commit 88d0faa: lenient priority parsing + graceful corpus loading. Crashes were caused by Priority::parse rejecting unknown values and load_corpus crashing on unparseable files. Both now degrade gracefully with stderr warnings.

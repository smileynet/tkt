---
id: "264"
title: "tkt new -o json returns id and path; edit --ac help explains its argument"
status: done
blocked_by: []
priority: medium
validation_criteria:
  - "tkt new -o json includes id and path fields; tkt edit --help describes --ac"
tags: ["kind:enhancement", "cli"]
assignee: "kimi"
---
## What to build

Structured success output for `tkt new -o json` (add `id` and `path`), and a help string for `tkt edit --ac`.

## Intent and context

Source: android-helper session retrospective, 2026-11-07. Filed from android-helper; no work done here. Extends done tickets #105 and #143 (JSON success output).

Observed with tkt 0.4.0 (1fbc40c):
- `tkt new -o json` returns `{"ok":true,"result":"created 01 demo (local only)"}`, with no `id` or `path`. A script parsing that text failed in android-helper, created a duplicate ticket and needed `tkt undo` (two extra commits).
- `tkt edit --help` lists `--ac <AC>` with no description. Passing text failed with "invalid digit found in string", so it takes checkbox indices (inferred from that error).

## Acceptance criteria

- [x] `tkt new -o json` includes `id` and `path` alongside the existing `result` (backward compatible)
- [x] `tkt batch -o json` reports each created id and path
- [x] `tkt edit --help` says what --ac accepts

## Resolution (2026-11-08)

Added stable creation identities to JSON output, structured batch collections, final retry paths, edit --ac help, regression tests, and synchronized user/agent guidance. Removed the disposable capability snapshot; no generated artifacts are retained for this ticket.

### Verification
1. ✓ tkt new -o json includes id and path fields; tkt edit --help describes --ac — "Commit 9a06540: creation_json passed 4/4 for normal, quiet, dry-run, help, and new/batch retry behavior; integration passed 104/104 and quiet_json 3/3; mise run check passed formatting, Clippy, 49 selected tooling tests, and the complete Rust suite with zero failures."

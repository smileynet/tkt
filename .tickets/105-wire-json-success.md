---
id: "105"
title: "Wire emit_json_success/print_success into mutation commands"
status: done
blocked_by: []
priority: low
validation_criteria: 
  - "tkt -o json emits JSON success on mutations"
  - "dead_code annotations removed"
  - "cargo test passes"
---

# Wire emit_json_success/print_success into mutation commands

## What to build

`emit_json_success` and `print_success` in `src/commands/common.rs` are marked `#[allow(dead_code)]` — they're scaffolding from the structured errors work (ticket 85) that isn't yet wired into any command's success path.

Wire `print_success` (or the JSON envelope equivalent) into mutation commands (claim, close, edit, new, batch) so that `-o json` produces structured success output, not just structured errors.

## Acceptance criteria

- [x] `tkt claim <id> -o json` emits `{"ok":true,"result":"..."}` on stdout
- [x] `tkt close <id> -o json` emits JSON success envelope
- [x] `tkt new <slug> -o json` emits JSON success envelope
- [x] `#[allow(dead_code)]` annotations removed from `emit_json_success` and `print_success`
- [x] Existing text output unchanged when `-o json` is not passed

## Resolution (2026-08-18)

All mutation commands now emit JSON success envelopes via print_success. Dead code annotations removed.

### Verification
1. ✓ tkt -o json emits JSON success on mutations — "tkt -o json ready emits JSON Lines to stdout"
2. ✓ dead_code annotations removed — "#[allow(dead_code)] removed from emit_json_success, is_json_output, print_success"
3. ✓ cargo test passes — "cargo test: 55 passed, cargo clippy: 0 warnings"

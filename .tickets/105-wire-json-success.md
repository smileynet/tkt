---
id: "105"
title: "Wire emit_json_success/print_success into mutation commands"
status: in_progress
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

- [ ] `tkt claim <id> -o json` emits `{"ok":true,"result":"..."}` on stdout
- [ ] `tkt close <id> -o json` emits JSON success envelope
- [ ] `tkt new <slug> -o json` emits JSON success envelope
- [ ] `#[allow(dead_code)]` annotations removed from `emit_json_success` and `print_success`
- [ ] Existing text output unchanged when `-o json` is not passed

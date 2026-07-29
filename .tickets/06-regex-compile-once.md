---
id: "06"
title: "compile regex patterns once instead of per-call"
status: done
blocked_by: ["03"]
---

# Compile regex patterns once instead of per-call

## What to build

Fixed regex patterns are compiled on every call to `parse_str`, `blocked_by`, `numeric_key`, `max_id`, `id_width`, and several CLI functions. Use `std::sync::LazyLock<Regex>` statics to compile once.

### Patterns to consolidate

| Current location | Pattern | Static name |
|-----------------|---------|-------------|
| `parse_str` | `^([A-Za-z_][A-Za-z0-9_-]*):(.*)$` | `RE_FM_KEY` |
| `blocked_by` | `\[(.*)\]` | `RE_BRACKET_LIST` |
| `numeric_key` | `^(\d+)(.*)$` | `RE_NUMERIC_PREFIX` |
| `max_id` / `id_width` | `^(\d+)-` | `RE_FILENAME_ID` |
| `cmd_new`/`cmd_batch` | `^[a-z0-9][a-z0-9-]*$` | `RE_SLUG` (move to validate.rs after ticket 03) |
| `cmd_close` / `flip_ac_boxes` | `^(\s*)- \[ \]` | `RE_UNCHECKED_AC` |
| `cmd_validate` | various | consolidate with validate module |

### Approach

1. Use `std::sync::LazyLock` (stable since Rust 1.80, no new dep)
2. Place statics near their semantic owner (core statics in ticket.rs, validation in validate.rs, CLI-specific in cli.rs)
3. Do NOT make dynamically-constructed patterns (like sync-plan row regex per-ticket) into statics — those stay as-is
4. Run `cargo test` and verify identical behavior

### Sequencing note

This ticket is blocked by #03 because the validation module will introduce new patterns and reorganize existing ones. Doing regex consolidation first would create rework.

## Acceptance criteria

- [ ] All fixed regex patterns compiled via LazyLock statics
- [ ] No `Regex::new(...)` calls inside frequently-called functions
- [ ] Dynamic patterns (per-ticket sync-plan regex) left as-is with comment
- [ ] `cargo test` passes with identical results
- [ ] `cargo clippy` clean
- [ ] No new dependencies added

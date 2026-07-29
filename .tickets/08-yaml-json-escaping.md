---
id: "08"
title: "safe YAML generation and structured JSON output"
status: open
blocked_by: []
priority: high
---

# Safe YAML generation and structured JSON output

## What to build

Two related trust-boundary issues:

### YAML generation

`new_ticket_text` and edit paths interpolate user-provided values (title, spec, note) into double-quoted YAML via `format!("\"{}\"", value)`. Characters like `"`, `\`, newlines, and control chars produce invalid or injected frontmatter.

**Fix:** Create a `yaml_escape(value: &str) -> String` helper that properly escapes YAML double-quoted scalars (escape `\`, `"`, newlines, tabs, control chars per YAML spec). Use it in every path that writes user text into frontmatter.

### JSON output

`ready --json`, `validate`, and `sync-plan` build JSON with `format!` and incomplete escaping (only `"` is escaped in some places). Backslashes, newlines, control chars, and Unicode in ticket data produce invalid JSON.

**Fix:** Create a `json_escape(value: &str) -> String` helper (or add `serde_json` — but prefer no new dep if possible). Use it consistently in all JSON output paths. Long-term, typed response structs with a serializer are better, but the escape helper unblocks safety now.

### Changes needed

1. Add `fn yaml_scalar_escape(s: &str) -> String` in `core/ticket.rs` or a new `core/format.rs`
2. Add `fn json_string_escape(s: &str) -> String` alongside it
3. Replace all `format!("\"{}\"", ...)` YAML interpolation with the escape helper
4. Replace all manual JSON string assembly with the escape helper
5. Add unit tests with adversarial inputs: quotes, backslashes, newlines, null bytes, Unicode

## Acceptance criteria

- [ ] Title containing `"` produces valid frontmatter file
- [ ] Title containing `\n` produces valid frontmatter (escaped, not literal newline)
- [ ] Title containing `\` produces valid frontmatter
- [ ] JSON output with adversarial title parses as valid JSON
- [ ] JSON output with adversarial file paths parses as valid JSON
- [ ] No new crate dependencies added
- [ ] All existing tests pass
- [ ] New unit tests cover: empty string, quotes, backslashes, newlines, tabs, null byte, Unicode, combined

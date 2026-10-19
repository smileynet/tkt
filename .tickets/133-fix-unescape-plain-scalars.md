---
id: "133"
title: "Fix yaml_scalar_unescape corrupting plain (unquoted) scalars"
status: done
blocked_by: []
priority: high
validation_criteria:
  - "unquoted title with backslash reads back correctly"
  - "unescape only applied to double-quoted values"
---

# Fix yaml_scalar_unescape corrupting plain (unquoted) scalars

> Source: #128 **F3** (P0, 2026-08-23 architecture audit). #128 is done; evidence + fix sketch below.

## What to build

Reading a hand-written *unquoted* frontmatter scalar must return its bytes unchanged.
Today `yaml_scalar_unescape` runs unconditionally on read, so an unquoted `title: C:\notes`
reads back as `C:` + LF + `otes`, and `spec: regex \d+` loses the backslash. Escape
decoding must apply **only** when the raw value was actually double-quoted (the encode-on-write
path already quotes such values). Stored bytes are unaffected today — the corruption is on the
read/display/query/ready-json path — but it produces silently wrong output.

## Context

- **Location (#128 F3):** `src/core/ticket.rs:448-452, 471-473` (drift possible — confirm).
- **Contract:** README promises "edit them by hand anytime" — unquoted scalars are legal input.
- **Fix sketch (#128):** unescape only when the raw value was double-quoted; leave plain scalars verbatim.

## Acceptance criteria

- [x] An unquoted `title: C:\notes` round-trips unchanged through read (typed title == `C:\notes`)
- [x] An unquoted `spec: regex \d+` retains its backslash on read
- [x] Double-quoted values still have escapes decoded (existing behavior preserved)
- [x] Regression test with a hand-edited backslash-scalar fixture (per #128 defect-class requirement)
- [x] cargo fmt && cargo clippy --all-targets && cargo test pass clean

## Resolution (2026-10-19)

Plain and single-quoted scalar text now remains literal; only complete double-quoted scalars decode YAML escapes. Added unit and command-level regressions for hand-edited Windows paths and regex values.

### Verification
1. ✓ unquoted title with backslash reads back correctly — "cargo test test_query_preserves_hand_edited_plain_scalar_backslashes: passed; query preserved C:\notes and regex \d+ from an unquoted ticket"
2. ✓ unescape only applied to double-quoted values — "cargo test scalars: passed; quoted newline and escaped backslash decoding remains covered"

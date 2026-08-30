---
id: "133"
title: "Fix yaml_scalar_unescape corrupting plain (unquoted) scalars"
status: open
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

- [ ] An unquoted `title: C:\notes` round-trips unchanged through read (typed title == `C:\notes`)
- [ ] An unquoted `spec: regex \d+` retains its backslash on read
- [ ] Double-quoted values still have escapes decoded (existing behavior preserved)
- [ ] Regression test with a hand-edited backslash-scalar fixture (per #128 defect-class requirement)
- [ ] cargo fmt && cargo clippy --all-targets && cargo test pass clean

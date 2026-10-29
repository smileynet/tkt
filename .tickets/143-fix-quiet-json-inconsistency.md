---
id: "143"
title: "Fix -q + -o json emits bare non-JSON output inconsistently"
status: backlog
blocked_by: []
priority: low
validation_criteria:
  - "all mutations produce valid JSON under -o json -q"
---

# Fix -q + -o json emits bare non-JSON output inconsistently

## What to build

Observed on 0.5.0 (`a827bb6`, 2026-10-29): `tkt new d --title Delta -q -o json` printed a
bare `04`, not JSON. Scripts that ask for `-o json` must always get JSON; `-q` should
shrink the object (for example `{"id":"04"}`), not change the format.

## Acceptance criteria

- [ ] Every mutating command run with `-o json -q` prints exactly one valid JSON value
- [ ] A test iterates the mutating commands under `-o json -q`
- [ ] Help documents how `-q` and `-o json` combine

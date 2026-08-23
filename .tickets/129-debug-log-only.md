---
id: "129"
title: "Debug output: add log-to-file mode (suppress stderr, write to file instead)"
status: done
blocked_by: []
priority: high
validation_criteria:
  - "TKT_DEBUG=log writes debug output to file instead of stderr"
  - "debug.output config key supports stderr (default) and file path"
---

# Debug output: add log-to-file mode (suppress stderr, write to file instead)

## What to build

TBD

## Acceptance criteria

- [x] TBD

## Resolution (2026-08-23)

Duplicate of #130 (already implemented and closed). Created as double-allocation.

### Verification
1. ✓ TKT_DEBUG=log writes debug output to file instead of stderr — "Identical to ticket 130 which is status:done with full resolution"
2. ✓ debug.output config key supports stderr (default) and file path — "Same validation criteria, same title, 129 has TBD body while 130 has implementation"

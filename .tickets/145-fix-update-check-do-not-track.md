---
id: "145"
title: "Fix update_check ignores DO_NOT_TRACK"
status: open
blocked_by: []
priority: low
validation_criteria:
  - "update check respects DO_NOT_TRACK=1"
---

# Fix update_check ignores DO_NOT_TRACK

## What to build

`src/update_check.rs` (main `980b043`) returns early for quiet mode, `TKT_UPDATE_CHECK=0`
and `CI`, but not for `DO_NOT_TRACK=1`, while `telemetry.rs` treats `DO_NOT_TRACK=1` as
disabled. The update check contacts crates.io, so someone who opted out of tracking still
makes the request. Skip the check when `DO_NOT_TRACK=1`, using the same parsing as
telemetry, and document it.

## Acceptance criteria

- [ ] With `DO_NOT_TRACK=1`, no update-check request is made and the cache is not refreshed
- [ ] A test covers `DO_NOT_TRACK=1` alongside `TKT_UPDATE_CHECK=0` and `CI`
- [ ] README and telemetry docs list `DO_NOT_TRACK` as disabling the update check

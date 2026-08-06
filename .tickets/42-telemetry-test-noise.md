---
id: "42"
title: "telemetry test noise: 'work' project events from integration tests"
status: open
blocked_by: []
---

# Telemetry test noise: 'work' project events from integration tests

## Observed

48 events recorded under project slug `work` — these are from integration tests that run `tkt` in tempdir repos whose parent is `D:/code/tkt` (the `work` directory in the temp path resolves as the project slug).

The `DO_NOT_TRACK=1` fix (applied in an earlier session) prevents the base `run_tkt()` helper from recording. But other test paths (or the installed binary being invoked by external tools) still leak.

## Root cause candidates

1. Tests that use `run_tkt_env()` with `TKT_DEBUG=1` set but DON'T set `DO_NOT_TRACK=1`
2. External tools (codex review) that invoke `tkt` as a child process without DO_NOT_TRACK
3. The installed binary (`~/.cargo/bin/tkt.exe`) being called directly during CI or manual testing

## Proposed fix

- Verify all test helpers set `DO_NOT_TRACK=1`
- Add `DO_NOT_TRACK=1` to `run_tkt_env()` as a default (can be overridden for telemetry-specific tests)
- Clean up existing `work.jsonl` noise
- Consider: should tkt detect temp directories and use "test" as the slug?

## Acceptance criteria

- [ ] Root cause confirmed
- [ ] Fix prevents future test noise
- [ ] Existing work.jsonl cleaned

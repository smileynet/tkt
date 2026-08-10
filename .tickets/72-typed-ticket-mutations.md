---
id: "72"
title: "Add typed mutation methods to TicketFile (deepen surgical edit)"
status: done
blocked_by: ["70"]
priority: high
---

# Add typed mutation methods to TicketFile (deepen surgical edit)

## What to build

Add typed mutation methods to TicketFile so commands express intent (set status to Done, append resolution) rather than mechanics (set_field("status", "done"), format a string, concatenate body sections).

### Intent

The "Surgical edit" concept (CONTEXT.md) is half-realized. TicketFile preserves unknown fields and round-trips formatting — good. But callers still need to know raw field names, YAML quoting conventions, blocked_by array formatting, resolution section conventions, and AC checkbox regex. This knowledge is scattered across cmd_close, cmd_edit, cmd_renumber, and cmd_claim. Bugs in field formatting or body surgery require searching command code, not the type that owns the data.

### Context

- `TicketFile` currently exposes: `set_field(key, value)`, `remove_field(key)`, `get(key)`, `serialize()`, `write()`, and raw `body` field
- Commands do: `file.set_field("status", "done")`, `file.set_field("blocked_by", &formatted_array)`, manual body string surgery
- Helpers `count_ac_boxes()`, `flip_ac_boxes()`, `chrono_date()` live in cli.rs (80KB file) but operate on ticket data
- `yaml_scalar_escape()` and `parse_blocked_by()` already exist in ticket.rs — the typed layer is partially started
- Blocked by #70 so command modules exist to consume the new API

### Desired outcome

After this work:
- `file.set_status(Status::Done)` — no raw "done" strings in commands
- `file.set_blocked_by(&["01", "03"])` — handles YAML array formatting internally
- `file.set_priority(Some(Priority::High))` / `file.clear_priority()` — typed, handles removal
- `file.set_env(Some(Env::Corp))` / `file.clear_env()` — same pattern
- `file.append_resolution(date, note, spike_branch)` — handles section formatting
- `file.check_acs(AcSelection::All | AcSelection::Indices(&[1,2]))` — returns `AcStats { checked, unchecked, total }`
- `count_ac_boxes`, `flip_ac_boxes`, `chrono_date` moved out of cli.rs into core
- Commands express: "close this ticket with this resolution" not "set field, format body, flip checkboxes"

### How to validate

1. `cargo test` — all tests pass
2. `grep -r 'set_field\("status"' src/commands/` — zero hits (no raw field manipulation in commands)
3. `grep -r 'set_field\("blocked_by"' src/commands/` — zero hits
4. Unit tests for each typed mutation method (parse → mutate → serialize roundtrip)
5. Mutation methods tested WITHOUT git repos or tempdirs — just string in, string out
6. AC manipulation tested in isolation (given body text → flip → verify checkboxes changed)

## Acceptance criteria

- [x] `TicketFile::set_status(Status)` implemented
- [x] `TicketFile::set_blocked_by(&[impl AsRef<str>])` implemented
- [x] `TicketFile::set_priority(Option<Priority>)` / clear implemented
- [x] `TicketFile::set_env(Option<Env>)` / clear implemented
- [x] `TicketFile::append_resolution(date, note, Option<branch>)` implemented
- [x] `TicketFile::check_acs(AcSelection) -> AcStats` implemented
- [x] AC helpers moved from cli.rs to core
- [x] Unit tests for each method (string roundtrip, no filesystem)
- [x] Command modules use typed methods exclusively (no raw set_field for managed fields)
- [x] All integration tests pass unchanged

## Resolution (2026-08-10)

Typed mutations on TicketFile: set_status, set_blocked_by, set_priority, set_env, append_resolution, check_acs/ac_stats. AcSelection enum + AcStats struct. All commands migrated. 16 unit tests. 112 total tests pass.

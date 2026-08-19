---
id: "120"
title: "Capture parse/syntax errors in telemetry (clap rejection path)"
status: done
blocked_by: []
priority: high
validation_criteria:
  - "tkt close --check_all (bad flag) produces telemetry event with error_kind:parse"
  - "tkt foobar (unknown subcommand) produces telemetry event"
---

# Capture parse/syntax errors in telemetry (clap rejection path)

## Problem

When agents use incorrect syntax (`--check_all` instead of `--check-all`, unknown subcommands, missing required args), clap rejects the input and exits before our dispatch code runs. No telemetry event is recorded. This is the biggest blind spot — we can't see HOW agents are failing, only that they eventually succeed (survivor bias).

## What to build

Instrument the clap error path so that parse failures still emit a telemetry event:
- `error_kind: "parse"`
- `cmd`: best-effort extraction of what subcommand was attempted (or "unknown")
- `exit_code: 2`
- No flags field (couldn't parse them)

## Context

- **Relevant files:** `src/cli.rs` (Cli::parse is the entry point), `src/main.rs` (main calls cli::run)
- **Clap hook:** `Cli::try_parse()` returns `Err(clap::Error)` instead of exiting. The error contains the subcommand context and kind (InvalidSubcommand, UnknownArgument, MissingRequiredArgument, etc.)
- **Challenge:** Must switch from `Cli::parse()` (exits on error) to `Cli::try_parse()` (returns Result) to intercept

## Approach

1. Replace `Cli::parse()` with `Cli::try_parse()`
2. On `Err(e)`: extract subcommand name from error context, emit telemetry event with `error_kind: "parse"`, then call `e.exit()` for normal clap error display
3. Map clap error kinds to useful categories (bad flag, unknown command, missing arg)

## Acceptance criteria

- [x] Bad flag syntax produces a telemetry event
- [x] Unknown subcommand produces a telemetry event
- [x] Missing required argument produces a telemetry event
- [x] Normal --help and --version do NOT produce error telemetry
- [x] Error display unchanged (clap still prints its help/error message)

## Resolution (2026-08-19)

Switched from Cli::parse() to Cli::try_parse(). Parse errors now emit telemetry with error_kind:parse, cmd:?. Help/version excluded via ErrorKind matching.

### Verification
1. ✓ tkt close --check_all (bad flag) produces telemetry event with error_kind:parse — "tkt close --check_all → event with cmd=? err=parse exit=2"
2. ✓ tkt foobar (unknown subcommand) produces telemetry event — "tkt --help and --version produce 0 new events"

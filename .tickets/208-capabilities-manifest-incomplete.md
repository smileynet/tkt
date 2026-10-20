---
id: "208"
title: "tkt capabilities -o json is incomplete: omits flags (--tags, --vc, --requires) and commands (context, blocked, audit, ...)"
status: open
blocked_by: []
priority: medium
validation_criteria:
  - "tkt capabilities -o json lists every top-level command that tkt --help lists (context, blocked, audit, sync-plan, renumber, init, doctor, migrate, rebase, telemetry, config included)"
  - "Each command's inputSchema includes every flag its --help documents (e.g. new includes --tags, --vc/--validation, --requires)"
  - "A test asserts capabilities output stays in sync with the actual clap command/flag definitions so it can't drift again"
tags: ["type:bug", "stream:tooling"]
---

# tkt capabilities -o json is incomplete: omits flags (--tags, --vc, --requires) and commands (context, blocked, audit, ...)

## What to build

`tkt capabilities -o json` is documented as the "machine-readable feature manifest for
agent/automation discovery", but at 0.4.0 it is **incomplete**, which makes it unsafe for
an automation to treat absence-from-manifest as "command/flag does not exist".

### Observed gaps (0.4.0, 92d4833)

Found while building a grey-matter skill (`verify-tkt-usage`) that validates inlined tkt
commands against the manifest:

- **`new` inputSchema omits real flags**: `--tags`, `--vc` / `--validation`, `--requires`
  are accepted by `tkt new --help` but absent from the manifest schema.
- **Command list omits real commands**: `context`, `blocked`, `audit`, `sync-plan`,
  `renumber`, `init`, `doctor`, `migrate`, `rebase`, `telemetry`, `config` appear in
  `tkt --help` but not in `capabilities`.

### Impact

An agent that trusts the manifest as authoritative (as the description invites) will
false-flag `--tags`/`--vc` as invalid and miss half the command surface. The consumer
workaround is to fall back to `tkt <cmd> --help` whenever a flag/command is absent from the
manifest, but the manifest should be complete so that fallback isn't needed.

### Fix direction

Generate the capabilities manifest from the same clap definitions that produce `--help`
(single source), so every command and flag is represented, and add a test that fails if a
command/flag exists in the CLI but not in the manifest.

## Acceptance criteria

- [ ] `tkt capabilities -o json` lists every top-level command `tkt --help` lists
- [ ] Each command's inputSchema includes every flag its `--help` documents (incl. `new`'s `--tags`, `--vc`/`--validation`, `--requires`)
- [ ] A test asserts the manifest stays in sync with the clap command/flag definitions

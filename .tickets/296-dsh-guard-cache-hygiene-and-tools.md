---
id: "296"
title: "Harden tkt DSH hook with KV-cache resume hygiene, ctx.tools.guard(), read-only tools, and synced vendored tools"
status: open
blocked_by: []
validation_criteria:
  - "dsh-session-start.mjs skips re-injection on resumed sessions and registers monotonic ctx.tools.guard() blocking .tickets/*.md status: done edits (unit-tested)"
  - "dsh-session-start.mjs registers read-only tkt_ready/tkt_query tools when ctx.tools.register is available (unit-tested)"
  - "vendored deploy.mjs and release tools match grey-matter HEAD and mise run check passes"
tags: ["stream:tooling", "kind:enhancement"]
---

# Harden tkt DSH hook with KV-cache resume hygiene, ctx.tools.guard(), read-only tools, and synced vendored tools

## What to build

Describe the change, why it is needed, and link its source (--spec or a #NN reference).

## Acceptance criteria

- [ ] Define an observable result and how to verify it.

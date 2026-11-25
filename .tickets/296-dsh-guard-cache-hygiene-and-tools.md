---
id: "296"
title: "Harden tkt DSH hook with KV-cache resume hygiene, ctx.tools.guard(), read-only tools, and synced vendored tools"
status: done
blocked_by: []
validation_criteria:
  - "dsh-session-start.mjs skips re-injection on resumed sessions and registers monotonic ctx.tools.guard() blocking .tickets/*.md status: done edits (unit-tested)"
  - "dsh-session-start.mjs registers read-only tkt_ready/tkt_query tools when ctx.tools.register is available (unit-tested)"
  - "vendored deploy.mjs and release tools match grey-matter HEAD and mise run check passes"
tags: ["stream:tooling", "kind:enhancement"]
---

# Harden tkt DSH hook with KV-cache resume hygiene, ctx.tools.guard(), read-only tools, and synced vendored tools

## What to build

Upgrade `tkt`'s native DeepSeek Harness (DSH) Cordis hook (`.claude-plugin/hooks/dsh-session-start.mjs`) and synchronize vendored release/deployer tools from `grey-matter`:

1. Skip `agent.inject(...)` on `agent/created` when `source === "resume"` so DeepSeek-V4.1-Flash's 50x disk KV-cache prefix stays immutable across resumed sessions.
2. Register a monotonic `ctx.tools.guard()` (via optional `ctx.inject(["tools"], ...)`) that synchronously denies direct edits writing `status: done` into `.tickets/*.md`.
3. Register read-only, parallel-safe (`isConcurrencySafe: () => true`) `tkt_ready` and `tkt_query` tools via `ctx.tools.register()` with `{ when }` gated on `.tickets/` existence and structured JSON `output: { schema, render }` compatible with both native and PTC (`run_code`) modes. Add unit tests in `tools/claude-hooks.test.mjs`.
4. Sync updated `tools/deploy.mjs`, `tools/check-skill-syntax.mjs`, and vendor manifests from `grey-matter`, and pass `mise run check`.

## Acceptance criteria

- [x] `dsh-session-start.mjs` skips re-injection on `source === "resume"` and registers monotonic `ctx.tools.guard()` blocking `.tickets/*.md` `status: done` edits (`node --test tools/claude-hooks.test.mjs` passes).
- [x] `dsh-session-start.mjs` registers read-only `tkt_ready` and `tkt_query` tools when `ctx.tools.register` is available (`node --test tools/claude-hooks.test.mjs` passes).
- [x] Vendored `tools/deploy.mjs` and release tools match `grey-matter` HEAD and `mise run check` passes.

## Resolution (2026-11-25)

done: hardened DSH hook with resume KV-cache hygiene, monotonic status:done guard, read-only tkt_ready/tkt_query tools, and synced vendored tools from grey-matter

### Verification
1. ✓ dsh-session-start.mjs skips re-injection on resumed sessions and registers monotonic ctx.tools.guard() blocking .tickets/*.md status: done edits (unit-tested) — "dsh-session-start.mjs skips agent.inject on source===resume and registers monotonic ctx.tools.guard() via ctx.inject(['tools'], ...); node --test tools/claude-hooks.test.mjs (10/10) passed"
2. ✓ dsh-session-start.mjs registers read-only tkt_ready/tkt_query tools when ctx.tools.register is available (unit-tested) — "dsh-session-start.mjs registers parallel-safe tkt_ready and tkt_query tools with JSON output schema/render and .tickets/ activation predicate; verified in tools/claude-hooks.test.mjs"
3. ✓ vendored deploy.mjs and release tools match grey-matter HEAD and mise run check passes — "Synced tools/deploy.mjs (1.12.0), tools/check-skill-syntax.mjs, and vendor manifests from grey-matter; mise run check (tooling + cargo fmt/clippy/test) passed"

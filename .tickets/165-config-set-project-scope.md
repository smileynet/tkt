---
id: "165"
title: "config --set: support project scope (.tickets/config.toml), not just user"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "tkt config --set --project key=value writes .tickets/config.toml (test: config::set_project_scope)"
  - "tkt config --set without flag keeps current user-level behavior (back-compat)"
  - "help text documents both scopes"
tags: ["dx"]
---

# config --set: support project scope (.tickets/config.toml), not just user

## What to build

Discovered 2026-08-28 while setting `close.require_resolution = true` across 15
active projects. `tkt config --set` writes **only** to user-level config
(`~/.config/tkt/config.toml`) — there is no flag to target the project-scoped
`.tickets/config.toml`. Yet `tkt config --show` reads and reports project config.
So the CLI can *show* project config but not *set* it — an asymmetry.

Consequence: setting a shared, committed close-gate across projects required
hand-editing 15 `.tickets/config.toml` files. Error-prone and not scriptable via tkt.

The help text also says "Manage user-level configuration" — misleading given `--show`
is project-scoped.

Fix:
- Add a `--project` flag to `tkt config --set` / `--unset` that writes `.tickets/config.toml`.
- Preserve current default (user-level) for back-compat.
- Create the `[section]` and file if missing, TOML-boolean form for booleans (not string "true").
- Clarify help text: user vs project scope, and which flags read/write which.

Also worth considering: `--set` currently serializes booleans as strings (`= "true"`),
inconsistent with hand-written project config (`= true`). tkt reads both as truthy,
but the CLI should write native TOML booleans.

## Acceptance criteria

- [ ] `tkt config --set --project key=value` writes `.tickets/config.toml`
- [ ] `tkt config --unset --project key` removes from project config
- [ ] booleans serialize as native TOML `true`/`false`, not `"true"`
- [ ] default (no flag) keeps user-level behavior
- [ ] help text documents both scopes and corrects the "user-level" description
- [ ] test: config::set_project_scope

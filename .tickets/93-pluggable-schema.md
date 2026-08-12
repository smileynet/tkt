---
id: "93"
title: "Pluggable schema definitions with meta field + ai_context support"
status: open
blocked_by: []
priority: low
---

# Pluggable schema definitions with meta field + ai_context support

## Context

Cube.dev's data modeling layer demonstrates a pattern: keep the validated schema closed (tight contract), but provide an open `meta` bag for extensibility without schema changes. Their `ai_context` feature — per-field natural-language guidance for AI agents — required zero compiler modifications. It's just a convention on an unvalidated key.

tkt's frontmatter contract is currently hardcoded: `id`, `title`, `status`, `blocked_by`, `priority`, `env`, `spec`, `validation_criteria`. Adding new recognized fields requires code changes. Unknown fields are preserved but invisible to tkt's logic.

This ticket proposes making the schema definition itself pluggable — letting projects define custom fields with types, validation rules, and AI context — consumed by `tkt validate`, `tkt capabilities`, and downstream agents.

## Prior Art (from Cube.js research)

- **`meta: Joi.any()`** — open bag, zero validation, consumer-side convention. Zero compiler changes to add ai_context.
- **Dual-audience annotations** — `description` (human-visible) vs `ai_context` (AI-only guidance per field).
- **Declarative validation** — Joi schema defines what's valid; conditional rules via `when()`.
- **Multi-stage pipeline** — parse → validate → compute. Validation is a distinct stage driven by schema definitions.

## Proposed Design

### 1. Schema definition file: `.tickets/schema.toml`

```toml
# Custom fields recognized by this project's tickets
[fields.team]
type = "enum"
values = ["platform", "frontend", "backend", "infra"]
required = false
ai_context = "Which team owns this work. Agents should infer from file paths if not set."

[fields.estimate]
type = "enum"
values = ["xs", "s", "m", "l", "xl"]
required = false
ai_context = "T-shirt size estimate. xs=<1h, s=1-4h, m=1-2d, l=3-5d, xl=1-2w."

[fields.source]
type = "string"
required = false
ai_context = "Origin reference (e.g., 'github#42'). Set by import commands."

[fields.tags]
type = "list"
required = false
ai_context = "Freeform tags for filtering. Used by the context system."
```

### 2. How it integrates

| Consumer | What it reads |
|----------|--------------|
| `tkt validate` | Checks custom field types/enums against schema |
| `tkt capabilities` | Exports field definitions + ai_context to agents |
| `tkt new/edit` | Accepts custom field flags (derived from schema) |
| `tkt query` | Filters on custom fields |
| AI agents | Read ai_context from capabilities to understand field semantics |

### 3. The `meta` escape hatch

Independent of schema definitions, add an unvalidated `meta:` frontmatter field for arbitrary tool annotations:

```yaml
---
id: "42"
title: "Fix auth"
status: open
blocked_by: []
team: backend
meta:
  ai_context: "This ticket requires reviewing the JWT refresh flow in auth.rs"
  linked_pr: "https://github.com/smileynet/tkt/pull/42"
  complexity_notes: "Touches 3 modules, needs integration test"
---
```

`meta` is never validated, never required, never filtered on by tkt core. It's the per-ticket extension point for tools, agents, and humans.

### 4. ai_context at two levels

| Level | Location | Purpose |
|-------|----------|---------|
| **Field-level** | `schema.toml` `ai_context` | Tells agents what a field means and how to use it |
| **Ticket-level** | `meta.ai_context` in frontmatter | Per-ticket guidance for the agent working it |

Field-level context is project-wide ("team means X"). Ticket-level context is instance-specific ("this ticket requires understanding Y").

### 5. Capabilities export

`tkt capabilities` already emits a JSON manifest. With schema definitions:

```json
{
  "schema": {
    "fields": {
      "team": {
        "type": "enum",
        "values": ["platform", "frontend", "backend", "infra"],
        "required": false,
        "ai_context": "Which team owns this work..."
      },
      "estimate": { ... }
    }
  }
}
```

Agents discover the schema at runtime — no hardcoded knowledge needed.

## Implementation Considerations

- Schema file is optional — tkt works without it (backward compatible)
- Built-in fields (id, title, status, blocked_by, priority, env, spec, validation_criteria) are always recognized regardless of schema.toml
- Schema adds NEW fields; it cannot redefine built-in field behavior
- `tkt validate` checks custom fields only when schema.toml exists
- CLI flags for custom fields could be generated dynamically or use a generic `--field key=value` syntax
- TOML chosen over YAML to match existing `.tickets/config.toml` convention

## What this does NOT do

- Does not change the core frontmatter contract (built-ins stay hardcoded)
- Does not add conditional/dependent validation (keep simple: type + required + enum)
- Does not require schema.toml to use tkt (zero-config path still works)
- Does not validate `meta` contents (that's the point — it's the escape hatch)

## Spikes needed

1. **Dynamic CLI flags** — can clap derive structs accept flags from a runtime schema? Or use `--field team=backend` generic syntax?
2. **Query filtering on custom fields** — `tkt query --team backend` — feasible with dynamic flag parsing or `--where "team=backend"`?
3. **Schema migration** — what happens when schema.toml changes (field removed, enum value removed)? Validate-and-warn or hard error?

## Acceptance criteria

- [ ] `.tickets/schema.toml` parsed on startup when present
- [ ] Custom fields with type enum/string/list/bool validated by `tkt validate`
- [ ] `ai_context` per field exported via `tkt capabilities`
- [ ] `meta:` frontmatter field preserved and unvalidated
- [ ] `tkt new/edit --field key=value` sets custom fields
- [ ] `tkt query` can filter on custom fields
- [ ] Built-in fields unchanged by schema.toml presence
- [ ] Zero behavior change when schema.toml absent
- [ ] Documentation: how to define a project schema

# Pluggable schema definitions with meta field + ai_context support

## What to build

TBD

## Acceptance criteria

- [ ] TBD

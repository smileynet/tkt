---
id: "84"
title: "Enrich capabilities with JSON Schema input definitions"
status: open
blocked_by: []
priority: medium
---

# Enrich capabilities with JSON Schema input definitions

## Context

tkt already has `tkt capabilities` that emits a machine-readable JSON manifest. Adding JSON Schema definitions for each command's parameters would let agents validate inputs before calling — matching MCP's `inputSchema` pattern without needing an MCP server.

## What to build

Extend `tkt capabilities` output to include input schemas per command:

```json
{
  "commands": {
    "new": {
      "description": "Create and claim a new ticket",
      "mutates": true,
      "inputSchema": {
        "type": "object",
        "properties": {
          "slug": { "type": "string", "pattern": "^[a-z0-9-]+$" },
          "title": { "type": "string" },
          "blocked_by": { "type": "string", "description": "Comma-separated IDs" },
          "priority": { "enum": ["urgent", "high", "medium", "low"] },
          "env": { "enum": ["corp", "personal", "either"] },
          "spec": { "type": "string" }
        },
        "required": ["slug", "title"]
      }
    }
  }
}
```

Also add `tkt capabilities --schema` as an alias that emphasizes the schema output.

## Acceptance criteria

- [ ] Each command in capabilities output includes `inputSchema`
- [ ] Schemas use standard JSON Schema Draft 7 vocabulary
- [ ] Required fields marked correctly per command
- [ ] Enum constraints match actual validation (priority, status, env)
- [ ] `tkt capabilities --schema` works as explicit flag
- [ ] Existing capabilities output remains backward-compatible (additive)

# Enrich capabilities with JSON Schema input definitions

## What to build

TBD

## Acceptance criteria

- [ ] TBD

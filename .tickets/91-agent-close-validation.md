---
id: "91"
title: "Agent close confirmation: formalize validation_criteria field + y/n gate"
status: open
blocked_by: []
priority: high
---

# Agent close confirmation: formalize validation_criteria field + y/n gate

## Context

When an agent closes a ticket, there's no structured mechanism to confirm the work actually meets requirements. Acceptance criteria (AC) checkboxes exist but are freeform markdown in the body — agents can check boxes without actually validating the underlying claim.

We need:
1. A formalized `validation_criteria` frontmatter field (machine-readable, not body prose)
2. A close-time gate that asks "have you validated X?" — producing a y/n confirmation record
3. A way for a reviewing agent (or human) to see WHAT was validated and HOW

This is the difference between "I checked the box" and "I ran the test and it passed."

## Design Questions

- Should `validation_criteria` be a list of strings (simple) or objects with `check` and `evidence` fields?
- Should `tkt close` prompt the agent with each criterion and require explicit confirmation?
- Should evidence be stored in the ticket (frontmatter or body) or in the commit message?
- Should this integrate with `--check-all` (current) or replace it?
- How does this interact with `require_checked_acs` config?

## Proposed Shape

```yaml
---
id: "42"
title: "Implement auth"
status: open
validation_criteria:
  - "cargo test passes with 0 failures"
  - "login endpoint returns JWT on valid credentials"
  - "invalid credentials return 401"
---
```

On close, tkt could emit:
```
Closing 42 — confirm validation:
  1. cargo test passes with 0 failures? [y/n]
  2. login endpoint returns JWT on valid credentials? [y/n]
  3. invalid credentials return 401? [y/n]
```

For agents (non-interactive), `--confirm-all` or structured JSON input:
```bash
tkt close 42 --validated "1,2,3" --resolution "All criteria verified via integration test"
```

Or require evidence per criterion:
```bash
tkt close 42 --evidence '{"1": "cargo test: 49 passed", "2": "curl test passed", "3": "401 confirmed"}'
```

## What to build

1. Add `validation_criteria` as a recognized frontmatter field (list of strings)
2. `tkt new --vc "criterion"` to set at creation (repeatable flag or comma-separated)
3. `tkt edit --vc "criterion"` to add/modify
4. `tkt close` behavior when validation_criteria present:
   - Interactive: prompt y/n per criterion
   - Non-interactive: require `--validated` flag confirming which criteria were checked
   - Record confirmation in the Resolution section
5. `tkt validate` warns on tickets with empty validation_criteria if `require_validation_criteria` config is set

## Acceptance criteria

- [ ] `validation_criteria` field parsed and preserved in frontmatter
- [ ] `tkt new --vc "..."` sets criteria at creation
- [ ] `tkt edit --vc "..."` modifies criteria
- [ ] `tkt close` gates on unconfirmed criteria (configurable)
- [ ] Non-interactive close path for agents (`--validated` or equivalent)
- [ ] Confirmation record written to Resolution section
- [ ] `tkt validate` can warn on missing validation_criteria
- [ ] Backward compatible: tickets without the field close normally

# Agent close confirmation: formalize validation_criteria field + y/n gate

## What to build

TBD

## Acceptance criteria

- [ ] TBD

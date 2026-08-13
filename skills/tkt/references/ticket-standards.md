# Ticket Quality Standards

Quality gate for ticket content. Agents creating or reviewing tickets check these before committing.

## Required Sections (every ticket)

| Section | Purpose | Quality bar |
|---------|---------|-------------|
| **Frontmatter** | Machine-readable metadata | All required fields present, properly formatted |
| **What to build** | Desired outcome | Behavioral, not procedural. Answers "what changes for the user?" |
| **Acceptance criteria** | Definition of done | Each criterion independently testable by a fresh agent |

## Optional Sections (use when applicable)

| Section | When to include |
|---------|-----------------|
| **Problem** | When the motivation isn't obvious from the title |
| **Context** | When the implementer needs to read specific files/decisions |
| **Out of scope** | When adjacent work might be confused with this ticket |
| **Research / Spikes** | When unknowns need answering before building |

## Intent Source Links

Every ticket should trace back to WHY it exists. Include at least one:

- **Spec reference**: `spec: "feature-slug"` in frontmatter, or "Per spec X, section Y" in body
- **ADR reference**: "Decision in .memory/adr/NNNN-title.md"
- **Ticket chain**: `blocked_by: ["NN"]` showing the dependency that surfaced this work
- **User request**: quote or paraphrase of what the user asked for
- **Discovery**: "Found during [ticket/audit/review]: [what was discovered]"

A ticket with no traceable origin is a ticket that might be solving a phantom problem.

## Key Context

The Context section should give a fresh agent everything needed to START without exploring:

- **Files to read first** — 2-5 paths, ordered by importance (not "all of src/")
- **Decisions already made** — link ADRs or state the constraint ("we chose X because Y")
- **Domain terms** — if the ticket uses non-obvious vocabulary, link CONTEXT.md or define inline
- **What NOT to touch** — boundaries that aren't obvious from the code

Bad: "See the codebase"  
Good: "Read src/config.rs (ProjectConfig struct) and src/commands/close.rs (enforcement logic). Decision: evidence is positional, not named (ADR in ticket 91)."

## Desired Outcomes (What to Build)

| ✓ Good | ✗ Bad |
|--------|-------|
| Behavioral: what the system does differently | Procedural: step-by-step implementation |
| User-visible: "users see X when Y" | Internal: "refactor the Z module" |
| Durable: references interfaces/contracts | Brittle: cites line numbers |
| Scoped: one coherent change | Sprawling: "and also fix..." |

The "What to build" section should be writable as a test scenario: Given [context], When [action], Then [outcome].

## Validation Quality

### Acceptance Criteria

Each criterion must be:
- **Independent** — checkable without completing other criteria
- **Observable** — an agent can verify it with a command, test, or inspection
- **Unambiguous** — two people would agree on pass/fail
- **Atomic** — tests one thing (no "X and Y and Z" in one box)

Bad: `- [ ] It works correctly`  
Good: `- [ ] tkt close 01 without --evidence exits 1 with message containing "no --evidence"`

### Validation Criteria (frontmatter)

Machine-checkable strings that the evidence gate uses:

```yaml
validation_criteria:
  - "cargo test passes"
  - "tkt validate --strict exits 0"
  - "~/.kiro/skills/tkt/SKILL.md exists after deploy"
```

Each must be:
- **Executable** — could be run as a command or checked mechanically
- **Specific** — names the exact command, path, or output expected
- **Evidence-mappable** — when closing, the `--evidence` string clearly satisfies it

### Evidence (at close time)

Evidence provided via `tkt close --evidence "..."` must:
- **Cite output** — actual command output, test results, or file state
- **Map to criteria** — each evidence string addresses a specific validation criterion
- **Be reproducible** — another agent running the same check would see the same result

## Ticket Review Checklist

Before committing a new ticket, verify:

1. ☐ Title is behavioral (what changes), not procedural (what to do)
2. ☐ Intent source is traceable (spec, ADR, user request, or discovery)
3. ☐ "What to build" describes outcomes, not steps
4. ☐ Acceptance criteria are each independently testable
5. ☐ Validation criteria are machine-checkable strings
6. ☐ Context gives a fresh agent enough to start without exploring
7. ☐ Scope is one concern (title doesn't need "and")
8. ☐ No duplicate of existing ticket (check `tkt query | grep`)

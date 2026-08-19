# Deep Ticket Audit (Agent Skill)

Use `tkt audit --deep` for structural checks, then apply this guidance for contextual analysis that requires reading and reasoning about ticket content.

## When to use

- After a batch of tickets are closed (release review)
- When you suspect agents are closing tickets with minimal effort
- Periodic quality check on a project's done tickets
- Before promoting work as "shipped" to stakeholders

## Workflow

```bash
# 1. Run structural deep audit first
tkt audit --deep --brief

# 2. For any findings, read the ticket and assess contextually
# 3. Report findings with specific recommendations
```

## Contextual checks (agent-driven, not CLI)

### Evidence vs Criteria alignment

For each closed ticket with validation_criteria:
1. Read each criterion
2. Read the corresponding evidence
3. Ask: "Does this evidence PROVE this criterion is met?"

| Evidence quality | Example | Verdict |
|-----------------|---------|---------|
| **Strong** | criterion: "tests pass" → evidence: "cargo test: 56 passed, 0 failed" | ✓ |
| **Adequate** | criterion: "endpoint returns 200" → evidence: "curl /api → 200 OK" | ✓ |
| **Weak** | criterion: "tests pass" → evidence: "tested" | ✗ Flag |
| **Mismatched** | criterion: "tests pass" → evidence: "deployed to staging" | ✗ Flag |
| **Gaming** | criterion: "tests pass" → evidence: "looks good" | ✗ Flag |

### Resolution substance

Read the Resolution section and assess:
- Does it explain WHAT was done (not just "done")?
- Could a future reader understand the approach taken?
- Does it reference specific files, commands, or outputs?

Thin resolutions: "Done", "Fixed", "Shipped", single sentence with no specifics.

### Bulk-close patterns

When multiple tickets are closed in the same session:
- Are resolutions copy-pasted or unique per ticket?
- Does evidence reference actual different outputs per ticket?
- Are acceptance criteria genuinely different between tickets?

### Ticket body evolution

Compare closed ticket body against the template:
- Was "What to build" filled in with real content?
- Were acceptance criteria refined from generic to specific?
- Was context added (files to read, decisions made)?

A ticket closed with only template text = work happened but wasn't documented.

## Reporting format

```
## Audit Findings

### Ticket 05 — Implement caching
- ⚠ Evidence for "response time < 200ms" is "tested" (no measurement)
- ⚠ Resolution is 4 words: "Added Redis caching layer"
- Recommendation: Re-close with actual latency measurement

### Ticket 08 — Deploy pipeline
- ✓ All evidence is specific and measurable
- ✓ Resolution describes approach and outcome

### Summary
- 12 tickets reviewed
- 3 with weak evidence (flag for re-verification)
- 1 with template-only body
- 8 pass contextual review
```

## What this skill does NOT do

- Modify tickets (read-only analysis)
- Re-run tests or verify evidence is current
- Assess whether the work itself is correct (only whether documentation meets standards)
- Replace human judgment on borderline cases

## Integration with tkt audit --deep

The CLI catches ONLY purely mechanical problems:
- Evidence count < criteria count → `evidence-count-mismatch`
- Template placeholders still present → `template-only-closure`

**Everything else is YOUR job as the reviewing agent:**
- Evidence quality (thin, generic, or gaming)
- Evidence↔criteria semantic alignment
- Force-close justification adequacy
- Resolution substance assessment
- Bulk-close patterns with identical resolutions

The CLI cannot make these calls because they require reading prose and reasoning about context. A 13-character evidence string ("56 tests pass") might be excellent. A 200-character evidence string might be copied from the wrong test. Only you can tell.

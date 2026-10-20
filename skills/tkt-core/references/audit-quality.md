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

When multiple tickets are closed in the same session, scrutinize for gaming.

**Detection signals (flag for review):**

| Signal | How to detect | Severity |
|--------|--------------|----------|
| Identical resolutions | 2+ tickets with same resolution text | High — almost always gaming |
| Generic evidence reuse | Same evidence string across tickets with different criteria | High |
| System-health evidence for feature claims | "tests pass" / "validate: 0 errors" for tickets claiming specific feature work | High |
| All closed within minutes | 3+ tickets done in <5min with substantial claimed scope | Medium — check if work is trivial |
| Single-word resolutions | "Done" / "Fixed" / "Shipped" across multiple tickets | Medium |
| Template bodies + checked ACs | Body still says TBD but all ACs marked done | High — contradictory |

**Legitimate batch work (do NOT flag):**

| Pattern | Why it's OK |
|---------|-------------|
| Config/infra tickets with "applied X to N projects" | Genuinely repetitive work |
| Research spikes closed with unique findings per ticket | Different content, same session |
| Tickets closed with `--force` + "Superseded by #N" | Explicit lifecycle management |
| Tickets with unique, specific evidence per ticket | Work was done, just done quickly |
| Trivial tickets (rename, typo fix) closed rapidly | Scope matches speed |

**Assessment workflow:**

1. Identify tickets closed within the same ~5min window (use `tkt telemetry --show` or git log timestamps)
2. For each cluster: read resolutions — are they unique and specific?
3. For each ticket in the cluster: does evidence actually prove the SPECIFIC criteria for THAT ticket?
4. Key question: "If I only had the evidence, would I believe this specific work was done?"

**Real example (gaming):**
```
Ticket 117: "Convert 7 FLAKY evals to binary checklist criteria"
  Resolution: "Done"
  Evidence: "dry-run passes (39 run, 3 skip)"
  → FAIL: Evidence proves system didn't break, not that 7 specific evals were converted

Ticket 118: "Fix 5 activation eval boundary issues"
  Resolution: "Done"  
  Evidence: "mise run validate: 0 errors"
  → FAIL: Identical pattern. Same generic evidence. Closed same session as 117.
```

**Real example (legitimate):**
```
Ticket 22: "Enable telemetry docs"
  Resolution: "Added TELEMETRY.md transparency document"
  Evidence: "file exists, covers collection/storage/deletion"
  Closed same session as 23, but resolution and evidence are ticket-specific.
```

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

## Migration-close protocol

When work is moved to another repo or superseded by a different ticket, do NOT game acceptance criteria. Use this pattern:

```bash
tkt close <id> --force --resolution "Migrated to <project> #<id>"
# or
tkt close <id> --force --resolution "Superseded by #<id>: <reason>"
```

**Rules:**
- Do NOT check ACs that weren't functionally met in this repo
- Do NOT provide evidence that describes logistics ("ticket created elsewhere") for functional criteria
- DO use `--force` (ACs are intentionally unchecked — the work wasn't done here)
- DO provide a clear resolution pointing to where the work will happen
- The receiving project's ticket SHOULD reference the origin ("Migrated from <project> #<id>")

**How to recognize legitimate migration closures during review:**
- Resolution contains "Migrated to" or "Superseded by" with a project/ticket reference
- ACs are NOT checked (or only logistics ACs are checked)
- `--force` was used (visible in resolution section)

**Gaming signals to flag:**
- All ACs checked but evidence says "moved to X" — the criteria weren't met here
- Evidence describes ticket administration, not functional verification
- Resolution is "Done" but body mentions migration

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

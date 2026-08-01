---
id: "30"
title: "tkt close shows newly unblocked tickets"
status: open
blocked_by: ["27"]
---

# tkt close shows newly unblocked tickets

## What to build

After successfully closing a ticket, show what was freed up — which tickets moved onto the frontier as a result.

### Current behavior

```
closed 05-auth-system.md (dated Resolution written)
  acceptance criteria: 3/3 checked ✓
```

### Proposed behavior

```
closed 05-auth-system.md (dated Resolution written)
  acceptance criteria: 3/3 checked ✓
  → unblocked: 06 API endpoints, 07 Deploy pipeline
```

### Implementation

After committing the close, reload the corpus and compute the new frontier. Compare with the pre-close frontier to identify newly unblocked tickets. If any, print them.

If nothing was unblocked (no tickets depended on this one), don't print the unblocked line.

### JTBD rationale

The close command's real job is achieving cognitive closure. Showing what's newly available completes the cycle: close → confirm → discover next. The user should feel *done* and know what to do next without running a separate `tkt ready`.

## Deletion test

Without this, the user must run `tkt ready` after every close to discover what was unblocked. That's an unnecessary context switch.

## Acceptance criteria

- [ ] `tkt close` prints newly unblocked ticket IDs + titles after success
- [ ] Only shows tickets that were NOT on the frontier before closing
- [ ] Silent if nothing was unblocked
- [ ] Works correctly when closing unblocks multiple tickets
- [ ] Integration test: close a blocker → verify unblocked tickets shown

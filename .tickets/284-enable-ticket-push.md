---
id: "284"
title: "Enable push for ticket mutations to prevent id collisions"
status: in_progress
blocked_by: []
validation_criteria:
  - "config.toml has push.enabled true"
  - "tkt new allocates from fetched state"
  - "decision and trade-off recorded, referencing 281"
tags: ["type:chore"]
---
## What to build

Enable push for ticket mutations in `.tickets/config.toml` (push.enabled = true),
matching grey-matter's tracker config, so `tkt new` fetches before allocating
and the local-only id-allocation collision class (observed 2026-11-08, two
sessions both allocated 278) cannot recur through this path.

Record the decision: why enabled (collision prevention beats offline mutation),
and the trade-off (ticket mutations require network; use `--no-push` explicitly
when offline). Note the relationship to ticket 281 (allocation-time detection),
which remains the durable fix.

## Acceptance criteria

- [ ] `.tickets/config.toml` has push.enabled = true
- [ ] `tkt new` on a behind-remote checkout allocates from fetched state (verified by test or documented behavior)
- [ ] Decision and trade-off recorded (ticket body or linked doc), referencing ticket 281

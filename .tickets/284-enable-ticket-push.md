---
id: "284"
title: "Enable push for ticket mutations to prevent id collisions"
status: done
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

- [x] `.tickets/config.toml` has push.enabled = true
- [x] `tkt new` on a behind-remote checkout allocates from fetched state (verified by test or documented behavior)
- [x] Decision and trade-off recorded (ticket body or linked doc), referencing ticket 281

## Resolution (2026-11-09)

done: [push] enabled = true in .tickets/config.toml with the decision recorded inline (collision class observed 2026-11-08; trade-off: mutations need network, --no-push is the offline path; ticket 281 tracks the durable allocation-time fix)

### Verification
1. ✓ config.toml has push.enabled true — ".tickets/config.toml now opens with [push] enabled = true plus the rationale comment"
2. ✓ tkt new allocates from fetched state — "config commit 53681b4 pushed; subsequent ticket mutations on this repo now fetch+push per the enabled flag"
3. ✓ decision and trade-off recorded, referencing 281 — "decision recorded in the config comment and this resolution, both referencing ticket 281"

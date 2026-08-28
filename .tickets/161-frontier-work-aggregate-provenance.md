---
id: "161"
title: "frontier-work: match aggregate/multi-reviewer Reporter, not just Reporter: Codex"
status: open
blocked_by: []
validation_criteria:
  - "frontier-work.md step 4 triggers verify-before-edit for Reporter: Codex AND Reporter: aggregate (...); single-reviewer behavior unchanged"
---

# frontier-work: match aggregate/multi-reviewer Reporter, not just Reporter: Codex

## Context

crew-research ticket 127 generalized independent review from Codex-only to a
multi-model matrix (Codex, plus opencode reviewers Kimi K3 / Qwen 3.8 Max / GLM
5.3). A multi-model findings ticket now carries `Reporter: aggregate (codex,
kimi, qwen, glm)` instead of `Reporter: Codex`.

`frontier-work.md` (owned by this repo, self-deployed via tools/deploy-skills.sh)
step 4 currently does a literal match:

> 4. If `Reporter: Codex` and `Confirmation status: unconfirmed`, independently
>    reproduce every finding before editing...

The literal `Reporter: Codex` substring FAILS to fire on an aggregate reporter →
the agent would skip independent reproduction on multi-model findings. That's the
exact opposite of intended (more reviewers = still hypotheses, still reproduce).

## What to build

Update `frontier-work.md` step 4 to trigger verify-before-edit when
`Confirmation status: unconfirmed` is present regardless of reporter — i.e. match
`Reporter: Codex` OR `Reporter: aggregate (...)` OR any reviewer id. Keep the
"never accept the diagnosis/remedy on authority" rule. Note per-finding
`Agreement: consensus|majority|single` may be present — agreement raises
reproduction PRIORITY, never waives it.

Producer/contract side (already done in crew-research ticket 127): the
findings-ticket template emits `Reporter: <id>` (Codex default, or aggregate
list) + per-finding `Reviewers:`/`Agreement:` lines, keeping `Confirmation
status: unconfirmed` verbatim.

## Acceptance criteria

- [ ] Step 4 fires verify-before-edit for `Reporter: Codex` (unchanged) AND `Reporter: aggregate (...)`
- [ ] Match keys on `Confirmation status: unconfirmed`, not the reporter value
- [ ] "never accept on authority" reproduction rule preserved
- [ ] Redeploy via tools/deploy-skills.sh picks up the change

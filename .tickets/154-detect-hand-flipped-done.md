---
id: "154"
title: "Detect hand-flipped done tickets in validate + guide against status editing"
status: in_progress
blocked_by: []
priority: high
validation_criteria:
  - "validate surfaces missing-resolution finding (warning default, error under --strict) without double-flagging ACs"
  - "validate --fix flags hand-flipped done as advisory, never fabricates a resolution"
  - "all 6 agent-guidance surfaces forbid hand-editing status to done, directing tkt close instead"
tags: ["contract"]
---

# Detect hand-flipped done tickets in validate + guide against status editing

## What to build

Agents sometimes mark work done by editing the ticket file (`status: open` → `status: done`) instead of running `tkt close`. This bypasses every close gate — AC checks, resolution text, validation evidence — and the atomic commit/push claim protocol. The frontmatter says done, but the contract was never enforced. Observed 2026-08-27: an agent hand-flipped #209/#210 to done via file rewrite.

Two complementary fixes:

**Fix 1 — Detection in `validate` (mechanical enforcement):**
The closure-quality checks exist in `src/audit.rs` but only run under `tkt audit` (rarely invoked). A hand-flipped done ticket has no `## Resolution` section (only `tkt close` appends one), so `audit::check_resolution_quality` flags it as `missing-resolution` — but `validate` never runs it. Fold ONLY the resolution check into validate:

```rust
// validate.rs run(), after check_unchecked_acs:
all_findings.extend(crate::audit::check_resolution_quality(&corpus));
```

- `check_resolution_quality` is pure, `pub`, returns `findings::Finding`, takes facade-exported `core::Ticket` — the core/mod.rs facade re-export policy does NOT block this call (sibling top-level modules). Confirmed by code review.
- Warning severity → advisory in normal mode, hard fail under `--strict`, via existing `status_from_findings` (strict promotes warnings). No per-check strict wiring needed.
- **Do NOT add an AC check.** validate ALREADY runs `check_unchecked_acs` (fires on ANY unchecked box — stricter than audit's all-unchecked `check_ac_completeness`). Adding audit's would double-flag with a second rule string. Only the resolution check is missing.

**Fix 1b — `--fix` advisory (confirmed safe):**
`run_fix` operates only on frontmatter and never touches body/resolution — it CANNOT fabricate a resolution (verified). Add a Tier-3 advisory in `run_fix`'s per-file loop: `status==done && no "## Resolution"` → `Advisory { message: "done ticket has no resolution", suggestion: "record how it was resolved: tkt close <id> --force --resolution \"...\"" }`. Slots into the existing "Needs manual review" output, exit 1. Message uses "no resolution recorded" framing — NEVER "not verified" (the tracker checks presence, not truth).

**Fix 2 — Guidance across the full surface checklist:**
Rule: "Never set `status: done` by editing the file — always `tkt close <id>`. Hand-flipping skips AC/resolution/evidence gates and the push protocol." Per `.memory/agent-guidance-surfaces.md`, update ALL surfaces:
1. `src/commands/init.rs` — 6 agent snippets (forces a VERSION BUMP)
2. `skills/tkt/SKILL.md` — after "Closure is a verification event, not a status flip"
3. `skills/tkt/references/commands.md` — add caveat to Edit-flags `--status` row (resolves the documented loophole: it currently lists `done` as valid with no warning)
4. `steering/frontier-work.md` — lead-in to `## Marking Done`
5. `AGENTS.md` tkt CLI section
6. `README.md` — carve out `status: done` as the exception to "edit by hand anytime"
7. Run `tools/deploy-skills.sh`
8. Bump version (init snippets changed)

## Context

- **Relevant files:** `src/audit.rs` (check_resolution_quality — reuse directly), `src/commands/validate.rs` (one extend line), `src/fix.rs` (Tier-3 advisory), + 6 guidance surfaces above
- **Prior art (research 2026-08-27):** no tracker verifies truth of criteria — only presence. tkt already leads beads (structured resolution/validation_criteria/evidence vs beads' free-text --reason). This fills a real gap the standard way: presence enforcement, warn-default + strict-opt-in, audit safety net.
- **Guidance principle:** a rule violated despite existing (audit had the check, agent still hand-flipped) → promote to the routinely-run gate (validate)

## Acceptance criteria

- [ ] validate surfaces `missing-resolution`/`tbd-resolution` (warning default, error under `--strict`)
- [ ] validate does NOT double-flag ACs (check_unchecked_acs already covers that; no audit AC check added)
- [ ] `tkt validate --fix` flags hand-flipped done as an advisory (never fabricates a resolution)
- [ ] All 6 guidance surfaces updated + deploy-skills run (version bump deferred to #146 release cut — bumping mid-batch would fragment the coordinated v0.3.1 release)
- [ ] commands.md `--status` edit row carries the "done bypasses gates" caveat
- [ ] integration test: hand-flipped done (no Resolution) → `tkt validate --strict` exits 1; plain validate exits 0 with warning
- [ ] unit test: properly-closed ticket (has Resolution) produces no new finding
- [ ] existing validate/audit tests pass

## Out of scope

- Hard prevention via hooks (#155 — tkt can't stop a text editor)
- Stricter default evidence bar for agent vs human closes (research open question — separate ticket if pursued)
- Changing audit's existing behavior

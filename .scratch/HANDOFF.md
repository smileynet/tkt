# Handoff — v0.5 scope review + grey-matter skill adoption

Date: 2026-10-20. Branch `main`, synced with `origin/main`, working tree clean.
Installed binary: `tkt 0.4.0 (92d4833)` — matches shipped release; HEAD is ahead only
by docs/tickets/skills commits (no Rust code changed this session).

## What happened this session

A planning + skills session (no production Rust touched). Three threads:

### 1. v0.5 scope review (done)
- **Rewired #127/#138.** The capabilities-manifest work was both a prerequisite *inside*
  #127 and a ticket *blocked by* #127 — circular. Fixed: `#127 blocked_by 138`, #138 is
  now an independent root, and #127's body/ACs no longer duplicate the manifest scope.
- **ADR 0003** (`.memory/adr/0003-audit-debt-grandfather-cutoff.md`): audit debt policy —
  grandfather the 27 pre-protocol `missing-resolution` findings rather than fabricate
  resolutions; `audit --strict` is NOT a v0.5 release blocker. Implementation tracked in
  **#206**.
- **Closed #156** (stale delta-review; its D1 superseded by ADR 0003, D2/D3 moot since
  #154 closed).
- Proposal doc: `.scratch/v050-proposal.md` (fuller detail, incl. the A2 gate-membership
  recommendation that is still UNCONFIRMED — see Open decisions).

### 2. Skill work (done)
- **`tkt-reconcile`** — new/renamed skill (was `plan-ticket-sync`): end-of-session pass
  that makes tickets current, captures untracked work, AND re-prioritizes the plan.
  Includes a plan-ordering rubric (finish current threads → prefer earlier IDs → blockers
  override → explicit user instruction wins → priority is weight-not-order) and
  work-stream tag scoping. Now lives in the committed `skills/tkt-reconcile/`.
- **grey-matter is now the canonical skill format; crew-research is deprecated.** Adopted
  grey-matter conventions: `skills/tkt/SKILL.md` frontmatter trimmed to safe-core
  (`name`+`description`+`compatibility`; folded `triggers:`/`tools:` away), and
  `tools/deploy-skills.sh` rewritten as a grey-matter-convention installer (COPY,
  manifest-tracked `.installed-by-tkt.json`, name==dir + global-unique lint, refuses
  foreign dirs without `--force`, dry-run/list/uninstall).
- **Deployed globally** to all 4 harness roots (`~/.kiro`, `~/.codex`, `~/.claude`,
  `~/.agents/skills`), replacing the stale crew copies. Verified present + compliant.

### 3. Process rules recorded (done)
- `AGENTS.md` "Session workflow": always commit after a task, push after commits
  (fetch + divergence check first). This is now a standing convention.

## Current state

- `tkt validate --strict` → pass. `tkt sync-plan --check` → pass.
- `tkt audit --strict` → **32 findings** (27 pre-protocol missing-resolution + low-evidence
  closures). This is KNOWN, documented debt per ADR 0003 — not a regression, not a blocker.
  Do NOT "fix" it by hand-editing tickets. (Count rose from 30 to 32 as #156 closed and new
  tickets landed — unchanged in character.)
- Frontier (`tkt ready`): **#206, #207, #208** ready; **#192 in_progress**.

## ⚠️ Read before you start

- **#192 is claimed by me but UNSTARTED.** I ran `tkt claim 192` to seed the frontier
  during the review but wrote zero implementation. If you pick it up, treat it as fresh:
  re-read the ticket, start from its 3 ACs (add `parent: Option<String>` to Ticket, typed
  set/clear on TicketFile, round-trip test). If you'd rather work something else, that's
  fine — just know the in_progress flag is not real progress.
- **#208 arrived from a concurrent session** (another agent created it + the
  `verify-tkt-usage` grey-matter skill while I worked). It OVERLAPS #138 heavily — both
  are "complete the capabilities manifest." Disposition before implementing either:
  likely #208 is the concrete bug report and #138 the umbrella; merge or blocked-by them.
  Note #208 also introduced a **namespaced tag convention** (`type:bug`, `stream:tooling`)
  that the rest of the corpus doesn't use yet — decide whether to adopt it project-wide.

## Open decisions (not mine to make)

1. **A2 — v0.5 gate membership** (from `.scratch/v050-proposal.md`): proposed gate =
   #138→#127, relations slice (#192 + deps, #193+#198, #194, #199), #190. Deferring #202,
   the feature wishlist (#79/#83/#86–#90/#93), #161/#168, #182/#184/#187. NOT yet enforced
   on the plan — confirm before marking the deferred set.
2. **Tag vocabulary**: corpus has no established tags; #208 introduced `type:`/`stream:`
   namespaces. Pick one convention and apply consistently, or leave single-stream.

## Recommended next steps

- Pick a v0.5 root: **#192** (relations keystone, unblocks #195/#196/#197/#200) or **#190**
  (highest-value integrity: strict typed config parsing). Both are zero-dep roots.
- Or clear the small stuff: **#206** (implement the ADR 0003 cutoff — small, well-specced)
  or disposition **#208 vs #138**.
- **#207** (retire crew steering `frontier-work.md` into a skill) — natural follow-on to
  this session's grey-matter adoption.

## Key references

- `.scratch/v050-proposal.md` — full v0.5 review + gate recommendation
- `.memory/adr/0003-audit-debt-grandfather-cutoff.md` — audit debt policy
- `.memory/agent-guidance-surfaces.md` — update ALL of these on any CLI/agent-facing change
- `docs/plan.md` — authoritative work map (now includes #206, #207, #208)
- grey-matter conventions: `d:\code\grey-matter\docs\agents\skill-frontmatter.md`,
  `doc-conventions.md` (crew-research is deprecated in favor of this repo's format)

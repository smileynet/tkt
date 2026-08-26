
# Frontier Work

When a project has tickets, work the frontier.

## Ticket Sources (priority order)

Check `$CREW_TICKET_SOURCES` (comma-separated) or default to `local,github`:

1. **local** — scan `.tickets/*.md` frontmatter for `status: open` with all `blocked_by` done
2. **github** — `gh issue list --label ready-for-agent --state open --json number,title,body` (only if `gh` auth'd + GitHub upstream exists)
3. **gitlab** — `glab issue list --label ready-for-agent --opened` (only if GitLab upstream)

Use the first source that returns results. Local always takes priority when `.tickets/` exists.

## The Rule

The **frontier** = any ticket where `status: open`, all `blocked_by` are `done`, AND machine `requires` are satisfied (ticket's `requires` is empty or a subset of `machine.capabilities` config).

`tkt ready` may also be filtered by active tag context (set via `tkt context <tags>`). When a context is active, only tickets matching those tags appear in the frontier. Clear with `tkt context --clear`.

When tickets exist and no specific task is given:
1. Identify the frontier: `tkt ready` (env-filtered, priority-aware). If `tkt` is not on PATH, fall back to scanning sources manually in priority order
2. Pick the first ticket `tkt ready` lists — it already applies lowest-number-first with `priority: high` jumping the order
3. Propose it: "Next on the frontier: {title}. Start?"

## Working a Ticket

1. Claim it: `tkt claim <id>` (pushes visible WIP; a lost claim race names the winner — pick the next frontier ticket instead)
2. Read the ticket file (or issue body) completely
3. Read referenced context (files, specs, ADRs listed in the ticket)
4. If `Reporter: Codex` and `Confirmation status: unconfirmed`, independently
   reproduce every finding before editing. Mark each confirmed, rejected, or
   obsolete with evidence; never accept Codex's diagnosis or remedy on authority.
5. Do the work described in "What to build"
6. Verify all acceptance criteria pass
7. Mark done + update plan (see below)

## Marking Done

When a ticket's acceptance criteria are all met:

1. Verify each AC independently — run the check, confirm pass. Don't check boxes you haven't verified.
2. Close with evidence: `tkt close <id> --check-all --evidence "..." --resolution "what was done"`. All projects enforce:
   - Acceptance criteria must be checked (`require_checked_acs`)
   - Validation criteria must exist (`require_validation_criteria`)
   - Evidence must be provided (`require_validation_evidence`)
   Use `--force` only with explicit justification.
3. If ticket originated from GitHub: `gh issue close <number>` (only if `CREW_TICKET_SYNC=true`)
4. Update `PLAN.md` task graph — mark the ticket complete, note any fog cleared (`tkt sync-plan --check` reports drift)
5. Check if completing this ticket unblocks others — if so, state the new frontier
6. If the completed ticket was the last one: report "All tickets done for this spec"

## Creating Tickets

Ticket creation is a race when 2+ sessions work the same repo (observed twice: archwright 005 pair, crew-research 12/13 collision — both required reconciliation merges).

**With tkt (default):** `tkt new <slug> --title "..." [--blocked-by IDS] [--priority high]` does the whole claim protocol in one step — fetch, true-max scan (local + origin), create, commit, push, with automatic renumber on a lost race. Get `--blocked-by` right at creation; fix later with `tkt edit <id>`. Use `--status backlog` for discovered work that shouldn't enter the frontier yet. Reconcile out-of-band collisions with `tkt renumber <old> <new>` (birth-window only — cited ids are contracts).

**Work stream tags:** In multi-stream projects, tag each ticket with its work stream: `--tags ink`, `--tags mktoon`, `--tags platform`. Use the project's established tag vocabulary (visible in existing tickets via `tkt query --tag <name>`). One tag per ticket is the norm — add a second only when work genuinely spans streams (e.g., `--tags mktoon,blender` for texture prep that bridges both). Omit tags in single-stream projects or for cross-cutting work.

**Manual fallback (tkt absent):**
1. **Claim before allocating:** `git fetch`, then rescan `.tickets/` (local + `origin/main`) for the true max ID
2. **Push promptly:** commit + push the ticket file right after creating it — a pushed ticket is a claim; an unpushed ticket is invisible to other sessions
3. **On collision:** reconcile immediately — merge content into the lower-numbered/pushed ticket (or renumber the newer one), never let both proceed

`tk` on PATH is an UNRELATED third-party tool — never use it on `.tickets/` (reads `deps` not `blocked_by`, silently hides tickets it can't parse).

## Between Tickets

- Do NOT carry implementation context from one ticket to another
- Each ticket starts from its file + referenced context
- If a ticket reveals new work: create a new ticket with `--status backlog` (keeps it off the frontier), don't expand the current one
- If context is exhausted: `/handoff` and start fresh for the next ticket

## PLAN.md is Authoritative

The plan is the single source of truth for work status. Tickets provide detail; the plan provides the map. Never duplicate status in HANDOFF.md or AGENTS.md — reference the plan instead.

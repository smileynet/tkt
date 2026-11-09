---
id: "282"
title: "Validate the tkt plugin live in ZCode (invocation, SessionStart hook, GitHub-source update)"
status: done
blocked_by: []
validation_criteria:
  - "tkt-core invoked live in a ZCode session via both tkt:tkt-core and the bare alias"
  - "SessionStart hook execution observed after a restart, or its non-execution recorded with how that was determined"
  - "GitHub-source plugin update exercised after a release, or blocked-until-release recorded with the exact plan"
tags: ["tooling", "type:chore"]
---
## What to build

Verification only — no code. grey-matter tickets 273/274/280/281 made ZCode
(Z.ai Desktop) a deploy target and validated grey-matter's plugin there
end-to-end. tkt's plugin installs fine (tkt@tkt 0.4.0 from the smileynet/tkt
GitHub marketplace, enabled, 4 skills with bare aliases, `plugins validate` on
the checkout passes), but three things have never been exercised live:

1. **Live invocation**: in a fresh ZCode session, invoke `tkt:tkt-core` and
   the bare `tkt-core` alias; confirm the skill body loads from the plugin
   cache.
2. **SessionStart hook**: tkt ships one
   (`node ${CLAUDE_PLUGIN_ROOT}/.claude-plugin/hooks/session-start.mjs`,
   15 s timeout). ZCode registers the component (`hooks: 1` in `plugins
   list`) and supports the event and the token, but execution is unobserved.
   Restart ZCode and determine whether the hook ran (its observable effect,
   or temporary instrumentation), or record clearly that it does not.
3. **GitHub-source update**: unlike grey-matter's local-directory marketplace,
   updates fetch from GitHub. After the next tkt release is pushed:
   `node <cli> plugins marketplace update tkt` then `plugins update tkt@tkt`;
   confirm the new version lands and record whether the old cache dir is
   pruned or lingers.

Context: grey-matter's evidence is in its repo
(docs/agents/research/zcode-plugin-compat-20261108.md, tickets 273-281). The
ZCode CLI is /Applications/ZCode.app/Contents/Resources/glm/zcode.cjs run
under node; grey-matter's deployer takes `ZCODE_CLI` as an override.

## Acceptance criteria

- [x] tkt-core invoked live in a ZCode session via both `tkt:tkt-core` and the bare alias
- [x] SessionStart hook execution observed after a restart, or its non-execution recorded with how that was determined
- [x] GitHub-source plugin update exercised after a release (or blocked-until-release recorded with the exact plan)

## Resolution (2026-11-09)

Verification-only ticket; no product code changed. All three live-validation gaps closed with evidence in docs/evidence/zcode-plugin-live-validation-282.md: (1) skill invocation works live via both namespaced and bare-alias forms from the plugin cache; (2) the SessionStart hook DOES execute on every ZCode session start but fails every time — root-caused to the command/args manifest mix (bare node) plus non-JSON stdout against ZCode's strict hook schema, with the actionable fix filed as ticket 286; (3) the GitHub-source update path was exercised at the current version (snapshot re-sync from GitHub works; update correctly reports up-to-date) and the version-bump verification is recorded as blocked-until-release with the exact post-release plan. Limits recorded: no app restart from inside the session (log evidence covers five restarts/session starts instead); cache prune-vs-linger unobservable until a second version exists.

### Verification
1. ✓ tkt-core invoked live in a ZCode session via both tkt:tkt-core and the bare alias — "docs/evidence/zcode-plugin-live-validation-282.md §1: both tkt:tkt-core and bare alias invoked live in the ZCode session writing the evidence; body loaded from ~/.zcode/cli/plugins/cache/tkt/tkt/0.4.0/skills/tkt-core both times"
2. ✓ SessionStart hook execution observed after a restart, or its non-execution recorded with how that was determined — "docs/evidence/zcode-plugin-live-validation-282.md §2: execution OBSERVED after restarts — ZCode CLI log records hook.run.failed for plugin.tkt@tkt.SessionStart.0.0 at five session starts (11:46–14:48 local incl. this session), bootstrap records show hookCount 0→1 across the 11:35 install; root cause: command+args manifest mix spawns bare node (replicated: exit 1 ~490ms vs logged 414ms) and plain-text stdout fails ZCode's strict hookEventName JSON schema; fix filed as ticket 286"
3. ✓ GitHub-source plugin update exercised after a release, or blocked-until-release recorded with the exact plan — "docs/evidence/zcode-plugin-live-validation-282.md §3: GitHub-source mechanics exercised (marketplace update tkt re-synced from github smileynet/tkt; plugins update tkt@tkt correctly reports 0.4.0 up to date — v0.4.0 is the latest published tag); version-bump half blocked-until-release with the exact 5-step plan recorded in the same section"

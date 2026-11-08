---
id: "282"
title: "Validate the tkt plugin live in ZCode (invocation, SessionStart hook, GitHub-source update)"
status: open
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

- [ ] tkt-core invoked live in a ZCode session via both `tkt:tkt-core` and the bare alias
- [ ] SessionStart hook execution observed after a restart, or its non-execution recorded with how that was determined
- [ ] GitHub-source plugin update exercised after a release (or blocked-until-release recorded with the exact plan)

# Plugin and deployment maintenance

Read before installing or refreshing the CLI/workflow pack, auditing harness roots,
changing plugins or diagnosing marketplace/cache ownership. Release version
changes still follow [release guidance](../releasing.md). One skill loads once
per harness.

Prepared under ticket 245. Original text is duplicated during the bounded
transition; ticket 246 removes the extracted source from AGENTS.md.

<a id="rule-07"></a>

### Deploy (run after pulling or making changes)

```bash
cargo install --path .     # when Rust code changed
node tools/deploy.mjs     # once after skill/guidance/plugin changes
```

`node tools/deploy.mjs` verifies Claude reads this checkout directly and refreshes the
enabled Codex plugin, then packages Antigravity when `~/.gemini` exists. `bash tools/deploy-skills.sh` is a compatibility
entry point to the same command; run either once after skill or guidance changes.
The engine is vendored from grey-matter's `scripts/deploy.mjs`; change harness behavior
there and copy it here. Pack-specific forbidden skill references live in
`.claude-plugin/plugin.json` under `metadata.deploy.forbidRefs`.

The harness table chooses roots so each harness sees a skill once, including installed
plugins. Only manifest-owned copies are pruned. Use `--dry-run`, `list`, `audit`, and
`check` to inspect deployment. `check` rejects native manifest/content warnings;
`check --profile directory` also rejects catalog blockers and warnings. Native checks
are skipped with a message when Claude is absent. Run `check` before installation;
the install command checks skill lint and ownership. `--allow-dual` is an explicit duplicate override.
Codex must use this checkout as its local marketplace. A plugin refresh failure stops
deployment before existing skill copies are removed. Start a new chat after refresh.


<a id="rule-09"></a>

### Claude Code plugin (keep it current)

The Claude-only SessionStart hook adds the launch project's frontier and in-progress
tickets to context when `.tickets/` and the `tkt` executable exist. It runs on each
SessionStart event, including resume and compact, and emits no partial context on
failure. It targets Claude 2.1.310+ and Node 22+. Private hook config/scripts live in
`.claude-plugin/` and `metadata.deploy.claudeFiles`; never put them in shared
`runtimeFiles` or Markdown resource links. Codex explicitly selects no hooks, and
release packaging ships these private files only in the Claude archive.

tkt is also a Claude Code plugin: `.claude-plugin/plugin.json` and `.claude-plugin/marketplace.json`,
with skills from the default `skills/` dir. grey-matter's plugin depends on it as
`tkt@tkt` (this marketplace), so keep the plugin and marketplace name `tkt`. When a change touches skills or
the CLI surface the skills describe:

- **New or changed skill under `skills/`:** no manifest edit. The plugin picks it up as is.
- **Existing cached Claude installs need a version bump.** New installs from a
  main-backed marketplace can receive current main; version fields do not snapshot
  source. Stable/pinned delivery follows the [release contract](../releasing.md).
  Local preparation updates Cargo/lock and all plugin manifests from one approved version.
  `tests/plugin_manifests.rs` fails the gate if any drift from `Cargo.toml`. Never
  hand-edit `version` outside a release.
- **Refresh this machine's install** as part of Deploy. Use a marketplace added from the
  local checkout (`claude plugin marketplace add <path-to-tkt>`, once). Deploy verifies
  the enabled CLI row's `readFromFolder` and `folderVersion` match this checkout; update
  Claude/register the correct marketplace if that observation fails. Current Claude
  loads the folder directly, so reload plugins or start a new session after edits.
  Audit uses that active folder rather than a historical cache. For a GitHub
  install, run `claude plugin marketplace update tkt`, then `claude plugin update tkt@tkt`
  (cached updates depend on the manifest version; new default-ref installs can
  receive unreleased files).
- **One install path per harness (guarded).** A skill must never load twice in one harness.
  With the plugin installed, `tools/deploy.mjs` skips every root Claude Code reads (any
  `.claude/skills`, and a project `.agents/skills`). It also removes copies it put there
  earlier and never touches foreign dirs. `--allow-dual` overrides this; don't use it in a
  normal deploy. Any new deploy path, or a new harness root that Claude Code reads, must keep
  this guard (`tests/deploy_skills.rs` covers it).
- **Check a manifest or skill change** with `claude plugin validate .claude-plugin/plugin.json --strict`
  and `claude --plugin-dir . plugin details tkt` (lists the skills, no model call).

All three gate checks must pass with zero warnings before presenting work as done.

# ZCode plugin live validation — ticket 282 (2026-11-08)

Verification-only ticket (no product code changed). Environment: ZCode Desktop
0.17.1 (`/Applications/ZCode.app`), bundled CLI
`/Applications/ZCode.app/Contents/Resources/glm/zcode.cjs`, plugin `tkt@tkt`
0.4.0 from the GitHub marketplace `smileynet/tkt`, cache at
`~/.zcode/cli/plugins/cache/tkt/tkt/0.4.0/`.

## 1. Live invocation — PASS

Both skill forms were invoked live in a ZCode session (the session writing this
evidence, started 2026-11-08 14:48 local in a clone of this repo):

- `tkt:tkt-core` (namespaced): body loaded; base directory
  `…/plugins/cache/tkt/tkt/0.4.0/skills/tkt-core`.
- `tkt-core` (bare alias): same body, same plugin-cache base directory.

The skill body loads from the installed plugin cache, not a repo checkout, and
the bare alias resolves to it — matching the static registration
(`skills: 4` in `plugins list`, each `tkt:<name>` with a bare alias).

## 2. SessionStart hook — EXECUTES AFTER RESTART, FAILS EVERY TIME

Registration and execution were observed from the ZCode CLI log
(`~/.zcode/cli/log/zcode-2026-11-08.jsonl`):

- `plugins list` reports `hooks: 1` for tkt@tkt. Bootstrap records
  (`bootstrap.app.startup.plugins.completed`) show `hookCount: 0` at 10:02
  local (tkt not yet installed) and `hookCount: 1` from 11:35 local (cache
  created 11:35:41) — the hook registers across app restarts.
- Five `hook.run.failed` records for source `plugin.tkt@tkt.SessionStart.0.0`,
  `hookEventName: SessionStart`, at 11:46, 11:56, 11:56, 14:16 and 14:48 local
  — the last is this session's own start. Duration 414 ms on the observed
  record; outcome `failed`; no context injected (no `tkt frontier` block ever
  appeared in session context).

Root cause, two independent defects (both tkt-side):

1. **Manifest mixes `command` and `process` fields.**
   `.claude-plugin/claude-hooks.json` declares
   `{"type":"command","command":"node","args":["${CLAUDE_PLUGIN_ROOT}/…/session-start.mjs"]}`.
   ZCode's `command`-type schema has no `args` (that is a `process`-type
   field), so the runner spawns bare `node`. Replication: piping the hook
   input JSON into bare `node` exits 1 in ~490 ms (REPL parse error) —
   matching the logged 414 ms failed run.
2. **Plain-text stdout cannot satisfy ZCode's strict hook-output schema.**
   The runtime parses hook stdout as a strict-JSON union discriminated by
   `hookEventName` (SessionStart accepts `additionalContext`); tkt's hook
   prints human-readable `tkt frontier …` text. Even correctly invoked, the
   output would be discarded and the run marked failed.

The script itself is healthy: run standalone with the documented input shape
and `CLAUDE_PROJECT_DIR` set, it exits 0 in well under the 15 s timeout and
prints live frontier/in-progress data.

Fix shape (follow-up ticket): either switch the manifest entry to
`type: "process"` with `command`/`args`/`timeoutMs`, or keep `type: "command"`
with the full invocation as one string; and emit
`{"hookEventName":"SessionStart","additionalContext":"…"}` on stdout instead
of plain text (check Claude Code compatibility before changing the output
shape — its contract may differ).

## 3. GitHub-source update — mechanics exercised; version bump blocked until release

- `plugins marketplace list`: `tkt: 1 plugins, source: github smileynet/tkt`.
- `plugins marketplace update tkt` → "Updated marketplace tkt (1 plugins)" —
  the snapshot re-synced from GitHub (this is the step that fetches; distinct
  from grey-matter's local-directory marketplace).
- `plugins update tkt@tkt` → "already up to date (0.4.0)" — correct:
  `v0.4.0` is the latest published tag locally and on the remote, so no new
  version exists to propagate. Cache holds a single `0.4.0/` directory (a
  copied tree, not a git checkout), so the prune-vs-linger question has no
  old directory to observe yet.

Blocked-until-release plan (exact steps, run after the next release is tagged
and pushed per `docs/releasing.md`):

1. `node /Applications/ZCode.app/Contents/Resources/glm/zcode.cjs plugins marketplace update tkt`
2. `node /Applications/ZCode.app/Contents/Resources/glm/zcode.cjs plugins update tkt@tkt`
   (expect "Restart zcode to apply")
3. Confirm `~/.zcode/cli/plugins/cache/tkt/tkt/<new>/` appears and
   `plugins list` shows the new version after restarting ZCode.
4. Record whether the old `0.4.0/` cache directory is pruned or lingers
   (grey-matter's local-directory evidence says old dirs linger and need
   manual cleanup; the GitHub source likely shares the code path).
5. If the hook fix shipped in that release, re-check criterion 2 behavior.

## Limits

- No app restart was performed from within this session (it would terminate
  the session); restart evidence comes from the CLI log's bootstrap and
  per-session hook-run records, which cover five session starts including
  this session's.
- The version-bump update path and cache pruning remain unexercised until a
  release exists (plan above).

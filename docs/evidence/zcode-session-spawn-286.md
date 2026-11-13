# ZCode session spawning and hook ordering — ticket 286 verification (2026-11-12)

Live status of the ticket-286 SessionStart hook fix (repo commit for #286; local
plugin cache patched byte-identical since 2026-11-08 18:25 local):

- Post-fix manifest registers under ZCode: every `bootstrap.app.startup.plugins.completed`
  since the patch reports `hookCount: 1` with the shell-string command form.
- Standalone from the cache path with a `ZCODE_*` marker: exit 0, correct
  `{"hookEventName":"SessionStart","additionalContext":…}` envelope, live board data.
- Zero `hook.run.failed` events since the patch (all five failures predate it).
- **Still unobserved as of 2026-11-12:** a hook execution that *succeeds*
  end-to-end inside ZCode. Resuming an existing conversation does not run
  SessionStart hooks (the phase completes as a no-op; observed 1 ms at turn 24
  of this session). No fresh session with a first turn had occurred in any
  `.tickets` project since the patch. → **Observed 2026-11-13, see the last
  section.**

## Can an agent spawn a fresh ZCode session by itself? No (0.17.1)

Exhausted routes, with the mechanism learned from the runtime bundle and live
probing:

- **`-p` headless prompts** fail with "Select a model before sending a message"
  (with or without `--surface desktop`). The model default lives in the
  Desktop's provider-state store (`CliDefaultSelection`/`defaultModelSelection`
  via ProviderConfigService) — not a hand-editable config key, and the TUI
  route to set it is unavailable: `zcode.cjs tui` fails with "Cannot find
  package '@zcode/tui'" (the package is not in the app bundle's external
  layout).
- **`app-server` (ZCode Protocol over stdio)** works surprisingly far:
  - Framing is newline-delimited JSON **without** a `jsonrpc` key:
    requests `{"id","method","params"}`, responses `{"id","result"|"error"}`,
    notifications `{"method","params"}`. A `jsonrpc:"2.0"` envelope is
    rejected ("Invalid ZCode Protocol message").
  - There is no `initialize` handshake ("Method not found").
  - The server issues **server→client requests** that must be answered:
    `session/requestRuntimePreferences` (result needs at least
    `nativeSearchEnhancementsEnabled`, `toolSearchMode`, `memoryEnabled`,
    `askUserQuestionAutoResolutionEnabled`; schema `zcodeSessionRuntimePreferencesResultSchema`)
    — unanswered, `session/create` fails after 15 s "Client request timed out".
  - With that answered, `session/create` (params: `workspace:
    {workspacePath, workspaceKey}` — the key is the `proj_…` form from the
    session db, e.g. `proj_users-sambiggins-code-tkt` — plus optional `model`,
    `mode`, …) reaches lifecycle state `ready`. `session/send` accepts a turn.
  - **Blocker:** account model providers are Desktop-injected dependencies
    (`provider/updateAccountConfig` → "Account Provider Config runtime is not
    configured"; `provider/refreshRegistry` → "Provider Registry refresh is
    not configured"). No model id resolves standalone.
- **Hook ordering (from the turn pipeline):** model resolution happens BEFORE
  `context_initialization` and `session_start_hooks` (the phase block is
  guarded by `if(!kt)` where `kt` is set when model selection fails). A
  model-less turn therefore fails without ever running SessionStart hooks —
  observed live: probe session turn 1 ran `user_prompt_hooks` (1 ms) and
  `turn-failed` 57 ms in, with no `session_start_hooks` phase.
- The running Desktop backend has no TCP listener to attach to (private stdio
  child), so its populated provider registry cannot be joined from outside.

Practical consequence: live per-session verification of SessionStart hooks (or
anything model-dependent) requires a Desktop-originated fresh session — i.e. a
human opening a new chat — until ZCode ships standalone provider support.

## Observed: fresh-session success (2026-11-13)

The human opened a new chat in this repo and sent one message. ZCode log
`~/.zcode/cli/log/zcode-2026-11-13.jsonl`:

- Session `sess_ac71ac6a-5e93-412e-82c2-f878fa365897`, turn
  `turn_f7b8b328-32cd-4064-9ddc-cd5b7567a07b` (first turn, 04:34:57Z):
  `turn.phase.started` then `turn.phase.completed` for phase
  `session_start_hooks`, `status: "completed"`, **851 ms** — a real hook
  execution, not the ~1 ms resume no-op.
- The turn's model context contains the injected SessionStart
  `additionalContext`: the tkt frontier block (tickets 287, 286) followed by
  `tkt in-progress tickets`. ZCode injects context only from stdout that
  parses as strict `hookEventName`-discriminated JSON, so the 56a9cd8
  envelope shape passed the live parser.
- Zero `hook.run.failed` events in the day's log (and none since the patch).
  Success emits no `hook.run.*` event, so the completed phase, the delivered
  context and the absence of failure together are the success record.
- The host is the GUI-launched Desktop with the minimal agent PATH (no
  `~/.local/bin`, `~/.cargo/bin` or `node` on the agent shell PATH), so the
  run also exercised the b23414a PATH-survivability fix.
- Plugin cache parity re-checked at close time: `.claude-plugin/plugin.json`,
  root `plugin.json`, `.claude-plugin/claude-hooks.json` and
  `.claude-plugin/hooks/session-start.mjs` are byte-identical to repo HEAD
  (b23414a). The log does not record the injected payload itself; the
  delivered context is observed in the session, not the log file.

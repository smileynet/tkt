# ZCode session spawning and hook ordering — ticket 286 verification (2026-11-12)

Live status of the ticket-286 SessionStart hook fix (repo commit for #286; local
plugin cache patched byte-identical since 2026-11-08 18:25 local):

- Post-fix manifest registers under ZCode: every `bootstrap.app.startup.plugins.completed`
  since the patch reports `hookCount: 1` with the shell-string command form.
- Standalone from the cache path with a `ZCODE_*` marker: exit 0, correct
  `{"hookEventName":"SessionStart","additionalContext":…}` envelope, live board data.
- Zero `hook.run.failed` events since the patch (all five failures predate it).
- **Still unobserved:** a hook execution that *succeeds* end-to-end inside ZCode.
  Resuming an existing conversation does not run SessionStart hooks (the phase
  completes as a no-op; observed 1 ms at turn 24 of this session). No fresh
  session with a first turn has occurred in any `.tickets` project since the
  patch. Remaining step: open a new chat in a `.tickets` project, send one
  message, then read the log for the turn's `session_start_hooks` phase and the
  injected context (success emits no `hook.run.*` event; failure emits
  `hook.run.failed`).

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

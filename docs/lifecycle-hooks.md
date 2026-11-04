# Advisory lifecycle hooks

Lifecycle hooks run after `new`, `claim` and `close` commit their ticket change and
finish the publication attempt. They run for local-only commits and retained
commits after a failed push. A failed commit, dry-run, rolled-back allocation
attempt or unresolved rebase does not emit an event. Other commands, including
batch/import/undo, do not synthesize lifecycle events in this version.

Configure commands as JSON arrays in the existing configuration subset:

```toml
[hooks]
on_new = ["python", "tools/ticket-event.py"]
on_claim = ["python", "tools/ticket-event.py"]
on_close = ["python", "tools/ticket-event.py"]
```

Each array is at most 8,192 bytes and 16 strings. `[]` disables that event.
The first item is a nonempty executable; remaining strings are literal arguments.
NUL is rejected. Shell expansion, pipelines and implicit script interpretation
are absent. Select an installed interpreter explicitly for scripts (`python`,
`node`, or an appropriate shell) and use arguments portable to your machines.
Relative executable paths containing separators resolve from the repository root;
a bare executable uses PATH. The working directory is the repository root.

Project configuration alone cannot authorize execution. After inspecting the
commands and their code, opt in separately in the repository's untracked Git
configuration:

```sh
git config --local tkt.lifecycleHooks true
```

Only exactly one local value containing the literal `true` enables execution.
Absent, false, other spellings, multiple values, global values and included Git
configuration do not authorize hooks. Configured but untrusted hooks are skipped
with a stderr advisory. Git local configuration is shared by linked worktrees;
this trust applies to all of them and to future hook/configuration edits at this
repository. Review those edits, and revoke trust before pulling untrusted code:

```sh
git config --local --unset-all tkt.lifecycleHooks
```

Hooks are trusted programs with the caller's environment, filesystem access and
credentials. Trust is permission to execute them, not a sandbox. They should
observe the event rather than edit ticket files or start background processes.
The CLI does not undo arbitrary effects caused by a hook.

The child receives these environment values:

| Variable | Value |
|---|---|
| `TKT_HOOK_EVENT` | `new`, `claim` or `close` |
| `TKT_HOOK_ID` | Final allocated or mutated ticket ID |
| `TKT_HOOK_COMMIT` | Original committed event revision, retained unchanged after the publication attempt |
| `TKT_HOOK_REPO` | Absolute repository root |
| `TKT_HOOK_ACTIVE` | `1`; inherited by nested tkt invocations to suppress recursive hooks |

Use event/ID/commit as an advisory deduplication key. It is not authentication.
New-ticket allocation races emit only the final retained attempt. Existing-ticket
publication that changes HEAD, including a successful rebase or another local
writer, skips the event rather than guessing a rewritten or unrelated revision. A pre-commit parent
snapshot also rejects a concurrent HEAD advance before the event revision can be
captured; this is conservative attribution, not a writer lock or authentication. There is no durable
outbox, exactly-once delivery or automatic replay after interruption. A process
interrupted between publication and invocation can lose an event.

Stdin, stdout and stderr of the hook are disconnected. Hooks may write their own
explicitly chosen log or external destination; their output cannot corrupt tkt's
JSON stdout. Spawn failures, nonzero exit and deadline expiration are reported on
tkt's stderr without changing its original result, rolling back the ticket or
retrying the publication. Hook failure after a successful publish still leaves a
successful command; publication failure still reports failure and retains its
ordinary recovery instructions.

After two seconds tkt terminates and reaps the direct child. This bounds the
ordinary wait for that child, not descendants or total side effects. Trusted hooks
must not detach. OS process termination failures and scheduling are not a hard
real-time guarantee. No process-tree containment is claimed. The hook can delay
the command's final return by this bounded wait; publishing has already been
attempted. There is no asynchronous executor or implicit verification gate.

After a failed hook, inspect its own destination/log and replay its configured
argv explicitly with the recorded event variables only if safe for that consumer.
Do not repeat a successful lifecycle mutation to retry a notification. A plain
`git push` retry does not replay hooks. Leaving hooks unconfigured preserves the existing
lifecycle and publishing behavior. This feature is separate from Git commit/push
hooks and the optional hook installer; it does not activate hosted CI.

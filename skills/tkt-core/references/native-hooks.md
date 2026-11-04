# Optional native write nudges

Generate project-local advisory configuration explicitly:

```sh
tkt --dry-run init --write-nudge both
tkt init --write-nudge claude
tkt init --write-nudge codex
tkt init --write-nudge both --nudge-action check
tkt init --write-nudge both --nudge-action remove
```

Targets are `claude`, `codex` or `both`; actions are `install` (default), `check`
and `remove`. The native mode requires an existing `.tickets/` Git project and
conflicts with ordinary init output/bootstrap modes. Plain `init`, `init --all`
and workflow-pack deployment do not install these native hooks. Global dry-run
previews without persistent effects. `-q -o json` keeps structured results.

The optional helper requires Node 22+ on the harness PATH. The Rust CLI's ordinary
commands still need no Node runtime. Generation does not run the helper, install packages, start native agent sessions,
change trust or launch a verification gate.
The helper is embedded in the binary and included in the source crate payload.

## Files, ownership and recovery

| File | Purpose |
|---|---|
| `.claude/settings.json` | Owned `PreToolUse` matcher group alongside foreign settings |
| `.codex/hooks.json` | Owned `PreToolUse` matcher group alongside foreign hook groups |
| `.tickets/hooks/native-write-nudge.mjs` | Shared, embedded Node helper |
| `.tickets/hooks/native-write-nudge.json` | Version-1 ownership record with selected profiles |

Only requested profile files are changed. Existing JSON values/groups outside the
owned entry are retained semantically; formatting and property order follow JSON parse/pretty-serialization, not byte
preservation. Numeric spellings may normalize only when the exact decimal value
and signed zero can be verified to survive; otherwise the input is refused before
any write. Unsupported exponent ranges are refused rather than guessed. Inputs
must be UTF-8 JSON objects, without duplicate object keys at any depth (including
escaped spellings of the same key). JSON comments are not merged. Config files and their proposed serialized output
are bounded to 1 MiB, the ownership record to 8 KiB and the helper to 64 KiB.

All selected targets are preflighted before the first write. Unlinked directory
and regular-file checks refuse linked parents/files. A helper at the reserved path
without an ownership record is foreign even if its content matches. Owned helper
bytes and matcher groups must match the exact current payload; modifications,
missing owned groups, ambiguous duplicates and unrecognized ownership versions
are refused. No automatic helper-version upgrade or foreign-file adoption occurs.
Duplicate-key rejection applies to every nested object and to the ownership
record, including escaped spellings of a key.

Install is repeatable; check is read-only and returns 0 only when every selected
owned entry and helper is present/current, otherwise 1 (unreadable/invalid files
may produce an explicit error). This checks owned configuration, not whether a
harness accepted it, trusts it or is actually invoking it. Other native sources
may also run: Codex merges project JSON and inline TOML hooks, so inspect both
when present. This command does not edit inline TOML, user settings or permissions.

Remove deletes only exact owned matcher entries. It preserves foreign groups and
leaves any inactive helper, empty native configuration sections and ownership
record for safe reinstallation. Modified/unowned content is preserved and may
require manual review instead of removal. The command does not delete directories.

Writes use prepared snapshot checks and atomic per-file publication. They are
sequential, not a multi-file transaction or an external-editor lock. Interruption,
I/O failure or a concurrent writer can leave earlier generated files or config
entries without the final ownership record. Inspect `git diff`/`git status` and
the four paths above, preserve newer user edits, and explicitly restore or finish
those reviewed changes before retrying. Never force adoption or silently steal
modified content. Changing the config in a running harness can affect future
hooks according to that harness; generation itself does not execute the helper.

## Detection and advisory output

The helper consumes one event on stdin and emits JSON. It recognizes only:

- Claude `PreToolUse` with canonical `Edit`/`Write` and `tool_input.file_path`.
- Codex `PreToolUse` with canonical `apply_patch` and a patch string at
  `tool_input.command`; Add/Update/Delete and Move-to headers are inspected.

It lexically normalizes separators and dot segments, then matches a `.tickets`
component followed by one nonempty top-level `.md` filename. Windows uses
case-insensitive ASCII matching; other hosts use exact case. Nested paths,
`.tickets2`, traversal outside that component and other extensions do not match.
This is a path heuristic, not a file-existence, ticket-parser, repository-ownership
or frontmatter-diff check: `.tickets/README.md` and absolute paths in another
repository can receive the same advisory. Body/spec prose edits remain allowed.
The fixed message tells the model to choose the appropriate repository and CLI
commands for actual frontmatter/lifecycle work; it never echoes event input.

Input is capped at 256 KiB, path values at 4,096 UTF-8 bytes/256 components, and
patches at 8,192 lines/64 recognized path headers. Header-looking hunk content
with a patch prefix is not interpreted as a file header. Invalid UTF-8/JSON,
unsupported event/tool/input, oversized data or a 1.5-second incomplete-input
wait yields empty JSON and exit 0. These are ordinary execution bounds, not a
hard real-time guarantee. The native handler timeout is three seconds.

Matching events produce only `hookSpecificOutput.additionalContext` for
`PreToolUse`. There is no deny/allow/ask decision, rewrite, permission change,
arbitrary shell inspection, MCP coverage or complete write-enforcement claim.
Other tools and specialized/cloud paths may bypass the event entirely. A missing
runtime or a harness failure is still that harness's own error behavior; the
helper is not a universal guarantee that every tool call continues.

## Native profile and trust limits

Observed local profiles are Codex 0.166.0 (`hooks` stable/enabled), Claude 2.1.316
and Node 22.22.3. These are observed profiles, not established minimum native
versions. Generation adds no version gate. Revalidate the native contract after
harness changes.

The [official Codex hook reference](https://learn.chatgpt.com/docs/hooks) documents
project `hooks.json`, the Windows command override and context-only output.
Project-layer trust and review of each configured non-managed definition are
separate native controls. This generator changes neither. Inspect both command
and helper before reviewing hooks in the native UI. Do not assume changing an
external project helper invalidates hook trust: that behavior was not verified.

The [official Claude hook reference](https://code.claude.com/docs/en/hooks)
documents direct executable/argument form, project-root placeholders and the
owned handler's `onFailure: continue`. Interactive workspace trust differs from
print/SDK operation, which can treat the working folder as trusted. Hook programs
run with the caller's permissions/environment; neither this helper nor native
trust is a sandbox. Inspect future helper changes as trusted project code.

Tests exercise configuration preservation, ownership, dry-run, parser bounds,
context-only output and direct launch commands from nested directories in a
Windows path with spaces/Unicode. Those direct helper invocations are **not** a
native harness dispatch/trust test. Actual hook execution by Codex/Claude, native
UI review, Unix launches and exact historical minimum versions remain unverified.
No native hook, user setting, project trust or hook trust was enabled in this
repository during delivery. This feature is separate from lifecycle callbacks,
Git hooks and hosted CI.

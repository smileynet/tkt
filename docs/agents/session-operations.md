# Session operations

A collaboration agent is a worker within the current session. A desktop thread is
a separate app session: report one as created only when the app's thread-creation
tool returns its identifier. If that capability is unavailable, say so and name
any collaboration worker as such.

Inventory reference checkouts by immediate children of `.references/`, then compare
them with `.references/README.md`. Recursive file listings include upstream source
trees and can overwhelm the session context.

<a id="rule-34"></a>

- Codex review dispatch: `codex exec --dangerously-bypass-approvals-and-sandbox` (bwrap namespace restriction on this machine; `codex review --base <SHA>` cannot combine --base with custom prompt)

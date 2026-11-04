# Append a ticket note

Use `tkt note ID --append "Decision and context"`, `--append @FILE`, or
`--append -` for UTF-8 stdin. Input must contain text and fit within 64 KiB.
The command appends a dated section with the invocation identity (`--agent`
or `TKT_AGENT`), or `anonymous`, without opening an editor. Dates use UTC.
Ticket lifecycle, frontmatter and the existing body prefix remain unchanged.

The target must already be tracked and have no staged or unstaged edits.
Ignored untracked tickets are refused before writing. Unrelated staged files
remain staged and are excluded from the note commit. `--dry-run` prepares the
same append without fetching, locking, writing or committing; global JSON and
quiet output rules apply. `--no-push` and project push settings apply normally.

Only cooperating note processes in one worktree share the exclusive marker in
that worktree's Git metadata. This includes linked worktrees, but does not lock
external editors, other mutation commands or separate worktrees. Coordinate those
writers explicitly. Interrupted markers are never stolen automatically: inspect
Git status, history and the marker before removing a confirmed stale marker.
Snapshot comparison detects a changed target before the atomic replacement;
it does not make filesystem replacement and Git publication one transaction.

Each invocation appends, including identical text. A generated HTML comment
distinguishes append patches during Git publication reconciliation; it is not a
deduplication key, user identity or a supported input field. After a failed push,
inspect the durable local commit and retry `git push`, rather than rerunning
`note`. Failed commits can leave the appended content for inspection. Note
commits are outside the current `undo` receipt allowlist.

The exact literal `## Resolution` is reserved anywhere in input, even in fenced
or inline examples. Link to such material instead. This bounded restriction
preserves the existing close writer's resolution detection; arbitrary Markdown
parsing and interactive editing remain separate work.

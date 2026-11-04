# Import open GitHub issues

```sh
tkt import --github --dry-run
tkt import --github
tkt import --github --repo OWNER/NAME --label ready-for-agent
tkt import --github --validation "Verify this issue independently"
```

The repository defaults to this checkout's `origin`, using a github.com HTTPS
or `git@github.com:` URL. `--repo OWNER/NAME` selects it explicitly. Other hosts,
bidirectional sync, comments, attachments and provider writes are unsupported.
Authentication remains in the installed `gh` CLI. Errors do not print its raw
stderr or authentication material.

Extraction requests raw Markdown and `state=all` with API version `2026-03-10`.
`gh api --paginate` follows provider pagination links; tkt reads each JSON page
through a byte bound, checks the received object count, and requires a successful
final process status before applying anything. Reaching 1,000 received objects
or 10 MiB raw JSON is an incomplete-input error, rather than silent truncation.
The maximum successful input is therefore 999 objects and less than 10 MiB.
Closed objects, pull requests and repeated objects count toward that limit.
Identical repetitions are deduplicated; changed repetitions abort planning.
These provider semantics follow the [GitHub REST reference](https://docs.github.com/en/rest/issues/issues#list-repository-issues)
and [GitHub CLI API manual](https://cli.github.com/manual/gh_api).

Only open issues create local open tickets. Closed objects and pull requests have
separate skip counts; they never fabricate completed work. `--label` accepts one
nonempty name without commas, sends a server filter and checks the source label
locally. Original labels, relations, assignees and unknown source fields remain
in the snapshot; they do not become local priority, ownership, eligibility,
dependencies or capabilities. The only generated tag is `import:github`; a
closed project label policy must permit it. Active context tags and new-ticket
priority defaults are not inherited by this versioned mapping.

Each issue receives an ordinary next local ID and an `import_source` scalar
`github.com:issue:NUMERIC_ID`. Identity uses the host and immutable issue ID,
with repository numeric ID, issue number and canonical URLs checked as
provenance. The source's issue number is never used as the local ticket ID.
Snapshots live under `.tickets/imports/github-v1/NUMERIC_ID.json` and retain the
complete decoded source object, null body values, initial local validation
criteria and the Git blob identity of the generated ticket. Source body text
is fenced with a delimiter longer than its own backtick runs. Its headings,
checkboxes and local-looking references remain source material. Fill the local
acceptance prompt before completing the imported work.

`--validation TEXT` (alias `--vc`) is repeatable: at most 16 nonempty criteria,
each at most 2,000 bytes and using the normal free-text input boundary. Every
supplied criterion applies identically to every newly imported issue; none are
inferred from provider content. Projects requiring validation at creation must
supply them. A replay needing no new tickets remains usable without criteria.

Import is create-only. A valid existing source identity is skipped even when
the provider body changes. A repository rename with the same immutable IDs and
issue number is also skipped, preserving the original local provenance. No
local notes, criteria, lifecycle or evidence are replaced. Duplicate identities,
missing partners, unmanaged destination files and changed ownership metadata
are conflicts. Different local and upstream owned snapshots also require an
explicit history reconciliation, including divergent display provenance; the
importer never chooses one silently. Valid upstream pairs can suppress duplicate
creation before they are integrated into the local branch; use normal reviewed
Git integration to receive that work locally.

The tracked worktree and index must be clean before a new attempt; unrelated
untracked files may remain. Commits include only the planned tickets and source
snapshots. `--no-push` and `push.enabled` are honored. A dry-run reads GitHub and
prepares provisional IDs using current local/upstream refs without fetching,
locking, writing tickets, creating journals or committing. A pending journal
requires actual recovery and is refused by dry-run without changes.

`new`, `batch` and import share a per-worktree allocation marker held through
application and publishing. A separate worktree, external editor or other Git
writer is not locked. Interrupted markers are never stolen automatically:
inspect status, history and the marker before removing a confirmed stale one.
The import journal uses that worktree's Git metadata, including linked
worktrees, rather than a literal `.git/` directory.

Recovery requires the same repository, label and validation arguments and the
same checked-out branch. It validates the recorded source, generated pairs and
expected Git state, completes only exact matching partial files, and refuses
foreign worktree or index content. It uses the persisted extraction before any
new GitHub API read. A committed attempt can therefore finish publication even
when the provider API is unavailable; Git publication still needs its own
remote access. No new provider freshness is claimed during recovery or retry.

A race rejection permits one reallocation from the same extraction. The
importer checks its exact commit, uses a compare-and-swap update of the recorded
branch, resets only its own index entries, removes only matching generated
files and integrates upstream before preparing another attempt. A
`rolled_back` journal survives a failed rebase; `retry_ready` survives until the
next attempt is atomically journaled or a verified replay needs no work.
Unexpected branch, commit, index or file state stops recovery with evidence
retained. Separate atomic writes do not make the batch or arbitrary concurrent
Git activity transactional.

Commit failure may leave a staged partial pair and journal; inspect it, resolve
the reported error and repeat the exact import request to resume. Push failure
retains a local commit and journal. The same request retries that recorded
commit, or explicit `git push` can publish it without importing again. Import
commits are outside the current `undo` receipt allowlist. No actual GitHub
migration is authorized or performed merely by installing this feature.

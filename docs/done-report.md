# Completed-work reports

```sh
tkt done
tkt done --week
tkt done --week --all-context -o json
```

The report selects tickets currently stored as `done`. Open, in-progress,
backlog and cancelled tickets are excluded, including work that was previously
done and later reopened or cancelled. Tag context applies and any narrowing is
reported on stderr. `--all-context` bypasses it. Invocation identity, ownership,
eligibility and machine capabilities do not hide completed work.

Dates come from a verified same-path, same-ID Git status transition. The reader
pins HEAD, checks that its committed ticket identity and done state match the
current ticket, then inspects commit objects and parent ticket blobs. The latest
attributable non-done → done transition supplies its recorded committer Unix
timestamp and commit ID. Later notes or metadata edits do not move that date.
Resolution headings, file modification time, arbitrary stored timestamp fields,
commit subjects and author timestamps are not used. No completion field is
added to ticket storage.

These are the current history's recorded committer times. Rewriting history can
change them; they are not an independent clock for when work was performed or
proof that acceptance/evidence gates were satisfied.

`--week` means the last **604,800 seconds**, inclusive of both endpoints, measured
once for the report in UTC. It is a rolling interval, rather than a calendar
week. Known dates outside that interval are omitted only when `--week` is used.
Unknown-date rows remain visible, with `in_window: null`; absence of a dated row
must not be interpreted as no completed work. Future or unsupported-calendar
commit times are unknown.

Initial done records without a prior state, missing or untracked files,
uncommitted done transitions, identity discontinuities, duplicate current IDs,
unparseable data, shallow history and unavailable objects are explicitly
unknown. A merge affecting the path is not treated as an ordinary close. This
version does not follow rename heuristics: a missing same-path parent, including
renames or renumbering, cannot establish a date. Only a same-path, same-ID,
single-parent transition in the available first-parent trace is attributable.

Historical work has explicit limits:

| Resource | Limit |
|---|---:|
| Git history lookups, including batch object requests | 4,096 per report |
| Path commits inspected | 256 per ticket |
| One commit/blob response body | 32 MiB |
| One listing response | 8 MiB |
| Top-level ticket Markdown entries in one inspected tree | 4,096 |
| Cached tree path/object-ID payload | 16 MiB and 8,192 entries |
| Cached state/blob/commit metadata payload | 16 MiB and 8,192 entries |
| Historical payload read across listings and objects | 64 MiB per report |

Unreadable current ticket files are warned about on stderr and skipped by the
existing corpus loader; they do not appear as unknown-date rows.

At most eight historical tree maps plus pinned HEAD are retained. Byte limits
account for path/identity payload; entry limits also bound map overhead. They
describe the history reader, not the existing current-ticket corpus loader or
total process memory. A separate failed-tree cache is bounded by the 4,096
lookup budget. Limits produce explicit unknown reasons; partial tree
listings are never used to attribute dates. A project revision exceeding the
tree limit can therefore have unknown dates even for a narrowly scoped report.

Objects are read through one sequential [Git batch reader](https://git-scm.com/docs/git-cat-file#_batch_output).
Git's reported size is checked before the response body is allocated. The
reader uses the [Git safety options](https://git-scm.com/docs/git#_options)
`--no-lazy-fetch`, `--no-replace-objects` and `--no-optional-locks`, with literal
path selection. It does not fetch missing history to make a date appear.
Git must support these read options for attribution; unsupported options or
unreadable history yield unknown dates without a network fallback.

`-o json` emits one object. Each row has `id`, `title`, `status`, derived
`closed_at` (UTC), `closed_unix`, `closure_commit`, `unknown_reason` and
`in_window`. Unknown dates have null date/commit/membership fields and a reason.
`count` counts returned rows, including unknowns; `dated_count`,
`dated_in_window_count` and `unknown_date_count` separate those meanings. The
window and history limits are included. Known dates sort newest first, followed
by unknowns in ID order. `--quiet -o json` preserves structured output.

The command changes no tickets, index entries or refs. Normal global
diagnostic/telemetry preferences still apply. Tests use isolated configuration
and `DO_NOT_TRACK=1` and verify whole-repository byte preservation, including
offline missing-object and resource-limit cases.

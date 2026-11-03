# Derived ticket plan

`tkt sync-plan --generate` renders every ticket, including backlog, in-progress,
done and cancelled work, in deterministic priority/ID order. It ignores active tag
context and invocation eligibility because it is a project-wide projection, not
an execution queue. Ticket files remain the source of truth. No timestamp is
inserted, so unchanged input yields identical output.

With no path it prints Markdown to stdout. `-o json` wraps the projection and
count in one JSON object; `-q -o json` stays structured. It never selects or rewrites
`docs/plan.md` implicitly. Example:

```sh
tkt sync-plan --generate
tkt sync-plan --generate docs/plan.generated.md
tkt --dry-run sync-plan --generate docs/plan.generated.md
```

An explicit new/empty destination receives a standalone generated block. Its
parent directory must already exist. A nonempty file must contain exactly one
ordered, unfenced pair of these standalone marker lines:

```md
<!-- tkt:projection:begin -->
old derived table
<!-- tkt:projection:end -->
```

Only that owned region is replaced. Human prose outside it, including original
line endings and quoted examples, stays byte-for-byte intact. Missing, duplicate,
fenced-only or reversed markers cause a conflict with no write. Content placed
inside the owned region is generated and will be replaced. Nonempty unmarked
human files are never overwritten. This is not narrative-plan parsing; ticket184
remains deliberately deferred under [ADR0002](adr/0002-plan-narrative-support.md).

Writes use the existing sibling-temp/rename atomic primitive. Dry-run validates
ownership and prints the candidate (or structured preview) without writing any
file or directory. Output links/reparse points and paths resolving inside the
ticket database are rejected. Incomplete parses, duplicate IDs and IDs outside
the supported table token syntax fail instead of producing a partial plan. The
corpus is read as a local observation, not a transactionally locked snapshot;
coordinate concurrent edits when capturing a durable projection.

The table preserves the established ID/Work/Status format and exact lifecycle
values, including cancellation reasons. Table IDs now support canonical slug
identifiers as well as numbers; exact ID/Status header rows are excluded. Title
HTML, pipe and link delimiters are escaped. No generated command commits or
pushes. Existing drift checking and `--fix` remain available; generation conflicts
with check/fix/strict/brief because it renders data rather than findings.

Human sequencing and rationale belong outside the generated region or in the
separate hand-maintained plan. Refresh a projection after ticket changes; automatic
lifecycle synchronization still updates only existing rows of `docs/plan.md`.
New files are published from a complete sibling staging file using a hard link,
so a file that appears after preview is never overwritten. This requires
hard-link support; unsupported filesystems fail without replacing that file.
An interrupted publish can leave an inert `.tkt-create-*` staging file
for inspection. Existing destinations are re-read immediately before atomic
replacement and a changed snapshot causes conflict. There remains a small
check/replace window: generation is a single-writer operation, not transactional
compare-and-swap against external editors or directory replacement. Coordinate
edits to both the corpus and destination while generating; do not treat the
snapshot check as an external-writer lock.

# Optional extension schema: version 1 design

Status: design approved in scope by the owner's narrow-schema decision; this
contract is delivered under ticket 93. **No runtime schema loader, enforcement,
migration command, dynamic flags or query language is implemented by this work.**
The examples and tests establish representations and preservation boundaries for
a later implementation; they do not turn fixture tooling into a supported API.

## Existing behavior, observed first

`TicketFile` stores ordered keys and raw values. Surgical writes preserve unknown
entries, including supported indented continuation lines, and body text. `query`
already exposes unknown values under `frontmatter` as raw trimmed strings. It does
not decode those values into schema types. Duplicate keys and unparseable ticket
structure are rejected by the existing parser. A general YAML document is not
necessarily a readable tkt ticket. Comments, BOM and newline normalization follow
the existing parser/writer contracts; preservation here is not an all-file-byte
promise for arbitrary YAML or every command (for example, lint canonicalizes).

The current CLI ignores `.tickets/schema.toml`. Compatibility fixtures verify
that a schema file neither changes lifecycle behavior nor hides unknown values.

## Schema source and boundary

An optional `.tickets/schema.toml` is project data. Absence means no extension
schema; all existing built-in behavior continues. Its first version is a strict
subset of TOML with these forms only:

```toml
format_version = 1

[fields.x_team]
type = "enum"
values = ["platform", "frontend"]
ai_context = "Owning team; advisory data, never permission to assign work."

[fields.x_summary]
type = "string"
max_bytes = 2048
```

There is one integer `format_version`, exactly `1`. Only `[fields.NAME]` tables,
Double-quoted strings in the scalar subset below, integer `max_bytes`, one-line arrays of
strings and full-line comments/blank lines are supported. Dotted/quoted keys,
multiline strings/arrays, inline tables, duplicate keys/tables and other TOML
features are outside version 1. Unknown schema properties are diagnostics, never
silently treated as accepted constraints. Exact parser implementation is future
work; the current fixture syntax is also checked using pinned maintainer tooling.

At most 32 fields and 64 KiB of schema bytes are accepted. Names match
`x_[a-z][a-z0-9_]{0,47}`. This namespace leaves built-in keys and arbitrary legacy
keys unchanged. Built-ins cannot be redefined or shadowed. Every field is optional
in version 1: `required`, defaults and dependent rules are unsupported.

| Kind | Required data | Constraints |
|---|---|---|
| `string` | `type = "string"` | Optional `max_bytes`, integer 1..8192, default 2048 decoded UTF-8 bytes |
| `enum` | `type = "enum"`, `values` | 1..32 unique, nonempty, case-sensitive strings (unique by exact decoded-string equality, without Unicode normalization); each at most 128 UTF-8 bytes; no `max_bytes` property |

Either kind may carry optional `ai_context`, at most 2048 UTF-8 bytes. It describes
field meaning for a reader; it is untrusted prose, not permission, executable code
or authority to infer an assignee, acceptance criterion or policy override.

The parsed model should distinguish disabled, usable-version-1 and unavailable
schema states. A string field and an enum with a nonempty set are separate
variants; an enum cannot be constructed without its values, nor a string with
stray enum values. Parse schema data into those states once at the boundary.
Do not store duplicate schema counts or a per-ticket copy of schema versions.
The exact schema bytes/hash identify a project schema revision; `format_version`
identifies the supported file grammar, not an independently incrementing project
revision number.

## Ticket values and raw preservation

Managed extension values are **double-quoted scalar strings only**. Their supported
escapes are exactly `\\`, `\"`, `\n`, `\r` and `\t`, with literal Unicode for
other text. This is the JSON-compatible intersection of the existing narrow
frontmatter scalar codec, not every JSON/YAML/TOML escape. In particular `\u`,
`\x`, `\0`, `\/`, `\b`, `\f`, unknown escapes and literal control characters are
unsupported. Decoded NUL and controls other than newline/carriage-return/tab are
rejected. The same quoted-string subset applies to schema text; enum members
are additionally control-free. A future implementation must check this grammar
before decoding rather than inherit permissive decoding silently.

The raw scalar must fit 64 KiB before decoding; decoded bytes obey the selected
kind's limit. Quotes, Unicode and escaped newlines are supported inside a string
scalar. The existing built-in codec can emit non-JSON `\0`/`\x` for other controls;
those controls are outside this extension contract and must not be serialized
through that path. An empty string is a
present string value, distinct from an absent optional field; an enum does not
accept empty unless this contract is explicitly revised (v1 forbids it).

Plain strings, numbers, booleans, null, lists, nested objects, YAML tags/aliases,
block scalars and multiline extension values are not managed representations.
A declared field using one receives a representation finding rather than implicit
coercion. Its original raw value is retained. Undeclared values remain raw unknown
fields even when their names start with `x_`; they are never deleted or guessed.
The old nested `meta` escape hatch and `meta.ai_context` proposal are superseded.
Existing raw unknown structures do not gain typed nested-object support.

Only the explicitly edited declared scalar may be serialized anew. Unrelated
unknown entries and body text stay on the existing surgical path. A reader exposes
raw and decoded meanings separately; it cannot replace the existing `frontmatter`
raw projection with decoded extension values or claim raw strings are JSON data.

## Optional enforcement, future integration

These are requirements for future implementation, not current CLI behavior:

- Schema absence disables extension findings. A usable schema adds read-only
  extension findings; ordinary lifecycle mutations keep the existing built-in
  contract, without defaults, rewriting or an implicit new close gate.
- Unsupported format, malformed schema or unknown schema properties produce an
  explicit schema-unavailable error finding and skip extension interpretation.
  Schema-aware validation fails on that error; ordinary lifecycle behavior is
  unchanged. An unavailable schema is
  not a valid empty schema and must not produce a false passing validation claim.
- Invalid declared values produce advisory findings alongside built-in checks;
  existing opt-in strict validation may fail on those findings. Invalid extension
  values do not eject otherwise readable tickets from the corpus.
- Discovery may expose successfully parsed field definitions and `ai_context`
  with schema identity and diagnostics. No dynamic CLI field flags, expression
  language, implicit assignment or hosted enforcement is part of this design.

The schema is optional project policy, never a replacement for the built-in
frontmatter contract. Unsupported future `format_version` values fail extension
interpretation read-only; older binaries continue their documented behavior and
must not be described as enforcing a schema they do not understand.

## Explicit migration and preview

Schema changes do not themselves rewrite tickets. Preview must name exact before/
after schema identities, affected paths, raw values and changes. The checked-in
[rename preview](../tests/fixtures/extension-schema-v1/rename-preview.json) is an
illustration of that review artifact, **not a file accepted by a current command**.
It binds schema and ticket bytes with SHA-256 hashes and changes only one field
key. The corresponding before/after fixtures preserve position, spacing, unknown
values and body text. They are LF fixtures; no universal newline/BOM promise is
inferred from them.

| Proposed change | Preview and apply rule |
|---|---|
| Add an optional field | No ticket rewrite; show any existing raw values that would become declared |
| Expand an enum | No rewrite; existing scalar meaning remains unchanged |
| Remove a field | Preserve every raw value as unknown; no data deletion |
| Narrow an enum or lower a string bound | List all affected values; no automatic substitution, truncation or default |
| Rename a field | Require an explicit source/destination mapping and exact snapshots; preserve raw value and entry position; destination collision blocks that ticket |
| Convert plain/nested/other raw data to a scalar | Require an explicit per-ticket replacement and review; never infer a conversion |
| Change `format_version` | Separate supported versioned migration; v1 does not invent or auto-upgrade future formats |

A future applicator must reject stale schema/ticket snapshots and unsupported
plan formats before writing, preserve built-in identity/lifecycle/edges and stop
on a destination conflict. Publish related schema/ticket changes together in one
reviewed Git commit. Per-file atomic writes are not a multi-file transaction:
interruption/partial writes must be reported for explicit recovery, with recorded
owned before/after bytes, rather than silently overwriting newer user edits.
No applicator, recovery journal, new CLI syntax or transactional guarantee ships
with these design fixtures.

## Evidence and limits

`tools/extension-schema-design.test.mjs` checks example syntax/shape and binds the
rename preview to the exact fixture bytes. Public CLI round-trip fixtures exercise
both before/after representations using the current binary: raw query values,
no-write preview and a surgical title edit preserve unknown entries and body.
Those checks prove representability and current compatibility. They do not prove
schema enforcement, migration execution, nested YAML semantics or future API
compatibility. No new parser or dependency is selected by this design.

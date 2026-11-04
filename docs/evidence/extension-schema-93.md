# Optional extension schema design evidence

PASS: final `mise run check` completed 497 Rust tests, 42 Node checks, clean
formatting and all-target clippy with zero warnings. SKIP: one existing Windows
relative-link fixture. Additional `node --test tools/extension-schema-design.test.mjs`
passed both checks for schema example shape and exact preview/fixture hashes.

The public CLI compatibility fixture exercises both the before and after scalar
representations, raw query projection, quote/Unicode/escaped-newline decoding,
whole-repository no-write preview and a surgical title edit that leaves every
other LF fixture byte unchanged. It verifies the current CLI ignores the optional
schema file. It does not exercise a schema loader, enforcement or migration engine.

Independent critic accepted the final design-only scope, clarified acceptance
wording, scalar escape boundary and fresh passing gate before closure. The narrow
codec intersection was checked against the actual existing escape/unescape code;
unsupported escapes/controls are explicitly outside the proposed extension grammar.
Historical runtime-shaped acceptance wording is retained under a superseded heading
so checked current criteria cannot imply shipped runtime enforcement.

See [the contract](../extension-schema-design.md) and companion source hashes.
The schema is optional; values are narrow quoted strings/enums, unknown raw entries
remain user-owned, and migration requires explicit reviewed snapshots. The preview
JSON is an illustrative artifact, not current CLI input. General YAML, nested meta,
dynamic flags, a parser dependency and runtime schema commands are not delivered.
Windows execution was observed; Unix execution and non-LF preservation remain
unverified. Small contract fixtures remain tracked; disposable owned logs are
recycled after promotion. Shared target outputs and unrelated artifacts are kept.

# Explicit native write-nudge delivery — 2026-11-04

PASS: fresh `mise run check` completed 508 Rust tests, 42 Node checks,
`cargo fmt --check` without diff and all-target clippy with zero warnings.
SKIP: one existing Windows relative-link fixture. Additional
`node --test tools/native-write-nudge.test.mjs` passed all four helper cases.
`cargo package --list --allow-dirty` includes the embedded helper and command module.
Pack lint, native Antigravity strict validation and guidance coverage (45 units/
355 frozen source lines) passed. Independent critic accepted the final source
identities, boundary fixes and fresh gate before closure.

Eleven public Rust fixtures cover ordinary-init nonactivation, dry-run byte
preservation, mode conflicts, foreign JSON preservation, idempotence, check/remove,
profile independence, ownership drift/duplicates/collisions, every selected-file
preflight, input/output byte caps, exact numeric round trips or refusal, nested and
escaped duplicate-key rejection, linked parents and a linked ticket store pointing
to a second real Git repository. Direct generated Windows launch commands reach
the helper from a nested directory beneath a path containing spaces and Unicode.
The two native-profile launch shapes were executed directly, not through a harness.

Node fixtures cover path separator/dot normalization, Windows casing, unrelated/
nested paths, Add/Update/Delete/Move-to patch headers, fake headers inside hunk
content, unknown events/tools, malformed/invalid-UTF8/oversized input, line/path
caps and incomplete-stdin expiry. Output remains bounded context-only JSON without
permission decisions or event-input echo. Body/spec prose edits remain allowed.

Review found and fixed preservation boundaries before closure: ordinary JSON Value
parsing silently kept duplicate keys; float serialization could round foreign
numbers; pretty-printing could exceed the subsequent read cap; and resolving Git
from a linked `.tickets` store could select the other repository. The link case
was RED before the guard fix (incorrect exit0 installation acceptance), then GREEN
with foreign bytes unchanged. Duplicate rejection and exact decimal comparison
now happen before constructing effects; output size is checked before any selected
plan executes. Earlier passing 505/506/507-test runs predate later fixes and are
intermediate evidence only. An initial unused-import lint failure and an initial
duplicate-config regression were repaired before the final passing gate.

Serde was already a locked transitive crate. Direct use enables its official
Deserialize visitor interface for duplicate rejection; Cargo.lock changes only
the root package dependency list, with no new resolved crate/version. Global
serde_json precision behavior is unchanged. The optional helper requires Node22+;
ordinary Rust CLI commands still do not need Node.

See [setup, ownership, trust and recovery](../../skills/tkt-core/references/native-hooks.md).
Observed profiles are Codex0.166.0, Claude2.1.316 and Node22.22.3; they are not
minimum-version claims. Actual native harness dispatch/trust review, Unix launches
and exact historical minimum versions remain unverified. Project helper changes
must be inspected independently; no claim is made that native hook-definition
trust covers arbitrary external helper bytes. No native hook/user setting/project
trust/hook trust was enabled in this repository. Per-file atomic writes are not a
multi-file transaction; partial I/O or concurrent edits require explicit recovery.

Build output grew beyond500MiB because of repeated full qualification and dependency
feature compilation; one shared target tree is retained for remaining work, with
current sizes in the companion JSON. Private fixtures clean themselves up. Owned
logs are promoted here and recycled; unrelated legacy artifacts remain untouched.

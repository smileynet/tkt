# Mutation preview coverage

`tests/mutation_preview.rs` compares the current capability manifest's mutating
command set with its coverage routes. It snapshots file bytes, directories and
symlink destinations, including Git metadata and private state roots. Symlinked
large executables are referenced rather than copied or read into snapshots.

| Commands / modes | Observed controls |
|---|---|
| new, batch, claim, close, cancel, reopen, edit, note, renumber, move-ac | Successful previews, expected output and unchanged full fixture tree |
| sync-plan fix / generate, validate fix, lint | Drift/noncanonical input and intended preview output, without replacement |
| context set / clear, user/project config set / unset | Named operation preview and unchanged files |
| init all, native nudge install / check / remove, hook install / remove | Prepared absent/existing owned paths; no activation or publication during preview |
| doctor fix | Explicit existing unimplemented warning; no fabricated repair |
| rebase, prefix and local dry-run forms | Real cached upstream collision; both repositories unchanged without fetch |
| import and pending recovery | Private read-only provider probe, proposed import, pending-journal refusal and complete local snapshots |
| telemetry enable / disable / clear | Explicit private-path unit matrix in commands/telemetry.rs; never invoke CLI mutations against Windows Known Folders |

The local table has 29 cases. Four public tests pass; the ignored
`preview_hook_probe` is a subprocess entry, not an omitted behavior check. A real
private claim must execute it and write a marker before the undo-preview control;
new/claim/close previews must not create that marker. Assertions check intended
preview output as well as unchanged state. Existing specialized fixtures cover
additional invalid input, ownership, push-race, migration and recovery cases.

Each local case asserts the complete set of JSON preview operation names, including
conditional lifecycle notifications, lock release, temporary Git index/hook support
and persistent diagnostics. Import recovery plans also have private unit coverage
for exact ticket/snapshot paths. Telemetry controls assert private-path summaries
and prove that preview never invokes the platform path resolver.

The partial-clone control starts with missing tree objects: preview refuses the
cached read without fetching, while an ordinary Git read subsequently fetches
those objects. The remote allocation control advances the remote after the last
fetch and requires an explicit unresolved cached-ref notice. These are local Git
controls, not proof of a future remote outcome.

Ticket #261 routes these plans through the shared typed dispatcher while retaining
the distinct transaction owners. No rendered UI, hosted CI or real GitHub import
is exercised here. See [the execution receipt](evidence/git-effects-261.md).

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

The local table has 29 cases. Three public tests pass; the ignored
`preview_hook_probe` is a subprocess entry, not an omitted behavior check. A real
private claim must execute it and write a marker before the undo-preview control;
new/claim/close previews must not create that marker. Assertions check intended
preview output as well as unchanged state. Existing specialized fixtures cover
additional invalid input, ownership, push-race, migration and recovery cases.

This suite is a regression baseline, not evidence that the remaining #261 Git,
process and support effects already use a single dispatcher. The parent #191
remains open for that consolidation and final phase coverage. No rendered UI,
hosted CI or real GitHub import is exercised here.

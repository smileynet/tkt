# Binary-only workflow assets — design contract

Status: proposed implementation contract, accepted design scope for ticket 202. No `tkt skills` command or embedded assets ship in the current binary. This design does not change the current CLI, Cargo package or deployment engine.

## Observed gap and supported first targets

`Cargo.toml` explicitly includes source/build/readme/license files and excludes `skills/`. Repository deployment lives in `tools/deploy.mjs`, with ownership manifests and native plugin guards. It requires a checkout and JavaScript tooling. The current four standalone skills comprise 15 files including references and per-skill licenses. The companion inventory records their exact current hashes; these are input evidence, not a released bundle.

The first binary-only targets are Kiro and OpenCode. Kiro documents project `.kiro/skills/<name>/SKILL.md` and global `~/.kiro/skills/<name>/SKILL.md`; OpenCode documents project `.opencode/skills/` and global `~/.config/opencode/skills/`, plus compatibility discovery from `.claude/skills` and `.agents/skills`. Those compatibility roots matter when checking duplicates. Sources: [Kiro CLI skills](https://kiro.dev/docs/cli/skills/), [Kiro skills](https://kiro.dev/docs/skills/), [OpenCode skills](https://opencode.ai/docs/skills/). Reverify paths and any `KIRO_HOME` behavior during implementation; version-one target discovery must reject an unsupported home override rather than silently install into the wrong place.

Claude and Codex retain their native plugin installation paths; no native plugin refresh, hook installation or trust approval is simulated by this feature. Antigravity and Kiro Crew are outside the first binary-only target registry. Unsupported targets return a validation error listing supported targets. Existing `init` remains project instruction generation, separate from skill installation; exported assets contain no project-specific instructions or hooks.

## Bundle format and build boundary

Bundle format is `tkt-workflow-assets/v1`. Product version comes from Cargo's package version (currently 0.4.0); a content digest distinguishes unreleased revisions sharing that version. Identity is `(format, product_version, content_digest)`, never version alone. The content digest is SHA-256 over sorted UTF-8 relative paths, NUL, per-file SHA-256, newline. Files are exact bytes. Paths use `/`, are unique, relative, case-insensitively distinct and reject traversal, absolute names and symlinks. Never include the checkout, Git metadata, build outputs or all deploy runtime files.

Compile-time `include_bytes!` assets and a generated manifest must be explicitly listed in Cargo `include`, with a package-content test on the `.crate` archive. The asset allowlist is the four skill directories and their reference/license closure, plus a bundle manifest and top-level LICENSE/NOTICE when required by provenance. The attached source inventory contains the 15 skill files; final redistribution review determines additional legal files. Build fails on missing assets, unresolved local Markdown links, foreign workflow-skill dependencies or digest mismatch. No new runtime dependency is approved by this design; implementation must choose justified hashing/build tooling separately.

## Proposed interface (not available today)

- `tkt skills list`: bundle identity, supported targets and contents. Human default; global `-o json` uses a stable structured envelope.
- `tkt skills export --dir DIR`: write a new self-contained bundle directory. Existing unmanaged nonempty destinations conflict. Global `--dry-run` plans only, including when the parent does not exist.
- `tkt skills install --target kiro|opencode --scope project|user`: scope is required, no default user-wide installation. Project root follows the existing Git/worktree resolver. User root follows the verified target registry. No arbitrary target root in version one.
- `tkt skills remove --target ... --scope ...`: remove only verified receipt-owned paths; preserve foreign additions and report modified owned files as conflicts.

Do not use `--output DIR`: it conflicts with the global output-format flag. Commands perform no Git commit, network access, plugin refresh or telemetry-consent change. Install and remove require explicit invocations; `init` never invokes them implicitly. Errors use existing domain/operational exit taxonomy. JSON diagnostics separate stdout results and stderr warnings as current commands do.

## Ownership, duplicates and transaction semantics

Use `.tkt-workflow-assets.json` in each selected skills root, distinct from the repository deployer's receipt. Record format, product version, content digest and per-file hashes/paths. A directory name or marker alone is not ownership. Foreign same-name skills conflict even if their bytes match. A repository-deployer-owned skill also conflicts: provide migration instructions for its owner; do not take ownership or remove its receipt. Unknown receipt formats are conflicts, never partial upgrades.

Before changing files, resolve all paths and verify they remain inside the selected root. Reject symlink/reparse-point traversal at any existing component. Check all documented compatibility roots for these four skill names in the selected harness's discovery scope. Native plugin discovery that cannot be reliably inspected requires an explicit unsupported/conflict diagnostic and guidance, not a claim of duplicate safety. Do not offer a normal bypass flag. Distinct targets may read the same compatibility roots; inspect visibility, not just the chosen destination.

Serialize install, remove, rollback and journal recovery with one exclusive per-root operation lock. The lock location is derived from the canonical selected root so aliases cannot acquire separate locks. Use an OS-held exclusive lock whose ownership is released on process exit; a persistent marker alone is not proof of a live operation. Record an operation token in the journal, recover an interrupted journal only while holding the lock, and never steal a lock merely because a timestamp is old or a PID appears absent. A busy root returns a conflict without changing assets; stale marker cleanup requires acquiring the OS lock first. Missing-root creation and lock placement must share a canonical parent lock so simultaneous first installs cannot race. Implementation must select a portable lock primitive with measured Windows/Unix behavior before shipping; this design does not claim that existing atomic_write supplies locking. Dry-run performs read-only inspection, creates no lock or marker and reports a concurrent/busy operation or uncertain snapshot instead of promising an authoritative plan. Revalidate preflight under the lock immediately before applying.

Plan the whole operation before mutation. Verify old owned bytes against receipts; changed owned files and foreign additions inside a replaceable directory cause a conflict. Stage new assets on the same filesystem, validate them, then apply an on-disk journal recording old/new identities and staged paths. Per-file rename uses the existing atomic-write primitive; a multi-file install is not claimed to be one atomic filesystem operation. Write the new receipt only after asset application. A crash/retry must recover or roll back the journal before accepting another operation. Do not delete a destination before replacing it.

Keep one previous fully owned bundle beside the journal for explicit `install --rollback` (same target and scope). Rollback validates both current and backup hashes and uses the same transaction path; modified/unmanaged paths conflict. A second successful upgrade replaces only the previously verified owned backup. Journal/backup retention and removal are reported; no unbounded checkout snapshots or version directories. Removal retains modified files and returns a conflict with paths, rather than pretending a complete uninstall.

## Acceptance fixtures for implementation

`embedded-skills-202.json` records the current source asset inventory and the following required behavioral cases. Expected outcomes are design requirements, not observations of an implemented installer.

| Case | Expected result |
|---|---|
| Fresh export/install | Exact bytes and valid receipt; no other files or Git changes. |
| Identical owned install | No-op; no timestamp churn or writes. |
| Different content, same product version | Upgrade recognized by digest, not silently ignored. |
| Unmanaged same-name directory | Conflict; preserve all bytes. |
| Modified owned file / foreign addition | Conflict before any asset write. |
| Repository-deployer receipt | Conflict; no ownership takeover. |
| Duplicate in compatibility root | Conflict with both discovery paths named. |
| Unknown native plugin visibility | Explicit unsupported duplicate check; no installation. |
| Dry-run with absent root | No root, receipt, staging directory, lock, journal or Git mutation. |
| Concurrent first install / install-remove / rollback | One canonical root lock serializes operations; busy operation conflicts, no shared-journal interleaving. |
| Crash-released lock / stale marker | Reacquire OS lock before journal recovery or cleanup; never age/PID-based lock stealing. |
| Interrupted application | Retry restores coherent old/new bundle via journal; fault injection at every rename/receipt boundary. |
| Rollback | Exact previous owned bytes/identity; modified-current/backup conflicts preserved. |
| Traversal/symlink/case collision | Validation error before mutation on Windows and Unix fixtures. |
| Unsupported target / home override | Explicit error; no fallback destination. |
| Packaged binary without checkout | Export succeeds with network disabled, includes linked references and licenses. |

Implementation order: embedded allowlist/package verification; read-only list/export; target discovery and duplicate/conflict preflight; journaled install/remove/rollback; integration fixtures with isolated homes and fault injection; all CLI guidance surfaces. Each stage needs separate implementation tickets before execution. Repository deployment remains supported during that work.

## Verification and limits

Design checks: source inventory hashes, local Markdown reference closure and exact expected 15-file count; official target discovery documentation; scope review against Cargo and deployer source. Runtime export/install, package inclusion, crash recovery, OS-specific root discovery and duplicate detection remain unimplemented and unverified. The observed native checkout snapshots of roughly 4.5 GB each motivate a closed asset allowlist, not a guarantee about future native plugin installers.

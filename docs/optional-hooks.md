# Optional advisory Git hook

`tkt hooks install` explicitly installs a pre-commit advisory into this repository's
Git common hooks directory. Linked worktrees share that directory. It changes no
Git config, ticket, plan, commit or remote and is never invoked by `init` or deploy.
No hook is installed by default. This checkout's standing local verification gate
is unchanged; the hook is a convenience, not an enforcement substitute.

- `tkt hooks check` reports absent, installed, foreign or custom-hooks-path state.
  On Unix, an owned non-executable hook is reported inactive; install refuses without changing permissions, so the owner can inspect it first.
- `tkt hooks install` installs the exact managed LF shell shim, or is a no-op when
  it is already present. Git for Windows supplies the shell; Unix needs `/bin/sh`.
- `tkt hooks remove` removes only the exact current managed shim. A modified file,
  directory, symlink/reparse point or foreign hook is preserved with an error.
- `tkt hooks run` reports corpus findings as advisory and returns success for
  validation findings. Operational/config errors may still return nonzero; the
  installed shell shim catches them and always lets the Git commit continue.
- Global `--dry-run` previews install/remove without creating hook or staging
  files. `-o json` and `-q -o json` report one structured state object.

The shim invokes the `tkt` on PATH with `DO_NOT_TRACK=1` for that child so validation
does not trigger update checks or telemetry. Missing/failing tkt emits a short
advisory. Validation is synchronous and proportional to the local ticket corpus;
there is no promise of zero latency. It never runs a build or hosted workflow.
Normal tkt ticket commits keep their existing `commit.hooks=skip` default; opting
into `commit.hooks=run` is a separate explicit project choice.

Any effective `core.hooksPath` setting is preserved, including global/relative
settings: installation/removal refuses rather than silently selecting another
system. Keep an existing hook and integrate its preferred advisory command
manually. There is no force overwrite, global installer or automatic chaining.

Ownership requires exact current shim bytes, not its marker/name alone. The
installer prepares complete bytes and executable Unix permissions in a sibling
staging file, then publishes with a hard link that cannot overwrite a file created
after preflight. This requires hard-link support (verified on this Windows NTFS
workspace); unsupported filesystems return an operational error with existing
hooks untouched. Concurrent install retries accept an identical completed shim
or preserve a foreign arrival. A crash leaves either an absent or complete target;
a uniquely named `.tkt-create-*` staging orphan may remain inert and should be
inspected before manual cleanup. No backup is needed because existing hook bytes
are never replaced. Future shim-version upgrades must explicitly recognize prior
owned versions; this first installer does not take ownership of unknown versions.

Run the full verification gate independently before committing code. See
[the enforcement model](agents/enforcement-model.md) for local bypass limits and
manual-only hosted workflow policy.

Git behavior references: [hook discovery, executability and pre-commit exit status](https://git-scm.com/docs/githooks) and [canonical common-directory resolution](https://git-scm.com/docs/git-rev-parse). Windows Git execution was observed in the integration fixture; Unix mode behavior requires a Unix run.

Removal first renames the candidate to a unique sibling, then verifies the moved
bytes. A foreign replacement is restored with a no-overwrite hard link. If a new
hook already occupies the path, both versions remain and recovery locations are
reported. A moved link/directory is quarantined intact for manual recovery.
Coordinate external editors while changing hooks: an already-open editor handle
can still write afterward, and external directory replacement is outside the
installer's concurrency contract. An interrupted removal can leave an inert
`.tkt-removed-pre-commit-*` candidate; retry never deletes unknown staging files.

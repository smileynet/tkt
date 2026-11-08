# Kimi personal-plugin live-update probe — ticket 269

Date: 2026-11-08. Client: Kimi Work on macOS, current repository session. Package ID: `tkt`.

## Method and package identity

Built the maintained package with `node tools/build-kimi-plugin.mjs`, then copied `dist/kimi-plugin/` to the untracked disposable source `.scratch/kimi-plugin-live-update-20261108/`. The maintained manifest and disposable initial manifest were both version `0.4.0`. Added the HTML comment marker `tkt-live-update-probe: same-version-20261108` only to the disposable `tkt-core` skill and command reference. Native validation and the rights scan passed before each registration.

Initial SHA-256 comparisons:

| File | Maintained package | Installed managed copy |
|---|---|---|
| `skills/tkt-core/SKILL.md` | `bea4706cb5a2a1b0d5cc5d5757ead22c0d0443196de50a19079d23af51e58ce1` | `f953ffa23efe354794e599ae0fa782fb48a6e543065dd40039114494f2e5d266` |
| `skills/tkt-core/references/commands.md` | `5967cbb4a67b968440cf48d04b08b1a67a66b4ef8ba1140e16dcd4ca5aa824e6` | `74d59b28eaf31702ec75b3cb3c8d2c53677005d0aedb3325cc4ff84f862712f7` |

The marked disposable skill SHA-256 was `09e41a72080529f080c1aa45d0e86f1ad215c825bfbe6a96ec3d8fa022c0f2ff`.

## Same-version re-registration

Registered the disposable source at unchanged version `0.4.0` through `register_personal.sh`. Read-back passed for ID, version, and source path. The official response reported `sourceVersion: 0.4.0`, `installedVersion: 0.4.0`, `status: installed`, `versionUnchanged: true`, and only `removePlugin`.

The installed managed skill hash remained `f953ffa23efe354794e599ae0fa782fb48a6e543065dd40039114494f2e5d266`, not the marked disposable hash. A live `tkt:tkt-core` model invocation in the current session loaded the old managed bytes: the probe marker and the newly added edit-body guidance were absent. Thus same-version re-registration updated registry source metadata but did not replace the installed bytes or the current session's loaded skill.

## Changed-version registration

Ran the plugin builder's `cachebuster.py` on the disposable source, producing `0.4.0+local.20261108-132255`, then re-registered it. Read-back passed. The official response changed to `sourceVersion: 0.4.0+local.20261108-132255`, `installedVersion: 0.4.0`, `status: updatable`, with `updatePlugin` available. This establishes that a version change exposes the supported update action; registration alone did not claim or demonstrate installation.

The update confirmation was dismissed without an answer. No Update click, managed-copy edit, fresh chat, or post-update skill load was observed. Fresh-session timing and the exact bytes a new chat receives after Update therefore remain unverified.

## Restoration and limits

Restored personal registration to the maintained `dist/kimi-plugin/` source at `0.4.0`. Final read-back reported matching source/installed versions, `status: installed`, and no update action. The installed plugin was not knowingly changed by the probe.

Observed procedure: same-version re-registration is not an installed-content update; use a changed version and the client's Update action. Do not describe registration as instantaneous delivery to active or new sessions. A future run must click Update, compare managed bytes to the intended source, and invoke the skill in a genuinely new chat before closing the remaining acceptance criterion.

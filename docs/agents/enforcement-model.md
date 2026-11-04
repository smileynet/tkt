# Optional consumer enforcement and its limits

tkt returns findings and exit codes. A consumer may choose to run those checks in
its own CI and require a successful result through its hosting platform. Neither
installing tkt nor adding configuration installs a hosted check, branch rule or
hook. Hook installation is a separate opt-in concern; existing Git hooks can fail
when configured to run. Neither local hooks nor CI are universal enforcement layers.

## Opt-in consumer recipe

After checking out the consumer repository and installing its explicitly pinned
tkt version, add a shell step like this to its chosen pipeline:

```yaml
- name: Check ticket policy
  shell: bash
  run: |
    tkt --version
    tkt validate --strict --brief
    tkt lint --check
```

Run it from the repository root against the complete ticket corpus, without a
session tag context. The consumer owns runner setup, the pinned install method,
trigger choice and the decision to require this job. The displayed version helps
identify which CLI contract produced the result; it does not install or pin it.
These are read-only checks: `validate --strict` fails on warnings/errors and
`lint --check` fails when normalization is needed. Do not substitute mutating
`validate --fix` or bare `lint` in this gate.

If the consumer maintains `docs/plan.md`, optionally add:

```bash
tkt sync-plan --check --strict --brief
```

For another plan path, append that path explicitly. Plain `sync-plan` is advisory
and exits zero on drift; it is not an equivalent gate. A project without a plan
does not need to create one merely to use the first two checks.

The independent plan-comparison repair in ticket 248 checks all five lifecycle
labels, including plain `done`, and reports unknown cells. It preserves canonical
`✅ done` and cancellation reasons. Narrative parsing remains deferred under
ticket 184; this repair only compares existing table cells.

## What a required check can and cannot establish

The CLI can report mechanically detected findings for the checked revision. It
does not prove that evidence is truthful, an acceptance criterion meets intent,
the deployed binary is current, or the result applies to a later commit. Reviews
and project-specific tests still matter.

On GitHub, required status checks and branch protection are repository/organization
configuration. Their effect depends on the selected branches, permissions and
bypass settings; administrators and custom roles may have exceptions unless the
configured restrictions also apply to them. A workflow/configuration can be
changed by someone permitted to do so. This is optional consumer policy, not an
unbypassable property of tkt. See [GitHub protected branches](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches).

Skipping a GitHub workflow does not manufacture a successful required result:
its check may remain pending and block a merge. A manual-only workflow also does
not create a fresh passing check for every commit by itself. Configure the
consumer's triggers and required checks together. See [GitHub workflow skipping](https://docs.github.com/en/actions/how-tos/manage-workflow-runs/skip-workflow-runs).

## This repository's authorization and observed configuration

This recipe adds no workflow here and does not change the maintainer's local full
gate, hook policy, installed-binary validation distinction or release authority.
Agent-initiated hosted runs require an explicit request under the current standing
policy; optional consumer CI is not authorization to dispatch them.

This repository has no push or pull_request CI workflow; the legacy
`.github/workflows/ci.yml` was removed. The local `mise run check` gate is the
change gate ([optional local pre-push checks](../optional-pre-push.md) require explicit installation). The remaining hosted
workflows are manual-only, and no hosted workflow is dispatched without an
explicit request. Publication remains separately authorized and is limited by
the release contract.

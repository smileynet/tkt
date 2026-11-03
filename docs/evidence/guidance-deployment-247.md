# Final guidance audit and deployment

The approved stages are applied and independently reviewed. The contributor
runbook is 140 lines after the owner's later CI-policy correction. All 45 frozen
coverage units remain accounted for across 355 source-line positions, with exact
homes, reliable task triggers and no transition duplication. The JSON coverage
map records the owner amendment from commit `4b78bf4`, which removed automatic CI;
the local gate and manual hosted/publication boundaries remain explicit.

## Observed checks

- Pass: `node tools/check-guidance-coverage.mjs --final`, Node syntax, link/scope
  checks, strict ticket validation and strict plan check, all with zero findings.
  A controlled negative case rejects an unrelated ancestor as authorization for
  the CI amendment; restoring exact inventory bytes returns to pass.
- Pass: the final Rust behavior source is unchanged from `e38037b`, whose full
  `mise run check` passed 399 Rust tests, formatting and warning-denied clippy.
  After the clean integration of upstream deployer 1.8/CI changes, the affected
  Node fixtures pass 42 with one existing Windows relative-link skip, and all six
  Rust deployment-guard tests pass. These are fresh checks of the changed tools.
- Pass: dashboard/server model tests, ten passed. No rendered acceptance follows
  from them. A fresh computer-use inventory still reports local browser use
  disabled in Work settings; tickets 228, 240 and 243 remain unclosed.
- Pass: strict native manifest/inventory and directory-profile checks; current
  deployer refresh verifies Claude's active checkout and refreshes Codex, Kiro,
  OpenCode and Antigravity. The agy validator passes with its generic-icon notice.
- Pass: Claude `tkt@tkt` is enabled, reads `D:\code\tkt` directly, folder version
  0.4.0. Codex is enabled from the same local marketplace. All 24 declared runtime
  files match source in the refreshed Codex cache; all 15 skill files match in
  Codex, Kiro and OpenCode. Only deployer-owned bare Claude/Codex copies were
  removed. The [structured receipt](guidance-deployment-247.json) records identities.
- Skip: the optional Kiro Crew app was built with four skills but not activated:
  `kirocrew` is not on PATH. The deployer reports built-only; no successful app
  installation or owner trust is inferred.
- Assertion failure retained as a limit: wider `deploy.mjs audit --strict --json`
  fails for foreign/unowned roots and six legacy Kiro duplicate names, including
  49 untracked project Kiro skill directories. None is a duplicate tkt skill;
  none was pruned. This is not a globally passing inventory audit.
- CLI release installation succeeded. Its pre-integration freshness observation
  was HEAD/installed `5c2de09`; final bookkeeping requires a fresh install/check
  against the final HEAD, rather than treating that earlier metadata as current.

## Independent acceptance and retained data

The critic renewed acceptance after upstream integration, independently checking
the owner amendment, native inventories, all 24 runtime and 15-per-root skill
hashes, coverage and truthful skip/failure classifications. Accepted identities:

- Receipt: `5EC2000E79AB7B4BBFEB469DD222863FB34405E97AFF51BCE8C2527533BDC884`
- AGENTS: `00EFFDE8762F12990C0911FF7B1DE4A2EA9606458C688AE53782E959506609BF`
- README: `6F9E7406C22409E8917D9684279FB4155B41A143D60604C95C551DDE8855DBB6`
- Checker: `C168032746470CE41639DA7B722FD6D37DEF77F2AFC1C7A25C72622F45748307`
- Deployer: `1D2B7932B73773BA6EA50103ABA81F9708778329740886F22B4A5C555EB3A19D`

The structured receipt and this compact proof are promoted; disposable session
helpers, raw logs and incomplete footprint report are released to OS trash after
their useful results are recorded. The footprint helper itself reported review /
incomplete inventory due overlapping `.crush` paths and did not prove cache sizes;
the following are direct logical file-length sums, not allocated/reclaimable space:

| Retained family | Logical bytes | Lifecycle |
|---|---:|---|
| Native Codex tkt cache | 4,521,130,532 | Harness-managed while this plugin is active; replace/uninstall through its manager |
| Native Claude tkt cache | 4,506,902,767 | Harness-managed install/fallback cache; active source is the checkout |
| Shared pre-existing/mixed target cache | 4,497,706,314 | Maintainer build/install cache; not wholesale owned by this session |
| Antigravity build | 352,386 | Current installed-package payload, replaced by the next deploy |
| Kiro Crew build | 352,456 | Staged until CLI installation/activation or next build |
| Existing Node dependencies | 3,934,146 | Pinned maintainer tooling; shared lifecycle |

The native installers copied ignored build data. This measured design input was
added to still-deferred ticket 202; it does not activate that work. No versions
were bumped and no hosted workflow or release was dispatched.

Unknown-owned artifacts were preserved: `.scratch/research` 148,002 bytes,
`review` 80,544, `review-input` 85,113, `review-output` 43,080, `subagent-input`
2,553, `subagent-raw` 130,827; `v050-proposal.md` 5,129; `.references` 1,672;
`.crush` 946,727; project `.kiro/skills` 348,048. The active LAN server state is
retained until its supervised process stops. The native cache and retained-build
paths in the receipt are scoped handoff data, not substitutes for tracked proof.

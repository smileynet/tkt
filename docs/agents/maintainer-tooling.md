# Maintainer tooling traps

Read when inspecting cargo-dist, checking crate-name availability or changing
generated release workflows. [Release guidance](../releasing.md) controls
staging/publication authorization; this reference grants none. Read
[release staging](../release-staging.md) for the generated workflow contract.

Prepared under ticket 245 and activated by ticket 246. The extracted source
now lives here; AGENTS.md keeps operational rules and conditional read pointers.

<a id="rule-35"></a>

- cargo-dist binary name is `dist` (not `cargo dist`) — `cargo dist --version` will fail; use `dist --version`


<a id="rule-40"></a>

- crates.io name availability: use `cargo search <name>`, NOT raw `curl` to the API (returns 403 "violation of API data access policy" for automated requests); a 404 on the web page also confirms availability


<a id="rule-41"></a>

- Release CI is generated through `node tools/generate-release-workflow.mjs --write` using cargo-dist 0.32.0. Never hand-edit `.github/workflows/release.yml`; `mise run check:release` always checks real generation. Routine checks select it only for generator/configuration inputs; cheap committed-workflow security and seal checks always run. See `docs/release-staging.md`. Staging is build-only; the separate manual `publish.yml` owns reviewed publication. Final candidate/install/audit/staging gates remain open.

## Tooling check scope

`node tools/check-tools.mjs [--base REF] [--plan]` always selects manifest byte
preservation, ownership, credentials, publication boundaries, workflow security
and seal integrity fixtures. It selects `build-identity.test.mjs` for build identity
inputs and `release-generation.test.mjs` for cargo-dist configuration or generator
inputs. Dependency changes select both; selector and task-wiring edits retain the
cheap selector fixtures without adding native prerequisites. This keeps unrelated
tooling work independent of an installed cargo-dist or repeated identity rebuilds.

The default committed range starts at the origin/main merge-base. Tracked working
edits, staged deletions and nonignored untracked files are included. Supply
`--base REF` to check an explicit range; an invalid explicit ref fails. If the default
baseline cannot be determined, both real regressions are selected. `--plan` prints
selection only and never counts as test evidence. `--release` forces both;
`mise run check:release` adds the full Rust gate for product release verification.
The optional full pre-push hook remains opt-in and runs the full Rust gate plus
source-selected Node checks; no hook is installed by these commands.

# Release maintenance

Read the [coordinated release contract](https://github.com/smileynet/grey-matter/blob/main/docs/agents/release-contract.md)
for version authorities, modes, channels and recovery. Its
[research](https://github.com/smileynet/grey-matter/blob/main/docs/agents/research/github-release-strategy.md)
records primary GitHub/cargo-release/dist sources and non-atomic publication warnings.
Current authorization stops at reviewed, staged candidates. Publication capability
and integrated independent review are complete; final candidate, install, audit and
staging gates remain open.

## Local preparation and verification

Maintainers need Node 22+, Git, the Rust toolchain and mise. Run
`npm ci --ignore-scripts` once for the pinned private Node development dependencies.
They support release tooling; users of the installed Rust CLI do not need them.

```sh
node tools/release.mjs prepare --repo /path/to/tkt --version X.Y.Z --out /path/to/new-output
bash tools/release.sh prepare --repo /path/to/tkt --version X.Y.Z --out /path/to/new-output
mise run release -- prepare --repo /path/to/tkt --version X.Y.Z --out /path/to/new-output
```

These are alternative entry points. Default preparation is read-only. Only an approved
version/scope permits `--execute`, which makes a separate local draft branch/worktree,
notes and sealed plugin bundles. Cargo.toml owns the version; preparation derives the
root Cargo.lock and all plugin manifests from it. It creates no tag or publication.
Old patch/minor/major shell modes and automatic mise bump aliases are removed.

`verify` coordinates fresh product/native checks for one clean commit and matching
sealed artifacts through `node tools/release.mjs verify --repo DIR --sha SHA
--artifacts DIR --out NEW_DIR`. Add `--execute` for actual checks. It binds the child
reports to the same identity, rejects old output and retains all remaining release
gates. Preparation state and sealed artifacts stay unchanged.

`verify-product` forwards to `tools/release-product.mjs`; `verify-native` forwards to
`tools/release-native.mjs`. Both default to inspection, with explicit `--execute` for
actual checks. Read the [preparation guide](https://github.com/smileynet/grey-matter/blob/main/docs/agents/release-preparation.md)
for required options, output ownership, resume, native binary selection and remaining
gates. Product verification runs `mise run check`, including vendored-tool fixtures,
formatting, clippy and Rust tests. Passing partial checks never implies release readiness.

Shared tools are owned by `tools/release-vendor.json`, with MIT notices in
`tools/LICENSE-grey-matter.txt` and NOTICE. Edit canonical files in grey-matter and use
its `scripts/sync-release-tools.mjs --target /path/to/tkt --write`. That command refuses
foreign/local edits and mismatched maintainer dependencies. A tkt-only checkout runs
its own copies; it does not need grey-matter installed. `mise run release:tools` checks
the local hash inventory, entry points, dependency pins and read-only preparation.

CI installs Node 22 and the locked maintainer packages before the same gate, following
[GitHub's Node workflow guidance](https://docs.github.com/en/actions/tutorials/build-and-test-code/nodejs)
and [setup-node v7](https://github.com/actions/setup-node/tree/v7).
The action requires runner 2.327.1+; this job uses GitHub-hosted Ubuntu. Its v7 token
fallback change is irrelevant here: no registry-url, npm authentication or publication
is configured, and package caching is explicitly disabled. These sources were read
2026-10-23; the hosted run still needs to pass on the pushed commit.

## Staging and later publication

All open tickets ship or receive explicit user deferral before a release. Record the
approved version/scope, exact commit, notes, assets/hashes, platform gaps and tested
compatibility. grey-matter's [release skill](https://github.com/smileynet/grey-matter/blob/main/skills/meta/release/SKILL.md)
orchestrates candidate, install, audit and staging gates for both products. Include the [v0.4 skill-link migration](../README.md#upgrading-skills-from-v040)
in previous-release upgrade probes; those releases had no plugin packages. No preparation command publishes a crate, GitHub release,
tag or stable ref. The candidate index needs review before separate publication.

The generated `.github/workflows/release.yml` is build-only; configure those jobs
through the checked dist generator. The separate manual `.github/workflows/publish.yml`
is the sole publisher. It checks out exact candidate C and review R separately,
uses Cargo/rustc 1.99.1 and the verified Linux namespace runtime, and calls
`tools/publish-reviewed.mjs`. Its default is read-only planning. A fixed concurrency
group serializes publication across versions, without canceling an active publisher.

The review index directory contains `tkt/artifacts/` and `tkt/evidence/`. Commit the
exact plugin bundles, metadata and verification logs there on the review branch;
CI downloads the separate reviewed dist/crate archive using its bound Actions
identity. These are reviewed bytes, not outputs rebuilt during publication.

Local `tools/dispatch-publication.mjs` accepts `plan`, `dispatch` and `observe`,
with `--review-repo DIR --review-sha R --index FILE --state EXTERNAL_FILE`.
Without `--execute`, dispatch only plans. Authorized execution verifies the remote
candidate branch and workflow, saves an external request receipt before POST,
and dispatches CI. Repeating the command observes that request; it never sends it
again automatically after timeout or runner loss. `observe` makes no remote writes.
A completed run is not itself certification of published channels: read CI's
per-channel result and reconcile the actual release, registry and stable state.

Production execution requires separately authorized immutable-release enablement
and a `CARGO_REGISTRY_TOKEN` repository secret. The secret is exposed only to the
publisher step and read by its delayed credential callback. Preparation subprocesses
receive sanitized environments. Do not configure settings/secrets or execute
publication as part of staging. Ticket 68 accepted scoped Linux parent-loss and
product-gate evidence and integrated Standards/Spec review. The product probe used
0.4.0 source, not the final 0.5.0 candidate; no production publication has run.

The shared publication-toolchain action supplies Node 22, Cargo/Rust 1.99.1 with
rustfmt/clippy, mise 2026.10.18, cargo-dist 0.32.0 and bubblewrap. The read-only
`publication-product-probe.yml` uses that same setup and runs the actual development
product gate. Both use the exact owned clone's mise trust path. Stable-only recovery
re-observes matching immutable GitHub and registry publication, then skips development
and candidate ref updates. Failure JSON retains C/R and last observed channel
statuses/URLs, with stable unconfirmed until its result is verified.

Local maintenance uses `node tools/deploy.mjs` once after skill/guidance/plugin edits.
The bash deploy wrapper is an alternative. Install the CLI with `cargo install --path .`
when Rust changed. Check deployment/audit and start a new chat after plugin refresh.
Local deployment does not publish a product release.

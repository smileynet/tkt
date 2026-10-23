# Release maintenance

Read the [coordinated release contract](https://github.com/smileynet/grey-matter/blob/main/docs/agents/release-contract.md)
for version authorities, modes, channels and recovery. Its
[research](https://github.com/smileynet/grey-matter/blob/main/docs/agents/research/github-release-strategy.md)
records primary GitHub/cargo-release/dist sources and non-atomic publication warnings.
Current work stops at reviewed, staged candidates; publication/resume is unfinished.

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
compatibility. Tickets 67–69 in grey-matter retain integration/candidate tests; 70–71
own final audit/staging. No preparation command publishes a crate, GitHub release,
tag or stable ref. The candidate index needs review before separate publication.

Inspect release.toml, Cargo.toml workspace.metadata.dist and the generated
.github/workflows/release.yml when implementing publication. Configure generated jobs
through dist's generator. That workflow remains the sole GitHub publisher; local
tooling must dispatch/observe its identity without creating a competing release.

Local maintenance uses `node tools/deploy.mjs` once after skill/guidance/plugin edits.
The bash deploy wrapper is an alternative. Install the CLI with `cargo install --path .`
when Rust changed. Check deployment/audit and start a new chat after plugin refresh.
Local deployment does not publish a product release.

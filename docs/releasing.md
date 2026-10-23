# Release maintenance

Read the [coordinated release contract](https://github.com/smileynet/grey-matter/blob/main/docs/agents/release-contract.md)
when preparing, verifying, staging, publishing or resuming a tkt release. It is the
shared authority for modes, versioning, channel promotion and recovery. Its
[research](https://github.com/smileynet/grey-matter/blob/main/docs/agents/research/github-release-strategy.md)
records primary GitHub/cargo-release/dist sources and non-atomic publication warnings.

Cargo.toml owns tkt's product version. Inspect release.toml for manifest replacements,
tests/plugin_manifests.rs for consistency, tools/release.sh for local mechanics and
Cargo.toml workspace.metadata.dist plus generated .github/workflows/release.yml for CI ownership.
The release includes skills even when CLI behavior is unchanged.

The existing tools/release.sh calls cargo-release and does not yet implement the
coordinated preparation/resume gates. Tickets 67–69 in grey-matter provide those
mechanics and candidate tests. Until verified, use inspection/planning only; do not
execute the legacy publication path for staged-candidate work. Keep dist's generated
workflow configured through its generator. It is the sole GitHub publisher;
local tooling prepares/dispatches the identity and observes CI.

All open tickets ship or receive explicit user deferral before a release. Record the
approved version, included scope, exact commit, notes, assets/hashes and tested
compatibility. Preparation/staging publishes no tag, crate, GitHub release or stable
ref. The final candidate index must be reviewed before any separate publication.

Local maintenance uses `node tools/deploy.mjs` once after skill/guidance/plugin edits.
The bash wrapper is an alternative, not an extra deploy. Install the CLI with
`cargo install --path .` when Rust changed. Run the deployment check/audit and start
a new chat to load refreshed skills. This does not publish a product release.

# Stage release binaries and crate

The manual **Stage release artifacts** workflow builds all configured cargo-dist
targets, installers, source archive and the clean Cargo crate. It seals their
filenames, sizes and SHA-256 hashes in `tkt-stage.json`. Staging creates Actions
artifacts; it does not publish a crate, GitHub release, tag or stable branch.

Use Node 22+, locked npm dependencies and cargo-dist 0.32.0 for maintenance:

```sh
npm ci --ignore-scripts
cargo install cargo-dist --version 0.32.0 --locked
node tools/generate-release-workflow.mjs
mise run check
```

Edit the generator wrapper or Cargo configuration, then regenerate with
`node tools/generate-release-workflow.mjs --write`. Never hand-edit
`.github/workflows/release.yml`. The gate runs the real pinned generator twice,
checks deterministic output and tests candidate/file identity failures. CI installs
the pinned generator before running the same gate.

Push the reviewed candidate branch, then dispatch its exact commit:

```sh
gh workflow run release.yml --repo smileynet/tkt --ref CANDIDATE_BRANCH \
  -f candidate_sha=FULL_CANDIDATE_SHA
```

The workflow must already be registered on the default branch. Every job refuses to build
unless the expected SHA, run SHA and checked-out SHA agree. Preserve the completed
run ID, attempt, artifact IDs/digests/expiration and the seal hash for later review.
The final artifact is named `tkt-sealed-stage-RUN_ID-ATTEMPT`. Intermediate
uploads/downloads also include the attempt. Use **Re-run all jobs** after a failure;
rerunning only failed jobs cannot substitute inputs from an older attempt.

The separate manual publisher now reads the later review commit, verifies run
provenance and downloads those sealed files. It must not rebuild or replace expired
artifacts: stage again and repeat review after retention expires. The prepared Cargo
transport and durable intent barrier are implemented and mock/native-tested. Ticket
68 accepted scoped Linux parent-loss/product evidence and integrated independent
review. Fresh final-candidate, install/upgrade, platform and audit gates remain;
source-version probes do not validate the approved 0.5.0 candidate.
See [release maintenance](releasing.md) for the manual workflow, staging carrier and
local dispatch/observation boundaries. Staging itself still performs no publication.

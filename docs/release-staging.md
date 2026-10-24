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
The final artifact is named `tkt-sealed-stage-RUN_ID-ATTEMPT`.

Publication integration remains unfinished. A future publisher must read the later
review commit, verify run provenance and every sealed file, and download those exact
artifacts. It must not rebuild or silently replace expired artifacts. Stage again
and repeat review if retention expires. Crate checksums alone do not establish that
a future Cargo publish will upload the sealed bytes; that transport proof and a
durable upload-attempt barrier remain release blockers.

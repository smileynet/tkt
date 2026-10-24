#!/usr/bin/env node
// Bind the sole tkt CI publisher to reviewed files and its exact dispatch identity.
// Usage: publishTktCi(options, services); CLI/workflow integration owns credential setup.
import { realpathSync } from "node:fs";
import { resolve } from "node:path";
import { git, requireGithubRemote, requireCandidateTag, pushCandidateRefs } from "./release-git.mjs";
import { ReleaseError } from "./release-version.mjs";
import { readStagedTktCandidate } from "./release-staging.mjs";
import { githubActionsReadApi } from "./release-ci-artifacts.mjs";
import { githubReleaseApi } from "./release-github.mjs";
import { cratesIoTkt } from "./release-registry.mjs";
import { prepareCargoUpload } from "./release-cargo-upload.mjs";
import { prepareDevelopment } from "./release-development.mjs";
import { promotePublishedStable } from "./release-publish.mjs";
import { reconcileTktPublication } from "./release-tkt-publication.mjs";

export async function requireTktPublisherRun({ candidate, context, actions }) {
  const spec = candidate.staging.publication;
  if (!spec || context?.event !== "workflow_dispatch" || context.repository !== candidate.repository || context.sha !== spec.workflowSha || context.ref !== spec.branch || context.workflowRef !== `${candidate.repository}/.github/workflows/publish.yml@${spec.branch}` || !Number.isSafeInteger(context.runId) || context.runId <= 0 || !Number.isSafeInteger(context.attempt) || context.attempt <= 0) throw new ReleaseError("tkt execution requires the exact reviewed CI dispatch");
  const run = await actions.run(candidate.repository, context.runId);
  if (run?.id !== context.runId || run.run_attempt !== context.attempt || run.head_sha !== spec.workflowSha || run.event !== "workflow_dispatch" || run.workflow_id !== spec.workflowId || run.path !== ".github/workflows/publish.yml" || run.status !== "in_progress" || run.repository?.full_name !== candidate.repository || run.repository.id !== candidate.staging.ci.repositoryId || run.head_repository?.id !== run.repository.id) throw new ReleaseError("publisher run is stale, foreign or differs from reviewed workflow identity");
}

export async function publishTktCi(options, {
  actions = githubActionsReadApi(), api, registry = cratesIoTkt(),
  prepareMain = prepareDevelopment, pushCandidate = pushCandidateRefs,
  prepareUpload = prepareCargoUpload, promoteStable = promotePublishedStable,
} = {}) {
  const execute = options.execute ?? false, retryUnconfirmed = options.retryUnconfirmed ?? false;
  if (typeof execute !== "boolean" || typeof retryUnconfirmed !== "boolean" || (execute && options.dryRun)) throw new ReleaseError("invalid CI publication/recovery mode");
  const candidate = await readStagedTktCandidate({ reviewRepo: options.reviewRepo, reviewSha: options.reviewSha, indexFile: options.index, artifacts: options.artifacts, evidence: options.evidence, api: actions });
  const spec = candidate.staging.publication;
  if (!spec) throw new ReleaseError("tkt staging lacks the reviewed publication binding");
  const repo = realpathSync.native(git(resolve(options.repo), ["rev-parse", "--show-toplevel"]).trim()), remote = options.remote ?? "origin";
  const check = () => {
    if (git(repo, ["rev-parse", "HEAD"]).trim() !== candidate.candidateSha || git(repo, ["status", "--porcelain=v1", "-z"])) throw new ReleaseError("CI publisher requires a clean exact candidate checkout");
    requireGithubRemote({ repo, repository: candidate.repository, remote });
  };
  check();
  const sealed = candidate.assets.filter((asset) => asset.name === `tkt-${candidate.version}.crate`);
  if (sealed.length !== 1) throw new ReleaseError("reviewed CI assets must contain exactly one tkt crate");
  const crate = { name: "tkt", version: candidate.version, bytes: sealed[0].bytes, sha256: sealed[0].sha256 };
  if (!execute) return { status: "pass", mode: "dry-run", publisher: "cargo-dist", candidateSha: candidate.candidateSha, reviewSha: options.reviewSha, ciStage: candidate.ciStage, assetNames: candidate.assets.map((asset) => asset.name), writes: false, plan: ["verify exact CI dispatch and immutable policy", "check development integration", "atomic candidate/tag/main refs", "prepare Cargo before durable intent", "reconcile registry and immutable GitHub", "verify stable"] };
  await requireTktPublisherRun({ candidate, context: options.context, actions });
  api ??= githubReleaseApi({ execute: true });
  if (await api.immutableEnabled(candidate.repository) !== true) throw new ReleaseError("repository release immutability must be enabled before pushing candidate refs");
  if (typeof options.developmentOut !== "string" || typeof options.getToken !== "function") throw new ReleaseError("CI publisher requires an owned development output and delayed credential callback");
  const development = await prepareMain({ repo, remote, candidateSha: candidate.candidateSha, product: "tkt", version: candidate.version, out: options.developmentOut, execute: true });
  if (development?.status !== "pass" || development.candidateSha !== candidate.candidateSha || !/^[a-f0-9]{40}$/.test(development.developmentSha)) throw new ReleaseError("development integration was not verified");
  check();
  const tagObject = git(repo, ["hash-object", "-t", "tag", "-w", "--stdin"], { input: spec.tagRecord }).trim();
  if (tagObject !== spec.tagObject) throw new ReleaseError("imported annotated tag differs from review");
  const full = { ...candidate, tagObject };
  const tag = `refs/tags/v${candidate.version}`;
  requireCandidateTag({ repo, candidateSha: candidate.candidateSha, tagObject, tag });
  await requireTktPublisherRun({ candidate, context: options.context, actions });
  const refs = await pushCandidate({ repo, remote, branch: spec.branch, tag, tagObject, candidateSha: candidate.candidateSha, developmentSha: development.developmentSha, execute: true });
  if (refs?.status !== "pass" || refs.complete !== true) throw new ReleaseError("atomic candidate refs were not observed");
  check();
  const result = await reconcileTktPublication({ candidate: full, crate, api, registry, execute: true, retryUnconfirmed, observe: options.observe,
    prepareUpload: (sealedCrate) => prepareUpload({ repo, candidateSha: candidate.candidateSha, crate: sealedCrate, cargo: options.cargo, rustc: options.rustc, getToken: options.getToken }),
    promoteStable: (published) => promoteStable({ repo, remote, candidate: published, api, publisher: "cargo-dist", execute: true }),
  });
  return { ...result, candidateSha: candidate.candidateSha, reviewSha: options.reviewSha, development, ciStage: candidate.ciStage, directorySubmission: "not-requested", marketplaceUrl: `https://github.com/${candidate.repository}/tree/stable` };
}

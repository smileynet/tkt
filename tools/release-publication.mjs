#!/usr/bin/env node
// Publish/resume a reviewed grey-matter staging identity; dry-run by default.
// Usage: node scripts/release-publication.mjs publish|resume --repo DIR --review-repo DIR
//   --review-sha SHA --index FILE --product grey-matter --artifacts DIR --evidence DIR
//   --tag-object SHA --state FILE [--remote origin] [--execute | --dry-run]
import { closeSync, existsSync, mkdirSync, openSync, readFileSync, realpathSync, renameSync, unlinkSync, writeFileSync } from "node:fs";
import { createHash, randomUUID } from "node:crypto";
import { basename, dirname, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { git, requireGithubRemote, requireCandidateTag, pushCandidateRefs } from "./release-git.mjs";
import { ReleaseError } from "./release-version.mjs";
import { readStagedCandidate } from "./release-staging.mjs";
import { githubReleaseApi } from "./release-github.mjs";
import { prepareDevelopment } from "./release-development.mjs";
import { reconcileGithubRelease, promotePublishedStable } from "./release-publish.mjs";
const encode = (value) => JSON.stringify(value, null, 2) + "\n";
const hash = (value) => createHash("sha256").update(value).digest("hex");
function canonical(path) {
  if (existsSync(path)) return realpathSync.native(path);
  return resolve(canonical(dirname(path)), basename(path));
}
export async function publishStaged(options, { api, prepareMain = prepareDevelopment, pushCandidate = pushCandidateRefs, promoteStable = promotePublishedStable } = {}) {
  const execute = options.execute === undefined ? false : options.execute;
  if (typeof execute !== "boolean" || (execute && options.dryRun) || !["publish", "resume"].includes(options.mode)) throw new ReleaseError("invalid publication mode");
  if (options.product === "tkt") throw new ReleaseError("tkt publication belongs to cargo-dist CI; its integration is not enabled here");
  const repo = realpathSync.native(git(resolve(options.repo), ["rev-parse", "--show-toplevel"]).trim()), remote = options.remote ?? "origin";
  const load = () => readStagedCandidate({ reviewRepo: options.reviewRepo, reviewSha: options.reviewSha, indexFile: options.index, product: options.product, artifacts: options.artifacts, evidence: options.evidence });
  const candidate = { ...load(), tagObject: options.tagObject };
  if (candidate.product !== "grey-matter") throw new ReleaseError("tkt publication belongs to cargo-dist CI; its integration is not enabled here");
  const branch = git(repo, ["symbolic-ref", "HEAD"]).trim(), tag = `refs/tags/v${candidate.version}`;
  if (!branch.startsWith(`refs/heads/codex/release-${candidate.product}-${candidate.version}-`)) throw new ReleaseError("publication requires the dedicated candidate branch");
  function check() {
    load();
    if (git(repo, ["rev-parse", "HEAD"]).trim() !== candidate.candidateSha || git(repo, ["status", "--porcelain=v1", "-z"])) throw new ReleaseError("publication candidate must remain clean at the reviewed commit");
    requireGithubRemote({ repo, repository: candidate.repository, remote });
    requireCandidateTag({ repo, tag, candidateSha: candidate.candidateSha, tagObject: options.tagObject });
  }
  check();
  const statePath = canonical(resolve(options.state));
  if (statePath === repo || statePath.startsWith(repo + sep)) throw new ReleaseError("publication recovery state must be outside the candidate checkout");
  const identity = hash(encode({ repository: candidate.repository, product: candidate.product, version: candidate.version, candidateSha: candidate.candidateSha, staging: candidate.staging, branch, remote, tagObject: options.tagObject, assets: candidate.assets.map(({ name, sha256 }) => ({ name, sha256 })), notesSha256: hash(candidate.notes) }));
  function readRecovery() {
    let state;
    if (existsSync(statePath)) {
      try { state = JSON.parse(readFileSync(statePath, "utf8")); }
      catch (error) { if (error instanceof SyntaxError) throw new ReleaseError("invalid publication recovery state"); throw error; }
      if (state?.schema !== 1 || state.owner !== "grey-matter-publication" || state.identity !== identity || (state.releaseId !== null && (!Number.isSafeInteger(state.releaseId) || state.releaseId <= 0))) throw new ReleaseError("foreign recovery state or different reviewed publication identity");
    } else {
      if (options.mode === "resume") throw new ReleaseError("resume requires the existing publication recovery state");
      state = { schema: 1, owner: "grey-matter-publication", identity, candidateSha: candidate.candidateSha, releaseId: null, phase: "planned" };
    }
    return state;
  }
  let state = readRecovery();
  if (!execute) return { status: "pass", mode: "dry-run", candidateSha: candidate.candidateSha, reviewSha: options.reviewSha, publisher: "local", previousPhase: state.phase, assetNames: candidate.assets.map((asset) => asset.name), plan: ["prepare and fully check development integration", "atomic development/candidate/tag refs", "reconcile immutable GitHub release", "verify and promote stable"], writes: false };
  const save = (fields) => {
    Object.assign(state, fields);
    if (fields.phase && fields.phase !== "interrupted") delete state.error;
    if (fields.phase && fields.phase !== "complete") delete state.result;
    mkdirSync(dirname(statePath), { recursive: true });
    const temp = `${statePath}.${randomUUID()}.tmp`;
    writeFileSync(temp, encode(state), { flag: "wx", mode: 0o600 }); renameSync(temp, statePath);
  };
  api ??= githubReleaseApi({ execute: true });
  mkdirSync(dirname(statePath), { recursive: true });
  const lockPath = `${statePath}.lock`, lockReceipt = encode({ owner: "grey-matter-publication", identity, pid: process.pid, token: randomUUID() });
  let lock, ownsState = false;
  try { lock = openSync(lockPath, "wx", 0o600); }
  catch (error) { if (error.code === "EEXIST") throw new ReleaseError("publication recovery state is locked; inspect its owner before resuming"); throw error; }
  try {
    writeFileSync(lock, lockReceipt); closeSync(lock); lock = undefined;
    state = readRecovery(); ownsState = true;
    save({ phase: "candidate-refs" });
    if (await api.immutableEnabled(candidate.repository) !== true) throw new ReleaseError("repository release immutability must be enabled before pushing candidate refs");
    check();
    save({ phase: "development" });
    const development = await prepareMain({ repo, remote, candidateSha: candidate.candidateSha, product: candidate.product, version: candidate.version, out: `${statePath}.development`, execute: true });
    if (development?.status !== "pass" || development.candidateSha !== candidate.candidateSha || !/^[a-f0-9]{40}$/.test(development.developmentSha)) throw new ReleaseError("development integration was not verified");
    save({ phase: "candidate-refs", development }); check();
    const refs = pushCandidate({ repo, remote, branch, tag, candidateSha: candidate.candidateSha, tagObject: options.tagObject, developmentSha: development.developmentSha, execute: true });
    if (!refs.complete) throw new ReleaseError("candidate ref publication was not observed");
    save({ phase: "github" }); check();
    candidate.releaseId = state.releaseId ?? undefined;
    const published = await reconcileGithubRelease({ candidate, api, publisher: "local", execute: true, observe: (snapshot) => save({ github: snapshot, releaseId: snapshot.releaseId }) });
    save({ phase: "stable", releaseId: published.releaseId }); check();
    const stable = await promoteStable({ repo, remote, candidate: { ...candidate, releaseId: state.releaseId }, api, publisher: "local", execute: true });
    if (!stable.complete) throw new ReleaseError("stable promotion was not observed");
    const result = { status: "pass", mode: "published", candidateSha: candidate.candidateSha, reviewSha: options.reviewSha, development, github: published, stable, registry: "not-applicable", directorySubmission: "not-requested", marketplaceUrl: `https://github.com/${candidate.repository}/tree/stable` };
    save({ phase: "complete", result }); return result;
  } catch (error) {
    if (ownsState) save({ phase: "interrupted", error: error.message });
    throw error;
  } finally {
    if (lock !== undefined) closeSync(lock);
    if (existsSync(lockPath) && readFileSync(lockPath, "utf8") === lockReceipt) unlinkSync(lockPath);
  }
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const args = process.argv.slice(2), options = { mode: args.shift() };
    while (args.length) {
      const key = args.shift();
      if (key === "--execute") options.execute = true;
      else if (key === "--dry-run") options.dryRun = true;
      else if (["--repo", "--review-repo", "--review-sha", "--index", "--product", "--artifacts", "--evidence", "--tag-object", "--state", "--remote"].includes(key) && args.length) options[key.slice(2).replace(/-([a-z])/g, (_, c) => c.toUpperCase())] = args.shift();
      else throw new ReleaseError(`unknown/incomplete option: ${key}`);
    }
    if (["repo", "reviewRepo", "reviewSha", "index", "product", "artifacts", "evidence", "tagObject", "state"].some((key) => !options[key])) throw new ReleaseError("usage: publish|resume --repo DIR --review-repo DIR --review-sha SHA --index FILE --product grey-matter --artifacts DIR --evidence DIR --tag-object SHA --state FILE [--execute | --dry-run]");
    console.log(encode(await publishStaged(options)));
  } catch (error) { console.error(encode({ status: error instanceof ReleaseError ? "fail" : "error", errors: [error.message] })); process.exitCode = error instanceof ReleaseError ? 1 : 2; }
}

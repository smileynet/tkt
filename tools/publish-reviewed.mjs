#!/usr/bin/env node
// Execute the sole CI publisher against a separate, pinned staging review checkout.
// Usage: node tools/publish-reviewed.mjs --repo DIR --review-repo DIR --review-sha SHA
//   --index FILE --development-out DIR --cargo FILE --rustc FILE [--execute] [--retry-unconfirmed]
import { realpathSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { git, requireGithubRemote } from "./release-git.mjs";
import { ReleaseError } from "./release-version.mjs";
import { publishTktCi } from "./release-tkt-ci.mjs";
import { publicationFailure } from "./release-publish.mjs";

export function stagedLocations({ reviewRepo, index }) {
  if (typeof index !== "string" || !/^[A-Za-z0-9][A-Za-z0-9._/-]*$/.test(index) || index.split("/").some((part) => !part || part === "." || part === "..")) throw new ReleaseError("invalid staged index path");
  const base = resolve(reviewRepo, dirname(index));
  return { artifacts: resolve(base, "tkt/artifacts"), evidence: resolve(base, "tkt/evidence") };
}
export function publisherOptions(args, env = process.env) {
  const options = {};
  while (args.length) {
    const key = args.shift();
    if (["--execute", "--dry-run", "--retry-unconfirmed"].includes(key)) options[key.slice(2).replace(/-([a-z])/g, (_, c) => c.toUpperCase())] = true;
    else if (["--repo", "--review-repo", "--review-sha", "--index", "--development-out", "--cargo", "--rustc"].includes(key) && args.length) {
      const name = key.slice(2).replace(/-([a-z])/g, (_, c) => c.toUpperCase());
      if (options[name] !== undefined) throw new ReleaseError(`duplicate option: ${key}`);
      options[name] = args.shift();
    } else throw new ReleaseError(`unknown/incomplete publication option: ${key}`);
  }
  if (["repo", "reviewRepo", "reviewSha", "index", "developmentOut", "cargo", "rustc"].some((key) => !options[key]) || !/^[a-f0-9]{40}$/.test(options.reviewSha) || !/^[A-Za-z0-9][A-Za-z0-9._/-]*$/.test(options.index) || options.index.split("/").some((part) => !part || part === "." || part === "..") || (options.execute && options.dryRun)) throw new ReleaseError("invalid/incomplete publication arguments");
  Object.assign(options, stagedLocations(options));
  options.context = { event: env.GITHUB_EVENT_NAME, repository: env.GITHUB_REPOSITORY, sha: env.GITHUB_SHA, ref: env.GITHUB_REF, workflowRef: env.GITHUB_WORKFLOW_REF, runId: Number(env.GITHUB_RUN_ID), attempt: Number(env.GITHUB_RUN_ATTEMPT) };
  options.getToken = () => {
    const token = env.RELEASE_CARGO_TOKEN;
    if (!token) throw new ReleaseError("release credential is unavailable");
    return token;
  };
  return options;
}
export async function runPublisher(options, services) {
  const review = realpathSync.native(options.reviewRepo), repo = realpathSync.native(options.repo);
  if (review === repo || git(review, ["rev-parse", "HEAD"]).trim() !== options.reviewSha || git(review, ["status", "--porcelain=v1", "-z"])) throw new ReleaseError("review must be a separate clean checkout at the pinned review commit");
  requireGithubRemote({ repo: review, repository: "smileynet/grey-matter" });
  return publishTktCi(options, services);
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try { console.log(JSON.stringify(await runPublisher(publisherOptions(process.argv.slice(2))), null, 2)); }
  catch (error) { console.log(JSON.stringify(publicationFailure(error))); process.exitCode = error instanceof ReleaseError ? 1 : 2; }
}

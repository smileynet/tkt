#!/usr/bin/env node
// Dispatch or observe the sole CI publisher; never publish a local tkt release.
// Usage: node tools/dispatch-publication.mjs plan|dispatch|observe --review-repo DIR
//   --review-sha SHA --index FILE --state FILE [--execute] [--retry-unconfirmed]
import { closeSync, existsSync, mkdirSync, openSync, readFileSync, realpathSync, renameSync, unlinkSync, writeFileSync } from "node:fs";
import { createHash, randomUUID } from "node:crypto";
import { basename, dirname, resolve, sep } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { readStagedTktCandidate } from "./release-staging.mjs";
import { requireGithubRemote } from "./release-git.mjs";
import { ReleaseError } from "./release-version.mjs";
import { githubActionsReadApi } from "./release-ci-artifacts.mjs";
import { stagedLocations } from "./publish-reviewed.mjs";
const encode = (value) => JSON.stringify(value, null, 2) + "\n";
function canonical(path) { return existsSync(path) ? realpathSync.native(path) : resolve(canonical(dirname(path)), basename(path)); }
export function publicationDispatchApi({ run = (args, input) => spawnSync("gh", args, { input, encoding: "utf8", maxBuffer: 32 * 1024 * 1024, env: { ...process.env, GH_HOST: "github.com", GH_PROMPT_DISABLED: "1" } }) } = {}) {
  function request(repo, workflow, suffix = "", body) {
    if (repo !== "smileynet/tkt" || !Number.isSafeInteger(workflow) || workflow <= 0) throw new ReleaseError("invalid tkt dispatch target");
    const args = ["api", "--hostname", "github.com", "--method", body ? "POST" : "GET", "-H", "X-GitHub-Api-Version: 2026-03-10"];
    if (suffix.startsWith("/runs")) args.push("--paginate", "--slurp");
    if (body) args.push("-H", "Content-Type: application/json", "--input", "-");
    args.push(`repos/${repo}/actions/workflows/${workflow}${suffix}`);
    const result = run(args, body ? encode(body) : undefined);
    if (result.error || result.status !== 0) throw new ReleaseError("dispatch API outcome is uncertain; observe the saved request before retrying");
    let value;
    try { value = JSON.parse(result.stdout); } catch { throw new ReleaseError("invalid dispatch API response; observe the saved request"); }
    if (!suffix.startsWith("/runs")) return value;
    if (!Array.isArray(value) || value.some((page) => !Array.isArray(page?.workflow_runs))) throw new ReleaseError("invalid paginated dispatch runs");
    return value.flatMap((page) => page.workflow_runs);
  }
  return {
    workflow: (repo, id) => request(repo, id),
    ref: (repo, branch) => {
      if (repo !== "smileynet/tkt" || typeof branch !== "string" || !branch.startsWith("codex/release-tkt-")) throw new ReleaseError("invalid dispatch branch");
      const result = run(["api", "--hostname", "github.com", "--method", "GET", "-H", "X-GitHub-Api-Version: 2026-03-10", `repos/${repo}/git/ref/heads/${encodeURIComponent(branch)}`]);
      if (result.error || result.status !== 0) throw new ReleaseError("candidate branch could not be verified before dispatch");
      try { return JSON.parse(result.stdout); } catch { throw new ReleaseError("invalid candidate branch response"); }
    },
    dispatch: (repo, id, body) => request(repo, id, "/dispatches", body),
    runs: (repo, id, branch) => request(repo, id, `/runs?event=workflow_dispatch&branch=${encodeURIComponent(branch)}&per_page=100`),
  };
}
export async function dispatchPublication(options, { api = publicationDispatchApi(), actions = githubActionsReadApi(), readCandidate = readStagedTktCandidate, checkReview = requireGithubRemote } = {}) {
  if (!["plan", "dispatch", "observe"].includes(options.mode) || (options.execute !== undefined && typeof options.execute !== "boolean") || (options.retryUnconfirmed !== undefined && typeof options.retryUnconfirmed !== "boolean") || (options.mode !== "dispatch" && options.execute)) throw new ReleaseError("invalid dispatch mode");
  const locations = stagedLocations(options);
  checkReview({ repo: options.reviewRepo, repository: "smileynet/grey-matter" });
  const candidate = await readCandidate({ reviewRepo: options.reviewRepo, reviewSha: options.reviewSha, indexFile: options.index, ...locations, api: actions });
  const spec = candidate.staging.publication;
  if (candidate.repository !== "smileynet/tkt" || !spec) throw new ReleaseError("reviewed tkt publication binding is required");
  const workflow = await api.workflow(candidate.repository, spec.workflowId);
  if (workflow?.id !== spec.workflowId || workflow.path !== ".github/workflows/publish.yml" || workflow.state !== "active") throw new ReleaseError("reviewed publisher workflow is unavailable or changed");
  if (options.mode === "plan" || (options.mode === "dispatch" && !options.execute)) return { status: "pass", mode: "dry-run", publisher: "cargo-dist", writes: false, candidateSha: candidate.candidateSha, reviewSha: options.reviewSha, workflowId: spec.workflowId, ref: spec.branch };
  if (typeof options.state !== "string" || !options.state) throw new ReleaseError("dispatch/observe requires an external state file");
  const statePath = canonical(resolve(options.state)), review = realpathSync.native(options.reviewRepo);
  if (statePath === review || statePath.startsWith(review + sep)) throw new ReleaseError("dispatch state must be outside the review checkout");
  const retry = options.retryUnconfirmed ?? false;
  const identity = createHash("sha256").update(encode({ repository: candidate.repository, reviewSha: options.reviewSha, index: options.index, candidateSha: candidate.candidateSha, spec, retry })).digest("hex");
  let state;
  if (existsSync(statePath)) {
    try { state = JSON.parse(readFileSync(statePath, "utf8")); } catch { throw new ReleaseError("invalid dispatch state"); }
    if (state?.schema !== 1 || state.owner !== "tkt-publication-dispatch" || state.identity !== identity || !/^[a-f0-9]{32}$/.test(state.requestId) || typeof state.attempted !== "boolean" || (state.runId !== null && (!Number.isSafeInteger(state.runId) || state.runId <= 0))) throw new ReleaseError("foreign state or changed reviewed dispatch identity");
  } else {
    if (options.mode === "observe") throw new ReleaseError("observe requires saved dispatch state");
    state = { schema: 1, owner: "tkt-publication-dispatch", identity, requestId: randomUUID().replaceAll("-", ""), attempted: false, runId: null };
  }
  const save = () => { const temp = `${statePath}.${randomUUID()}.tmp`; writeFileSync(temp, encode(state), { flag: "wx", mode: 0o600 }); renameSync(temp, statePath); };
  const title = `Publish reviewed tkt ${state.requestId}`;
  function checked(run) {
    if (!["requested", "queued", "waiting", "pending", "in_progress", "completed"].includes(run?.status) || !Number.isSafeInteger(run?.id) || run.id <= 0 || run.head_sha !== spec.workflowSha || run.head_branch !== spec.branch.slice(11) || run.event !== "workflow_dispatch" || run.workflow_id !== spec.workflowId || run.path !== ".github/workflows/publish.yml" || run.display_title !== title || run.repository?.full_name !== candidate.repository || run.repository.id !== candidate.staging.ci.repositoryId || run.head_repository?.id !== run.repository.id) throw new ReleaseError("observed run differs from saved dispatch identity");
    return run;
  }
  async function observe() {
    let run;
    if (state.runId) run = checked(await actions.run(candidate.repository, state.runId));
    else {
      const matches = (await api.runs(candidate.repository, spec.workflowId, spec.branch.slice(11))).filter((item) => item.display_title === title);
      if (matches.length > 1) throw new ReleaseError("multiple runs match the saved request; inspect without redispatching");
      if (matches.length) { run = checked(matches[0]); state.runId = run.id; }
    }
    return { status: run?.status === "completed" && run.conclusion !== "success" ? "fail" : "pass", mode: "dispatch-observation", requestId: state.requestId, runId: state.runId, phase: run?.status ?? "dispatch-unconfirmed", conclusion: run?.conclusion ?? null, url: state.runId ? `https://github.com/${candidate.repository}/actions/runs/${state.runId}` : null, publication: "not-certified-by-dispatch-status", redispatched: false };
  }
  if (options.mode === "observe") return observe();
  mkdirSync(dirname(statePath), { recursive: true });
  const lockPath = `${statePath}.lock`, lockReceipt = encode({ pid: process.pid, identity, token: randomUUID() }); let fd;
  try { fd = openSync(lockPath, "wx", 0o600); } catch (error) { if (error.code === "EEXIST") throw new ReleaseError("dispatch state is locked; inspect its owner"); throw error; }
  try {
    writeFileSync(fd, lockReceipt); closeSync(fd); fd = undefined;
    // Re-read after acquiring the lock; another process may have dispatched already.
    if (existsSync(statePath)) {
      const latest = JSON.parse(readFileSync(statePath, "utf8"));
      if (latest.identity !== identity || latest.requestId !== state.requestId) throw new ReleaseError("dispatch state changed while acquiring its lock");
      state = latest;
    }
    if (!state.attempted) {
      const ref = await api.ref(candidate.repository, spec.branch.slice(11));
      if (ref?.ref !== spec.branch || ref.object?.type !== "commit" || ref.object.sha !== spec.workflowSha) throw new ReleaseError("remote dispatch branch differs from the reviewed candidate");
      state.attempted = true; save();
      let response;
      try { response = await api.dispatch(candidate.repository, spec.workflowId, { ref: spec.branch.slice(11), inputs: { review_sha: options.reviewSha, index_file: options.index, request_id: state.requestId, execute: true, retry_unconfirmed: retry } }); }
      catch { return await observe(); }
      if (Number.isSafeInteger(response?.workflow_run_id) && response.workflow_run_id > 0) { state.runId = response.workflow_run_id; save(); }
    }
    const result = await observe(); save(); return result;
  } finally { if (fd !== undefined) closeSync(fd); if (existsSync(lockPath) && readFileSync(lockPath, "utf8") === lockReceipt) unlinkSync(lockPath); }
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const args = process.argv.slice(2), options = { mode: args.shift() ?? "plan" };
    while (args.length) { const key = args.shift(); if (key === "--execute") options.execute = true; else if (key === "--retry-unconfirmed") options.retryUnconfirmed = true; else if (["--review-repo", "--review-sha", "--index", "--state"].includes(key) && args.length) options[key.slice(2).replace(/-([a-z])/g, (_, c) => c.toUpperCase())] = args.shift(); else throw new ReleaseError(`unknown/incomplete dispatch option: ${key}`); }
    const result = await dispatchPublication(options); console.log(encode(result)); process.exitCode = result.status === "pass" ? 0 : 1;
  } catch (error) { console.error(encode({ status: error instanceof ReleaseError ? "fail" : "error", errors: [error.message] })); process.exitCode = error instanceof ReleaseError ? 1 : 2; }
}

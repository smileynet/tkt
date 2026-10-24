import { test } from "node:test";
import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { dirname, join } from "node:path";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";
import { parse } from "yaml";
import { publisherOptions, stagedLocations } from "./publish-reviewed.mjs";
import { dispatchPublication, publicationDispatchApi } from "./dispatch-publication.mjs";
const repo = dirname(dirname(fileURLToPath(import.meta.url)));
const args = () => ["--repo", "/candidate", "--review-repo", "/review", "--review-sha", "a".repeat(40), "--index", "docs/releases/example/index.json", "--development-out", "/development", "--cargo", "/cargo", "--rustc", "/rustc"];
test("publisher CLI rejects overrides and defers reading its credential", () => {
  let reads = 0;
  const env = { get RELEASE_CARGO_TOKEN() { reads++; return "fixture-only"; } };
  const result = publisherOptions(args(), env);
  assert.equal(result.artifacts, stagedLocations(result).artifacts); assert.equal(reads, 0);
  assert.equal(result.getToken(), "fixture-only"); assert.equal(reads, 1);
  for (const extra of [["--token", "override"], ["--execute", "--dry-run"], ["--index", "../escape"]]) assert.throws(() => publisherOptions([...args(), ...extra]), /option|arguments/);
  for (const index of ["../escape", "/absolute", "path//index.json", "path/./index.json"]) assert.throws(() => stagedLocations({ reviewRepo: "/review", index }), /index path/);
});
test("publisher workflow is manual, serialized, defaults to planning and isolates credentials", () => {
  const workflow = parse(readFileSync(join(repo, ".github/workflows/publish.yml"), "utf8"));
  assert.deepEqual(Object.keys(workflow.on), ["workflow_dispatch"]);
  assert.equal(workflow.on.workflow_dispatch.inputs.execute.default, false);
  assert.equal(workflow.on.workflow_dispatch.inputs.retry_unconfirmed.default, false);
  assert.deepEqual(workflow.concurrency, { group: "tkt-release-publisher", "cancel-in-progress": false });
  const job = workflow.jobs.publish; assert.equal(job["runs-on"], "ubuntu-22.04");
  const checkouts = job.steps.filter((step) => step.uses?.startsWith("actions/checkout@"));
  assert.equal(checkouts.length, 2); assert.ok(checkouts.every((step) => step.with["persist-credentials"] === false));
  assert.equal(checkouts[0].with.ref, "${{ github.sha }}"); assert.equal(checkouts[1].with.ref, "${{ inputs.review_sha }}");
  const secretSteps = job.steps.filter((step) => step.env?.RELEASE_CARGO_TOKEN);
  assert.equal(secretSteps.length, 1); assert.match(secretSteps[0].run, /publish-reviewed\.mjs/);
  assert.ok(job.steps.filter((step) => step.run?.includes("npm ci")).every((step) => !step.env?.RELEASE_CARGO_TOKEN));
  assert.doesNotMatch(job.steps.map((step) => step.run ?? "").join("\n"), /cargo publish|gh release create|git push/);
});
async function fixture(fn) {
  const root = mkdtempSync(join(tmpdir(), "tkt publication dispatch ")), review = join(root, "review"); mkdirSync(review);
  const branch = "refs/heads/codex/release-tkt-0.5.0-fixture";
  const candidate = { repository: "smileynet/tkt", candidateSha: "a".repeat(40), staging: { ci: { repositoryId: 11 }, publication: { workflowSha: "a".repeat(40), workflowId: 12, branch, tagObject: "b".repeat(40), tagRecord: "fixture" } } };
  const state = { calls: 0, run: null, fail: false, invisible: false };
  const options = { mode: "dispatch", reviewRepo: review, reviewSha: "c".repeat(40), index: "docs/releases/example/index.json", state: join(root, "dispatch.json") };
  const api = {
    workflow: async () => ({ id: 12, path: ".github/workflows/publish.yml", state: "active" }),
    ref: async () => ({ ref: branch, object: { type: "commit", sha: candidate.candidateSha } }),
    dispatch: async (repository, id, body) => { state.calls++; assert.equal(repository, "smileynet/tkt"); assert.equal(id, 12); assert.equal(body.inputs.execute, true); state.run = { id: 17, head_sha: candidate.candidateSha, head_branch: branch.slice(11), event: "workflow_dispatch", workflow_id: 12, path: ".github/workflows/publish.yml", display_title: `Publish reviewed tkt ${body.inputs.request_id}`, repository: { full_name: "smileynet/tkt", id: 11 }, head_repository: { id: 11 }, status: "queued", conclusion: null }; if (state.fail) throw Error("response lost"); return { workflow_run_id: 17 }; },
    runs: async () => state.run && !state.invisible ? [structuredClone(state.run)] : [],
  };
  const services = { api, actions: { run: async () => structuredClone(state.run) }, readCandidate: async () => structuredClone(candidate), checkReview: () => {} };
  try { await fn({ root, options, state, candidate, services, run: (extra = {}) => dispatchPublication({ ...options, ...extra }, services) }); }
  finally { rmSync(root, { recursive: true, force: true }); }
}
test("local dispatch defaults to a read-only plan", () => fixture(async (f) => {
  assert.equal((await f.run()).mode, "dry-run"); assert.equal(f.state.calls, 0);
}));
test("dispatch saves intent before POST and replay observes without sending another request", () => fixture(async (f) => {
  const dispatch = f.services.api.dispatch;
  f.services.api.dispatch = async (...args) => { assert.equal(JSON.parse(readFileSync(f.options.state)).attempted, true); return dispatch(...args); };
  const result = await f.run({ execute: true }); assert.equal(result.runId, 17); assert.equal(result.phase, "queued");
  assert.equal((await f.run({ execute: true })).runId, 17); assert.equal(f.state.calls, 1);
  f.state.run.status = "completed"; f.state.run.conclusion = "success";
  const done = await f.run({ mode: "observe" }); assert.equal(done.status, "pass"); assert.equal(done.publication, "not-certified-by-dispatch-status");
}));
test("lost dispatch response is correlated without redispatching", () => fixture(async (f) => {
  f.state.fail = true; assert.equal((await f.run({ execute: true })).runId, 17);
  await f.run({ execute: true }); assert.equal(f.state.calls, 1);
}));
test("unknown response with no visible run remains uncertain, then observes the same request", () => fixture(async (f) => {
  f.state.fail = true; f.state.invisible = true;
  assert.equal((await f.run({ execute: true })).phase, "dispatch-unconfirmed");
  assert.equal((await f.run({ execute: true })).phase, "dispatch-unconfirmed"); assert.equal(f.state.calls, 1);
  f.state.invisible = false; assert.equal((await f.run({ mode: "observe" })).runId, 17);
}));
test("changed review, foreign run and failed run cannot report successful publication", () => fixture(async (f) => {
  await f.run({ execute: true });
  await assert.rejects(f.run({ execute: true, reviewSha: "d".repeat(40) }), /changed reviewed/);
  f.state.run.head_sha = "f".repeat(40); await assert.rejects(f.run({ mode: "observe" }), /differs/);
  f.state.run.head_sha = f.candidate.candidateSha; f.state.run.status = "completed"; f.state.run.conclusion = "failure";
  assert.equal((await f.run({ mode: "observe" })).status, "fail"); assert.equal(f.state.calls, 1);
}));
test("dispatch API uses typed POST bodies and paginated GETs only", () => {
  const calls = [], api = publicationDispatchApi({ run: (args, input) => { calls.push({ args, input }); return { status: 0, stdout: args.includes("--paginate") ? JSON.stringify([{ workflow_runs: [] }]) : '{"workflow_run_id":17}' }; } });
  api.dispatch("smileynet/tkt", 12, { ref: "candidate", inputs: { execute: true } }); api.runs("smileynet/tkt", 12, "candidate");
  assert.equal(calls[0].args[calls[0].args.indexOf("--method") + 1], "POST"); assert.equal(JSON.parse(calls[0].input).inputs.execute, true);
  assert.ok(calls[1].args.includes("--paginate")); assert.throws(() => api.workflow("other/repo", 12), /target/);
});

test("moved candidate branch refuses dispatch before recording an attempt", () => fixture(async (f) => {
  f.services.api.ref = async () => ({ ref: f.candidate.staging.publication.branch, object: { type: "commit", sha: "f".repeat(40) } });
  await assert.rejects(f.run({ execute: true }), /remote dispatch branch differs/); assert.equal(f.state.calls, 0);
}));

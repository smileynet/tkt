// Read reviewed CI artifacts without rebuilding files or changing remote state.
import yauzl from "yauzl";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { stableVersion, ReleaseError } from "./release-version.mjs";
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const positive = (value) => Number.isSafeInteger(value) && value > 0;
const digest = (value) => typeof value === "string" && /^[a-f0-9]{64}$/.test(value);
const name = (value) => typeof value === "string" && /^[A-Za-z0-9][A-Za-z0-9._-]*$/.test(value);
function checkedIdentity(expected) {
  if (!expected || !/^[A-Za-z0-9][A-Za-z0-9_.-]*\/[A-Za-z0-9][A-Za-z0-9_.-]*$/.test(expected.repository) || !/^[a-f0-9]{40}$/.test(expected.candidateSha) || !stableVersion(expected.version) || ![expected.repositoryId, expected.workflowId, expected.runId, expected.runAttempt, expected.artifactId, expected.sealJobId, expected.zipSize].every(positive) || !digest(expected.zipSha256) || !digest(expected.sealSha256)) throw new ReleaseError("invalid reviewed CI artifact identity");
  return `tkt-sealed-stage-${expected.runId}-${expected.runAttempt}`;
}
export async function inspectCiArchive(bytes, expected) {
  checkedIdentity(expected);
  if (!(bytes instanceof Uint8Array) || bytes.byteLength !== expected.zipSize || bytes.byteLength > 512 * 1024 * 1024 || hash(bytes) !== expected.zipSha256) throw new ReleaseError("downloaded CI archive differs from its reviewed seal");
  const zip = await yauzl.fromBufferPromise(Buffer.from(bytes), { lazyEntries: true, strictFileNames: true, validateEntrySizes: true });
  const files = new Map(), seen = new Set(); let total = 0;
  for await (const entry of zip.eachEntry()) {
    const path = entry.fileName, kind = (entry.externalFileAttributes >>> 16) & 0o170000;
    if (!name(path) || seen.has(path.toLowerCase()) || ![0, 0o100000].includes(kind) || (entry.externalFileAttributes & 0x10) || (entry.generalPurposeBitFlag & 1)) throw new ReleaseError(`unsafe, duplicate or nonregular CI archive entry: ${path}`);
    seen.add(path.toLowerCase());
    if (!Number.isSafeInteger(entry.uncompressedSize) || entry.uncompressedSize < 0 || entry.uncompressedSize > 128 * 1024 * 1024 || (total += entry.uncompressedSize) > 512 * 1024 * 1024 || seen.size > 128) throw new ReleaseError("CI archive exceeds resource limits");
    const stream = await zip.openReadStreamPromise(entry), chunks = []; let size = 0;
    for await (const chunk of stream) { size += chunk.length; if (size > entry.uncompressedSize) { stream.destroy(); throw new ReleaseError("CI archive entry exceeds its declared size"); } chunks.push(chunk); }
    if (size !== entry.uncompressedSize) throw new ReleaseError("CI archive entry size differs");
    files.set(path, Buffer.concat(chunks));
  }
  const sealed = files.get("tkt-stage.json");
  if (!sealed || hash(sealed) !== expected.sealSha256) throw new ReleaseError("CI stage record differs from the reviewed record");
  const record = JSON.parse(sealed.toString("utf8"));
  if (record.schema !== 1 || record.status !== "pass" || record.product !== "tkt" || record.version !== expected.version || record.candidateSha !== expected.candidateSha || record.repository !== expected.repository || record.runId !== String(expected.runId) || record.runAttempt !== String(expected.runAttempt) || record.workflow !== ".github/workflows/release.yml" || record.publicationPerformed !== false || !Array.isArray(record.files) || !record.files.length) throw new ReleaseError("CI stage record belongs to another candidate or execution");
  const assets = [], names = new Set();
  for (const file of record.files) {
    const local = files.get(file?.name);
    if (!name(file?.name) || file.name === "tkt-stage.json" || names.has(file.name) || !local || !positive(file.size) || local.length !== file.size || !digest(file.sha256) || hash(local) !== file.sha256) throw new ReleaseError("CI stage inventory has missing, duplicate or changed bytes");
    names.add(file.name); assets.push({ name: file.name, bytes: local, size: local.length, sha256: file.sha256 });
  }
  if (files.size !== assets.length + 1 || !names.has(`tkt-${expected.version}.crate`)) throw new ReleaseError("CI archive has unexpected files or lacks the source crate");
  return { record, assets, sealSha256: expected.sealSha256, zipSha256: expected.zipSha256 };
}
export async function readCiStage({ expected, api, now = Date.now() }) {
  const artifactName = checkedIdentity(expected);
  if (!Number.isFinite(now)) throw new ReleaseError("invalid CI observation time");
  function runIdentity(run) {
    if (run?.id !== expected.runId || run.run_attempt !== expected.runAttempt || run.head_sha !== expected.candidateSha || run.repository?.id !== expected.repositoryId || run.repository.full_name !== expected.repository || run.head_repository?.id !== expected.repositoryId || run.workflow_id !== expected.workflowId || run.path !== ".github/workflows/release.yml" || run.event !== "workflow_dispatch" || run.status !== "completed" || run.conclusion !== "success") throw new ReleaseError("CI run/attempt is changed, foreign or incomplete");
  }
  function artifactIdentity(artifact) {
    const run = artifact?.workflow_run;
    if (artifact?.id !== expected.artifactId || artifact.name !== artifactName || artifact.size_in_bytes !== expected.zipSize || artifact.digest !== `sha256:${expected.zipSha256}` || artifact.expired !== false || typeof artifact.expires_at !== "string" || !Number.isFinite(Date.parse(artifact.expires_at)) || Date.parse(artifact.expires_at) <= now || run?.id !== expected.runId || run.head_sha !== expected.candidateSha || run.repository_id !== expected.repositoryId || run.head_repository_id !== expected.repositoryId) throw new ReleaseError("CI artifact metadata differs or has expired");
  }
  async function observe() {
    runIdentity(await api.run(expected.repository, expected.runId));
    runIdentity(await api.attempt(expected.repository, expected.runId, expected.runAttempt));
    const jobs = await api.jobs(expected.repository, expected.runId, expected.runAttempt);
    if (!Array.isArray(jobs) || !jobs.length || jobs.some((job) => !positive(job?.id) || job.run_id !== expected.runId || job.status !== "completed" || job.conclusion !== "success")) throw new ReleaseError("CI attempt jobs are missing or incomplete");
    const seals = jobs.filter((job) => job.name === "seal-stage");
    if (seals.length !== 1 || seals[0].id !== expected.sealJobId) throw new ReleaseError("CI seal job differs from the reviewed attempt");
    const artifacts = await api.artifacts(expected.repository, expected.runId);
    if (!Array.isArray(artifacts)) throw new ReleaseError("invalid CI artifact list");
    const matches = artifacts.filter((artifact) => artifact?.id === expected.artifactId || artifact?.name === artifactName);
    if (matches.length !== 1) throw new ReleaseError("CI artifact is missing or its name was replaced");
    artifactIdentity(matches[0]); artifactIdentity(await api.artifact(expected.repository, expected.artifactId));
  }
  await observe();
  const log = await api.jobLog(expected.repository, expected.sealJobId);
  if (typeof log !== "string" || log.length > 16 * 1024 * 1024) throw new ReleaseError("invalid or oversized CI seal upload log");
  const lines = log.split(/\r?\n/);
  if (!lines.some((line) => line.endsWith(`Artifact ${artifactName} successfully finalized. Artifact ID ${expected.artifactId}`)) || !lines.some((line) => line.endsWith(`SHA256 digest of uploaded artifact is ${expected.zipSha256}`))) throw new ReleaseError("attempt-specific seal upload log does not bind the artifact ID/digest");
  const result = await inspectCiArchive(await api.download(expected.repository, expected.artifactId), expected);
  await observe();
  return { ...result, provenance: { repository: expected.repository, repositoryId: expected.repositoryId, candidateSha: expected.candidateSha, workflowId: expected.workflowId, runId: expected.runId, runAttempt: expected.runAttempt, artifactId: expected.artifactId, artifactName, sealJobId: expected.sealJobId } };
}
export function githubActionsReadApi({ run = (args, binary) => spawnSync("gh", args, { encoding: binary ? null : "utf8", maxBuffer: 512 * 1024 * 1024, env: { ...process.env, GH_HOST: "github.com", GH_PROMPT_DISABLED: "1" } }) } = {}) {
  function request(repository, path, { pages = false, field, binary = false, text = false } = {}) {
    if (!/^[A-Za-z0-9][A-Za-z0-9_.-]*\/[A-Za-z0-9][A-Za-z0-9_.-]*$/.test(repository)) throw new ReleaseError("invalid Actions repository");
    const args = ["api", "--hostname", "github.com", "--method", "GET", "-H", "X-GitHub-Api-Version: 2026-03-10"];
    if (pages) args.push("--paginate", "--slurp");
    if (binary || text) args.push("--allow-escape-sequences");
    args.push(`repos/${repository}/${path}`);
    const result = run(args, binary);
    if (result.error || result.status !== 0) throw new ReleaseError("Actions read failed; no artifact provenance established");
    if (binary || text) return result.stdout;
    const parsed = JSON.parse(result.stdout);
    if (!pages) return parsed;
    if (!Array.isArray(parsed) || parsed.some((page) => !Array.isArray(page?.[field]))) throw new ReleaseError("invalid paginated Actions response");
    return parsed.flatMap((page) => page[field]);
  }
  const id = (value) => { if (!positive(value)) throw new ReleaseError("invalid Actions object ID"); return value; };
  return {
    run: (repo, runId) => request(repo, `actions/runs/${id(runId)}`),
    attempt: (repo, runId, attempt) => request(repo, `actions/runs/${id(runId)}/attempts/${id(attempt)}`),
    jobs: (repo, runId, attempt) => request(repo, `actions/runs/${id(runId)}/attempts/${id(attempt)}/jobs?per_page=100`, { pages: true, field: "jobs" }),
    artifacts: (repo, runId) => request(repo, `actions/runs/${id(runId)}/artifacts?per_page=100`, { pages: true, field: "artifacts" }),
    artifact: (repo, artifactId) => request(repo, `actions/artifacts/${id(artifactId)}`),
    jobLog: (repo, jobId) => request(repo, `actions/jobs/${id(jobId)}/logs`, { text: true }),
    download: (repo, artifactId) => request(repo, `actions/artifacts/${id(artifactId)}/zip`, { binary: true }),
  };
}

// Read a reviewed staging index from an explicit Git commit, never a mutable sidecar.
import { readCiStage } from "./release-ci-artifacts.mjs";
import { readFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { resolve } from "node:path";
import { git } from "./release-git.mjs";
import { ReleaseError, stableVersion } from "./release-version.mjs";
import { verifyArtifacts } from "./release-artifacts.mjs";
import { readCandidateEvidence, requireMatchingIdentity } from "./release-verify.mjs";
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const gates = ["scope", "compatibility", "upgrade", "platform-approval", "final-audit", "staging"];

export function readStagedCandidate({ reviewRepo, reviewSha, indexFile, product, artifacts, evidence }) {
  if (!/^[a-f0-9]{40}$/.test(reviewSha) || git(reviewRepo, ["cat-file", "-t", reviewSha]).trim() !== "commit") throw new ReleaseError("staging requires an explicit reviewed commit");
  function committed(file, expectedHash) {
    if (typeof file !== "string" || !/^[A-Za-z0-9][A-Za-z0-9._/-]*$/.test(file) || file.split("/").some((part) => !part || part === "." || part === "..")) throw new ReleaseError("invalid staged evidence path");
    const entry = git(reviewRepo, ["ls-tree", reviewSha, "--", file]).split("\t")[0];
    if (!/^100(?:644|755) blob [a-f0-9]{40}$/.test(entry)) throw new ReleaseError("staged evidence must be a committed regular file");
    const bytes = git(reviewRepo, ["show", `${reviewSha}:${file}`]);
    if (expectedHash !== undefined && (!/^[a-f0-9]{64}$/.test(expectedHash) || hash(bytes) !== expectedHash)) throw new ReleaseError("staged evidence digest differs");
    return bytes;
  }
  function json(file, digest) {
    try { return JSON.parse(committed(file, digest)); }
    catch (error) { if (error instanceof SyntaxError) throw new ReleaseError("invalid committed staging JSON"); throw error; }
  }
  const index = json(indexFile);
  if (index?.schema !== 1 || index.kind !== "release-staging" || index.status !== "staged" || !Array.isArray(index.entries) || index.entries.length !== 2 || new Set(index.entries.map((entry) => entry?.product)).size !== 2 || index.entries.some((entry) => !["grey-matter", "tkt"].includes(entry?.product))) throw new ReleaseError("staging index must identify both coordinated products exactly once");
  const entry = index.entries.find((item) => item.product === product);
  if (!entry || !stableVersion(entry.version) || !/^[a-f0-9]{40}$/.test(entry.candidateSha) || typeof entry.repository !== "string" || !/^[A-Za-z0-9][A-Za-z0-9_.-]*\/[A-Za-z0-9][A-Za-z0-9_.-]*$/.test(entry.repository)) throw new ReleaseError("invalid staged candidate identity");
  let ci;
  if (product === "tkt") {
    if (!/^[a-f0-9]{64}$/.test(entry.ci?.sha256)) throw new ReleaseError("tkt staging requires a reviewed CI identity digest");
    ci = json(entry.ci?.file, entry.ci.sha256);
    requireMatchingIdentity({ product: "tkt", ...ci }, { name: "tkt", ...entry });
    if (ci.repository !== entry.repository) throw new ReleaseError("reviewed CI repository differs from staged candidate");
  }
  let publication;
  if (entry.publication !== undefined) {
    const spec = entry.publication;
    if (product !== "tkt" || spec?.workflowSha !== entry.candidateSha || !Number.isSafeInteger(spec.workflowId) || spec.workflowId <= 0 || typeof spec.branch !== "string" || !spec.branch.startsWith(`refs/heads/codex/release-tkt-${entry.version}-`) || !/^[a-f0-9]{40}$/.test(spec.tag?.objectSha)) throw new ReleaseError("invalid reviewed tkt publisher identity");
    git(reviewRepo, ["check-ref-format", spec.branch]);
    const record = committed(spec.tag.file);
    if (Buffer.byteLength(record) > 65536 || createHash("sha1").update(`tag ${Buffer.byteLength(record)}\0`).update(record).digest("hex") !== spec.tag.objectSha) throw new ReleaseError("reviewed tag record differs from its annotated object");
    const header = record.split("\n\n", 1)[0].split("\n");
    if (!header.includes(`object ${entry.candidateSha}`) || !header.includes("type commit") || !header.includes(`tag v${entry.version}`)) throw new ReleaseError("reviewed tag record differs from candidate identity");
    publication = { workflowSha: spec.workflowSha, workflowId: spec.workflowId, branch: spec.branch, tagObject: spec.tag.objectSha, tagRecord: record };
  }
  const metadata = verifyArtifacts(artifacts);
  requireMatchingIdentity(entry, metadata);
  if (hash(readFileSync(resolve(artifacts, "release-metadata.json"))) !== entry.metadataSha256) throw new ReleaseError("staged artifact metadata changed");
  const verification = readCandidateEvidence({ artifacts, evidence, verificationSha256: entry.verificationSha256 });
  if (!entry.gates || JSON.stringify(Object.keys(entry.gates).sort()) !== JSON.stringify([...gates].sort())) throw new ReleaseError("staging index has missing or unknown gates");
  for (const gate of gates) {
    const ref = entry.gates[gate];
    if (!/^[a-f0-9]{64}$/.test(ref?.sha256)) throw new ReleaseError("staged gate requires a digest");
    const report = json(ref?.file, ref.sha256);
    requireMatchingIdentity(report, metadata);
    if (report.schema !== 1 || report.kind !== "release-gate" || report.gate !== gate || report.status !== "pass" || !Array.isArray(report.blockers) || report.blockers.length || typeof report.summary !== "string" || !report.summary.trim() || !Array.isArray(report.evidence) || !report.evidence.length || report.evidence.some((item) => typeof item !== "string" || !item.trim())) throw new ReleaseError(`staged ${gate} evidence is incomplete or blocked`);
  }
  if (!/^[a-f0-9]{64}$/.test(entry.notes?.sha256)) throw new ReleaseError("staged notes require a digest");
  const notes = committed(entry.notes?.file, entry.notes.sha256);
  if (!notes.trim()) throw new ReleaseError("staged release notes are empty");
  const assets = [...metadata.assets.map((asset) => asset.name), "release-metadata.json", "SHA256SUMS"].map((name) => {
    const bytes = readFileSync(resolve(artifacts, name));
    return { name, bytes, size: bytes.length, sha256: hash(bytes) };
  });
  return { repository: entry.repository, product, version: entry.version, candidateSha: entry.candidateSha, title: `v${entry.version}`, notes, assets, gates: { status: "pass", product, version: entry.version, candidateSha: entry.candidateSha, pending: [] }, staging: { reviewSha, indexFile, verificationSha256: verification.verificationSha256, ...(ci ? { ci } : {}), ...(publication ? { publication } : {}) } };
}

export async function readStagedTktCandidate({ api, now, ...options }) {
  const candidate = readStagedCandidate({ ...options, product: "tkt" });
  const ci = await readCiStage({ expected: candidate.staging.ci, api, now });
  const assets = [...candidate.assets, ...ci.assets, ci.seal];
  if (new Set(assets.map((asset) => asset.name.toLowerCase())).size !== assets.length) throw new ReleaseError("plugin and CI release assets have colliding names");
  return { ...candidate, assets, ciStage: { ...ci.provenance, sealSha256: ci.sealSha256, zipSha256: ci.zipSha256 } };
}

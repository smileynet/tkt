#!/usr/bin/env node
// Run fresh product/native gates for one sealed commit identity; never publish.
// Usage: node scripts/release-verify.mjs --repo DIR --sha FULL_SHA --artifacts DIR --out NEW_DIR
//        [--execute | --dry-run] [--codex BINARY] [--claude BINARY] [--agy BINARY]
import { existsSync, lstatSync, mkdirSync, readFileSync, realpathSync, renameSync, writeFileSync } from "node:fs";
import { createHash, randomUUID } from "node:crypto";
import { basename, dirname, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { git } from "./release-git.mjs";
import { ReleaseError } from "./release-version.mjs";
import { verifyArtifacts } from "./release-artifacts.mjs";
import { verifyProduct } from "./release-product.mjs";
import { verifyNative } from "./release-native.mjs";
const encode = (value) => JSON.stringify(value, null, 2) + "\n";
const digest = (path) => createHash("sha256").update(readFileSync(path)).digest("hex");
function canonical(path) {
  if (existsSync(path)) return realpathSync.native(path);
  const parent = dirname(path);
  if (parent === path) throw new ReleaseError(`path has no accessible filesystem root: ${path}`);
  return resolve(canonical(parent), basename(path));
}
export function requireMatchingIdentity(product, metadata) {
  if (product.candidateSha !== metadata.candidateSha || product.product !== metadata.name || product.version !== metadata.version) throw new ReleaseError("candidate and sealed artifacts have different commit, product or version identities");
}
// The expected index hash must come from reviewed staging, not this evidence directory.
export function readCandidateEvidence({ artifacts, evidence, verificationSha256 }) {
  const root = realpathSync.native(resolve(evidence)), metadata = verifyArtifacts(realpathSync.native(resolve(artifacts)));
  function sealed(file, sha256) {
    if (typeof file !== "string" || !/^[A-Za-z0-9][A-Za-z0-9._/-]*$/.test(file) || file.split("/").some((part) => !part || part === "." || part === "..") || !/^[a-f0-9]{64}$/.test(sha256)) throw new ReleaseError("invalid evidence path or digest");
    const path = resolve(root, file);
    if (!realpathSync.native(path).startsWith(root + sep) || !lstatSync(path).isFile() || lstatSync(path).isSymbolicLink() || digest(path) !== sha256) throw new ReleaseError("evidence file escaped, changed or is not regular");
    return readFileSync(path);
  }
  function json(file, sha256) {
    try { return JSON.parse(sealed(file, sha256)); }
    catch (error) { if (error instanceof SyntaxError) throw new ReleaseError("invalid evidence JSON"); throw error; }
  }
  const index = json("verification.json", verificationSha256);
  requireMatchingIdentity(index, metadata);
  if (index.schema !== 1 || index.status !== "pass" || index.mode !== "candidate-verification" || index.publishable !== false || index.phases?.product !== "pass" || index.phases?.native !== "pass" || index.metadataSha256 !== digest(resolve(artifacts, "release-metadata.json"))) throw new ReleaseError("candidate verification index is incomplete or differs from artifacts");
  const pending = ["scope", "compatibility", "upgrade", "platform-approval", "final-audit", "staging"];
  if (JSON.stringify(index.pending) !== JSON.stringify(pending)) throw new ReleaseError("verification cannot waive staging or audit gates");
  const productRef = index.productEvidence, nativeRef = index.nativeEvidence;
  if (productRef?.file !== "product/product-evidence.json" || nativeRef?.file !== "native/native-evidence.json") throw new ReleaseError("unexpected evidence report location");
  const product = json(productRef.file, productRef.sha256), native = json(nativeRef.file, nativeRef.sha256);
  for (const [report, mode] of [[product, "product-evidence"], [native, "native-evidence"]]) {
    requireMatchingIdentity(report, metadata);
    if (report.schema !== 1 || report.status !== "pass" || report.mode !== mode || report.publishable !== false) throw new ReleaseError("underlying evidence did not pass an executed gate");
  }
  if (!Array.isArray(product.checks) || product.checks.length !== 1 || product.candidateError) throw new ReleaseError("product checks are incomplete");
  for (const check of product.checks) {
    if (check.exit !== 0 || check.signal !== null || check.error !== null || check.log !== "1-product.log") throw new ReleaseError("product command failed or evidence is incomplete");
    sealed(`product/${check.log}`, check.sha256);
  }
  if (!Array.isArray(native.rows) || native.rows.length !== metadata.assets.length) throw new ReleaseError("native evidence omitted an artifact");
  for (const asset of metadata.assets) {
    const rows = native.rows.filter((row) => row.host === asset.host && row.asset === asset.name && row.assetSha256 === asset.sha256);
    if (rows.length !== 1 || rows[0].status !== "pass" || typeof rows[0].nativeVersion !== "string" || !rows[0].nativeVersion.trim() || !Array.isArray(rows[0].checks) || !rows[0].checks.length || rows[0].checks.some((check) => check.exit !== 0 || check.signal)) throw new ReleaseError("native artifact checks are missing, duplicate or failed");
  }
  return { status: "pass", publishable: false, product: metadata.name, version: metadata.version, candidateSha: metadata.candidateSha, verificationSha256, pending, product, native };
}

export async function verifyCandidate(options) {
  if (options.execute && options.dryRun) throw new ReleaseError("--execute and --dry-run cannot combine");
  const repo = realpathSync.native(git(resolve(options.repo), ["rev-parse", "--show-toplevel"]).trim()), out = canonical(resolve(options.out)), artifacts = realpathSync.native(resolve(options.artifacts));
  const productOptions = { repo, sha: options.sha, out: resolve(out, "product"), progress: options.progress };
  const nativeOptions = { artifacts, out: resolve(out, "native"), codex: options.codex, claude: options.claude, agy: options.agy, progress: options.progress };
  const product = verifyProduct(productOptions), metadata = verifyArtifacts(artifacts);
  requireMatchingIdentity(product, metadata);
  const metadataHash = digest(resolve(artifacts, "release-metadata.json"));
  const native = await verifyNative(nativeOptions);
  const result = { schema: 1, status: "pass", mode: options.execute ? "candidate-verification" : "dry-run", publishable: false, candidateSha: metadata.candidateSha, product: metadata.name, version: metadata.version, metadataSha256: metadataHash, phases: { product: "not-run", native: "not-run" }, plan: { product: product.commands, native: native.plan }, pending: ["scope", "compatibility", "upgrade", "platform-approval", "final-audit", "staging"] };
  if (!options.execute) return result;
  if (existsSync(out) || [repo, artifacts].some((root) => out === root || out.startsWith(root + sep))) throw new ReleaseError("verification evidence requires a new directory outside the candidate and sealed artifacts");
  mkdirSync(dirname(out), { recursive: true }); mkdirSync(out);
  result.started = new Date().toISOString(); result.status = "not-run";
  const save = () => {
    const temporary = resolve(out, `verification.${randomUUID()}.tmp`);
    writeFileSync(temporary, encode(result), { flag: "wx" }); renameSync(temporary, resolve(out, "verification.json"));
  };
  save();
  try {
    options.progress?.("product");
    const checkedProduct = verifyProduct({ ...productOptions, execute: true });
    requireMatchingIdentity(checkedProduct, metadata);
    result.phases.product = checkedProduct.status;
    result.productEvidence = { file: "product/product-evidence.json", sha256: digest(resolve(out, "product/product-evidence.json")), platform: checkedProduct.platform };
    save();
    if (checkedProduct.status !== "pass") throw new ReleaseError("product gate failed; native verification was not run");
    options.progress?.("native");
    const checkedNative = await verifyNative({ ...nativeOptions, execute: true });
    requireMatchingIdentity(checkedNative, metadata);
    result.phases.native = checkedNative.status;
    result.nativeEvidence = { file: "native/native-evidence.json", sha256: digest(resolve(out, "native/native-evidence.json")), platform: checkedNative.platform, isolation: checkedNative.isolation };
    if (checkedNative.status !== "pass") throw new ReleaseError("native gate failed");
    // Neither an intervening checkout move nor changed bundles may inherit earlier checks.
    verifyProduct(productOptions); requireMatchingIdentity(product, verifyArtifacts(artifacts));
    if (digest(resolve(artifacts, "release-metadata.json")) !== metadataHash) throw new ReleaseError("sealed metadata changed during verification");
    result.status = "pass";
  } catch (error) {
    result.status = "fail"; result.errors = [error.message];
  } finally {
    result.finished = new Date().toISOString(); save();
  }
  return result;
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const args = process.argv.slice(2), options = {};
    while (args.length) {
      const key = args.shift();
      if (key === "--execute") options.execute = true;
      else if (key === "--dry-run") options.dryRun = true;
      else if (["--repo", "--sha", "--artifacts", "--out", "--codex", "--claude", "--agy"].includes(key) && args.length) options[key.slice(2)] = args.shift();
      else throw new ReleaseError(`unknown/incomplete option: ${key}`);
    }
    if (!options.repo || !options.sha || !options.artifacts || !options.out || (options.execute && options.dryRun)) throw new ReleaseError("usage: --repo DIR --sha FULL_SHA --artifacts DIR --out NEW_DIR [--execute | --dry-run]");
    options.progress = (phase) => console.error(`verify: ${phase}`);
    const result = await verifyCandidate(options); console.log(encode(result)); process.exitCode = result.status === "fail" ? 1 : 0;
  } catch (error) { const expected = error instanceof ReleaseError; console.error(encode({ status: expected ? "fail" : "error", errors: [error.message] })); process.exitCode = expected ? 1 : 2; }
}

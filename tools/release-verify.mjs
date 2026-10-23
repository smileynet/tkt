#!/usr/bin/env node
// Run fresh product/native gates for one sealed commit identity; never publish.
// Usage: node scripts/release-verify.mjs --repo DIR --sha FULL_SHA --artifacts DIR --out NEW_DIR
//        [--execute | --dry-run] [--codex BINARY] [--claude BINARY] [--agy BINARY]
import { existsSync, mkdirSync, readFileSync, realpathSync, renameSync, writeFileSync } from "node:fs";
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
  if (existsSync(path)) return realpathSync(path);
  const parent = dirname(path);
  if (parent === path) throw new ReleaseError(`path has no accessible filesystem root: ${path}`);
  return resolve(canonical(parent), basename(path));
}
export function requireMatchingIdentity(product, metadata) {
  if (product.candidateSha !== metadata.candidateSha || product.product !== metadata.name || product.version !== metadata.version) throw new ReleaseError("candidate and sealed artifacts have different commit, product or version identities");
}
export async function verifyCandidate(options) {
  if (options.execute && options.dryRun) throw new ReleaseError("--execute and --dry-run cannot combine");
  const repo = realpathSync(git(resolve(options.repo), ["rev-parse", "--show-toplevel"]).trim()), out = canonical(resolve(options.out)), artifacts = realpathSync(resolve(options.artifacts));
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

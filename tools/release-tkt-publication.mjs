// Coordinate the sole tkt publisher; adapters own transport, persistence and Git scope.
import { createHash } from "node:crypto";
import { ReleaseError } from "./release-version.mjs";
import { validateTktCrate, reconcileTktCrate } from "./release-registry.mjs";
import { reconcileGithubRelease } from "./release-publish.mjs";
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
export function tktRegistryIntent({ candidate, crate }) {
  validateTktCrate(crate);
  if (candidate?.product !== "tkt" || candidate.version !== crate.version || !/^[a-f0-9]{40}$/.test(candidate.candidateSha) || typeof candidate.repository !== "string" || !/^[A-Za-z0-9][A-Za-z0-9_.-]*\/[A-Za-z0-9][A-Za-z0-9_.-]*$/.test(candidate.repository)) throw new ReleaseError("registry intent requires the exact tkt candidate identity");
  const record = { schema: 1, product: "tkt", repository: candidate.repository, candidateSha: candidate.candidateSha, version: crate.version, registry: "https://crates.io", crate: { name: `tkt-${crate.version}.crate`, size: crate.bytes.byteLength, sha256: crate.sha256 }, uploadMayHaveBeenAttempted: true };
  const bytes = Buffer.from(JSON.stringify(record, null, 2) + "\n");
  return { name: "tkt-registry-intent.json", bytes, size: bytes.length, sha256: hash(bytes) };
}
export async function reconcileTktPublication({ candidate, crate, api, registry, prepareUpload, promoteStable, execute = false, retryUnconfirmed = false, observe = () => {} }) {
  if (typeof execute !== "boolean" || typeof retryUnconfirmed !== "boolean") throw new ReleaseError("tkt execution/recovery flags must be boolean");
  const receipt = tktRegistryIntent({ candidate, crate });
  if (!Array.isArray(candidate.assets)) throw new ReleaseError("tkt publication requires reviewed assets");
  const supplied = candidate.assets.filter((asset) => asset?.name === receipt.name);
  if (supplied.length > 1 || supplied.some((asset) => !(asset.bytes instanceof Uint8Array) || asset.size !== receipt.size || asset.sha256 !== receipt.sha256 || hash(asset.bytes) !== receipt.sha256)) throw new ReleaseError("registry intent differs from the deterministic candidate receipt");
  const ordinaryAssets = candidate.assets.filter((asset) => asset?.name !== receipt.name);
  const crateAssets = ordinaryAssets.filter((asset) => asset?.name === `tkt-${crate.version}.crate`);
  if (crateAssets.length !== 1 || crateAssets[0].sha256 !== crate.sha256 || crateAssets[0].size !== crate.bytes.byteLength) throw new ReleaseError("publication assets do not contain the sealed registry crate");
  let full = { ...candidate, assets: [...ordinaryAssets, receipt] };
  const github = (options = {}) => reconcileGithubRelease({ candidate: full, api, publisher: "cargo-dist", observe: (state) => observe({ channel: "github", ...state }), ...options });
  const crateState = (options = {}) => reconcileTktCrate({ crate, registry, observe: (state) => observe({ channel: "registry", ...state }), ...options });
  let gh = await github(), cargo = await crateState();
  if (gh.releaseId !== null) full = { ...full, releaseId: gh.releaseId };
  const priorIntent = !gh.missingAssets.includes(receipt.name);
  if (!execute) return { status: "pass", mode: "dry-run", github: gh, registry: cargo, intent: priorIntent ? "present" : "missing", stable: "not-attempted" };
  if (typeof promoteStable !== "function") throw new ReleaseError("tkt publication requires a bound stable promotion adapter");
  if (cargo.phase === "missing" && priorIntent && !retryUnconfirmed) throw new ReleaseError("prior registry intent is unconfirmed; inspect before explicitly retrying");
  if (gh.phase !== "published" && await api.immutableEnabled(full.repository) !== true) throw new ReleaseError("repository release immutability must be enabled before publication");
  let transport;
  try {
    if (cargo.phase === "missing") {
      if (typeof prepareUpload !== "function") throw new ReleaseError("no prepared registry transport is configured");
      const prepared = await prepareUpload(crate);
      if (typeof prepared?.publish !== "function" || typeof prepared?.close !== "function") throw new ReleaseError("registry preparation did not return a scoped transport");
      transport = prepared;
    }
    if (priorIntent) gh = await github({ execute: true, publish: false });
    else {
      const ordinary = await reconcileGithubRelease({ candidate: { ...full, assets: ordinaryAssets }, api, publisher: "cargo-dist", execute: true, publish: false, observe: (state) => observe({ channel: "github", ...state }) });
      full = { ...full, releaseId: ordinary.releaseId };
      gh = await github();
      if (!gh.missingAssets.includes(receipt.name)) throw new ReleaseError("another publisher created registry intent; this execution has no initial attempt permission");
      if (gh.phase !== "draft" || gh.missingAssets.some((name) => name !== receipt.name)) throw new ReleaseError("ordinary release assets changed before the registry barrier");
      let uploaded;
      try { uploaded = await api.upload(full.repository, gh.releaseId, { name: receipt.name, bytes: receipt.bytes }); }
      catch (error) {
        await github();
        throw new ReleaseError("registry intent upload outcome is uncertain; no initial Cargo attempt permitted", { cause: error });
      }
      if (!Number.isSafeInteger(uploaded?.id) || uploaded.id <= 0 || uploaded.name !== receipt.name || uploaded.state !== "uploaded" || uploaded.size !== receipt.size || uploaded.digest !== `sha256:${receipt.sha256}`) throw new ReleaseError("registry intent creation was not confirmed; no initial Cargo attempt permitted");
      gh = await github();
      const receipts = (await api.assets(full.repository, gh.releaseId)).filter((asset) => asset?.name === receipt.name);
      if (receipts.length !== 1 || receipts[0].id !== uploaded.id || receipts[0].state !== "uploaded" || receipts[0].size !== receipt.size || receipts[0].digest !== `sha256:${receipt.sha256}`) throw new ReleaseError("registry intent ID changed after creation; no initial Cargo attempt permitted");
    }
    if (gh.missingAssets.length) throw new ReleaseError("release assets are incomplete before Cargo publication");
    if (cargo.phase === "published") {
      cargo = await crateState();
      if (cargo.phase !== "published") throw new ReleaseError("previously published crate is no longer observable; refusing a new upload");
    } else cargo = await crateState({ execute: true, previouslyAttempted: priorIntent, retryUnconfirmed, registry: { inspect: (version) => registry.inspect(version), publish: (sealed) => transport.publish(sealed) } });
    gh = await github({ execute: true });
    cargo = await crateState();
    if (cargo.phase !== "published" || gh.phase !== "published" || !gh.immutable) throw new ReleaseError("registry/GitHub publication is incomplete; stable was not attempted");
    const stable = await promoteStable(full);
    if (stable?.status !== "pass" || stable.complete !== true || stable.sha !== full.candidateSha) throw new ReleaseError("stable promotion was not observed at the exact candidate");
    gh = await github(); cargo = await crateState();
    if (gh.phase !== "published" || !gh.immutable || cargo.phase !== "published") throw new ReleaseError("publication changed after stable promotion; inspect all channels");
    return { status: "pass", mode: "tkt-publication", github: gh, registry: cargo, intent: "present", stable };
  } finally { if (transport) await transport.close(); }
}

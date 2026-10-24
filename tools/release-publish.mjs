// Reconcile one assigned GitHub publisher through a supplied API adapter.
// Library only: API reconciliation and explicit Git promotion; no publication CLI.
import { pushStableRef } from "./release-git.mjs";
import { createHash } from "node:crypto";
import { stableVersion, ReleaseError } from "./release-version.mjs";

function identity(candidate, publisher) {
  if (!candidate || !["grey-matter", "tkt"].includes(candidate.product) || !stableVersion(candidate.version) || !/^[a-f0-9]{40}$/.test(candidate.candidateSha) || !/^[A-Za-z0-9][A-Za-z0-9_.-]*\/[A-Za-z0-9][A-Za-z0-9_.-]*$/.test(candidate.repository)) throw new ReleaseError("invalid publication identity");
  if (publisher !== (candidate.product === "tkt" ? "cargo-dist" : "local")) throw new ReleaseError(`the sole publisher for ${candidate.product} is ${candidate.product === "tkt" ? "cargo-dist" : "local"}`);
  if (candidate.tagObject !== undefined && !/^[a-f0-9]{40}$/.test(candidate.tagObject)) throw new ReleaseError("invalid sealed tag object");
  const gates = candidate.gates;
  if (gates?.status !== "pass" || gates.candidateSha !== candidate.candidateSha || gates.product !== candidate.product || gates.version !== candidate.version || !Array.isArray(gates.pending) || gates.pending.length) throw new ReleaseError("candidate gates are incomplete or belong to another identity");
  if (typeof candidate.title !== "string" || !candidate.title.trim() || typeof candidate.notes !== "string" || !candidate.notes.trim()) throw new ReleaseError("publication requires reviewed notes and title");
  if (!Array.isArray(candidate.assets) || !candidate.assets.length) throw new ReleaseError("publication requires sealed assets");
  const assets = new Map();
  for (const asset of candidate.assets) {
    if (!asset || typeof asset.name !== "string" || !/^[A-Za-z0-9][A-Za-z0-9._-]*$/.test(asset.name) || assets.has(asset.name) || !(asset.bytes instanceof Uint8Array) || !Number.isSafeInteger(asset.size) || asset.size < 0 || asset.size !== asset.bytes.byteLength || !/^[a-f0-9]{64}$/.test(asset.sha256) || createHash("sha256").update(asset.bytes).digest("hex") !== asset.sha256) throw new ReleaseError("invalid, duplicate or changed local asset");
    assets.set(asset.name, { name: asset.name, bytes: asset.bytes, size: asset.size, digest: `sha256:${asset.sha256}` });
  }
  if (candidate.releaseId !== undefined && (!Number.isSafeInteger(candidate.releaseId) || candidate.releaseId <= 0)) throw new ReleaseError("invalid persisted release ID");
  return { repository: candidate.repository, tag: `v${candidate.version}`, sha: candidate.candidateSha, title: candidate.title, notes: candidate.notes, assets, tagObject: candidate.tagObject, releaseId: candidate.releaseId ?? null };
}

export async function reconcileGithubRelease({ candidate, api, publisher, execute = false, publish = true, progress = () => {}, observe = () => {} }) {
  if (typeof execute !== "boolean" || typeof publish !== "boolean") throw new ReleaseError("publication execution mode and publish flag must be boolean");
  const expected = identity(candidate, publisher), repo = expected.repository;
  let releaseId = expected.releaseId;
  function observed(state) {
    observe({ phase: state.phase, releaseId, url: state.release?.html_url ?? null, immutable: state.release?.immutable ?? false, missingAssets: state.missing });
    return state;
  }
  async function inspect() {
    const tag = await api.tag(repo, expected.tag);
    if (!tag || tag.sha !== expected.sha || tag.annotated !== true) throw new ReleaseError("remote tag must be annotated and resolve to the candidate commit");
    if (expected.tagObject && tag.objectSha !== expected.tagObject) throw new ReleaseError("remote annotated tag object differs from the sealed object");
    // The adapter must paginate authenticated list results; by-tag 404 misses drafts.
    const releases = await api.releases(repo);
    if (!Array.isArray(releases) || releases.some((item) => !item || typeof item.tag_name !== "string" || !Number.isSafeInteger(item.id) || item.id <= 0)) throw new ReleaseError("invalid release-list response");
    const matches = releases.filter((item) => item.tag_name === expected.tag);
    if (matches.length > 1) throw new ReleaseError("multiple releases match the candidate tag");
    const release = matches[0];
    if (!release) {
      if (releaseId !== null) throw new ReleaseError("observed release ID disappeared; refusing a replacement");
      return observed({ phase: "missing", release: null, missing: [...expected.assets.keys()] });
    }
    if (releaseId !== null && release.id !== releaseId) throw new ReleaseError("release ID changed during publication");
    releaseId = release.id;
    if (release.name !== expected.title || release.body !== expected.notes || typeof release.draft !== "boolean" || typeof release.immutable !== "boolean" || (release.draft && release.immutable)) throw new ReleaseError("release notes or title differ, or release state is invalid");
    const remoteAssets = await api.assets(repo, release.id);
    if (!Array.isArray(remoteAssets)) throw new ReleaseError("invalid asset-list response");
    const seen = new Set();
    for (const asset of remoteAssets) {
      const local = expected.assets.get(asset?.name);
      if (!local) throw new ReleaseError(`unexpected asset: ${asset?.name}`);
      if (seen.has(asset.name) || asset.state !== "uploaded" || asset.size !== local.size || asset.digest !== local.digest) throw new ReleaseError(`asset differs or is incomplete: ${asset.name}; refusing to clobber`);
      seen.add(asset.name);
    }
    const missing = [...expected.assets.keys()].filter((name) => !seen.has(name));
    if (!release.draft && (!release.immutable || missing.length)) throw new ReleaseError("published release is not immutable or has missing assets");
    return observed({ phase: release.draft ? "draft" : "published", release, missing });
  }
  let snapshot = await inspect();
  const result = () => ({ status: "pass", mode: execute ? (publish ? "publisher-reconciliation" : "draft-reconciliation") : "dry-run", publisher, phase: snapshot.phase, releaseId, url: snapshot.release?.html_url ?? null, immutable: snapshot.release?.immutable ?? false, missingAssets: snapshot.missing });
  if (!execute || snapshot.phase === "published") return result();
  if (await api.immutableEnabled(repo) !== true) throw new ReleaseError("repository release immutability must be enabled before publication");

  async function apply(label, action, observed) {
    progress(label);
    let failure;
    try { await action(); }
    catch (error) { failure = error; }
    // A timeout is an unknown outcome, never evidence that the mutation did not land.
    snapshot = await inspect();
    if (!observed(snapshot)) throw new ReleaseError(`${label} success was not observed${failure ? `: ${String(failure)}` : ""}; inspect and resume this identity`);
  }
  if (snapshot.phase === "missing") await apply("create draft", () => api.createDraft(repo, { tag: expected.tag, title: expected.title, notes: expected.notes }), (state) => state.phase !== "missing");
  for (const name of snapshot.missing) {
    if (snapshot.phase === "published") break;
    if (!snapshot.missing.includes(name)) continue;
    const asset = expected.assets.get(name);
    await apply(`upload ${name}`, () => api.upload(repo, releaseId, { name, bytes: asset.bytes }), (state) => !state.missing.includes(name));
  }
  // Reinspect immediately before publication; no remembered phase can waive a gate.
  snapshot = await inspect();
  if (snapshot.phase === "draft" && snapshot.missing.length) throw new ReleaseError("assets changed before draft staging completed");
  if (snapshot.phase === "draft" && publish) {
    if (await api.immutableEnabled(repo) !== true) throw new ReleaseError("assets or repository immutability changed before publication");
    await apply("publish draft", () => api.publish(repo, releaseId), (state) => state.phase === "published");
  }
  return result();
}

// Re-observe immutable bytes before and after promoting the release-only channel.
export async function promotePublishedStable({ repo, remote = "origin", candidate, api, publisher, execute = false }) {
  if (typeof execute !== "boolean") throw new ReleaseError("stable execution mode must be boolean");
  const published = await reconcileGithubRelease({ candidate, api, publisher });
  if (published.phase !== "published" || !published.immutable) throw new ReleaseError("stable requires the complete immutable published release");
  const bound = { ...candidate, releaseId: published.releaseId };
  const stable = pushStableRef({ repo, remote, candidateSha: bound.candidateSha, execute });
  const checked = await reconcileGithubRelease({ candidate: bound, api, publisher });
  if (checked.phase !== "published" || !checked.immutable) throw new ReleaseError("publication changed during stable promotion; inspect without rolling refs back");
  return { ...stable, releaseId: checked.releaseId, url: checked.url, publication: "verified" };
}

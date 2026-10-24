// Observe crates.io's exact tkt version/checksum; publication uses an injected adapter.
import { createHash } from "node:crypto";
import { ReleaseError, stableVersion } from "./release-version.mjs";
const checksum = (bytes) => createHash("sha256").update(bytes).digest("hex");
const semver = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?$/;

export function cratesIoTkt({ request = fetch } = {}) {
  return {
    async inspect(version) {
      if (!stableVersion(version)) throw new ReleaseError("registry observation requires a stable tkt version");
      const response = await request("https://index.crates.io/3/t/tkt", { redirect: "error", cache: "no-store", signal: AbortSignal.timeout(15000), headers: { Accept: "text/plain", "Cache-Control": "no-cache", "User-Agent": "grey-matter-release-tools (https://github.com/smileynet/grey-matter)" } });
      if (response.status !== 200) throw new ReleaseError("tkt registry index unavailable; absence is unconfirmed");
      const lines = (await response.text()).trim().split("\n");
      const versions = new Set();
      let selected;
      for (const line of lines) {
        let row;
        try { row = JSON.parse(line); }
        catch (error) { if (error instanceof SyntaxError) throw new ReleaseError("invalid tkt registry index JSON"); throw error; }
        if (row?.name !== "tkt" || typeof row.vers !== "string" || !semver.test(row.vers) || !/^[a-f0-9]{64}$/.test(row.cksum) || typeof row.yanked !== "boolean") throw new ReleaseError("invalid tkt registry index entry");
        const key = row.vers.split("+")[0];
        if (versions.has(key)) throw new ReleaseError("duplicate tkt registry version identity");
        versions.add(key);
        if (key === version) selected = row;
      }
      if (!selected) return { phase: "missing", name: "tkt", version };
      if (selected.vers !== version) throw new ReleaseError("registry version differs by build metadata; refusing a colliding version");
      return { phase: "published", name: "tkt", version, sha256: selected.cksum, yanked: selected.yanked };
    },
  };
}

export function validateTktCrate(crate) {
  if (crate?.name !== "tkt" || !stableVersion(crate.version) || !(crate.bytes instanceof Uint8Array) || !crate.bytes.byteLength || !/^[a-f0-9]{64}$/.test(crate.sha256) || checksum(crate.bytes) !== crate.sha256) throw new ReleaseError("invalid or changed sealed tkt crate");
}

export async function reconcileTktCrate({ crate, registry, execute = false, previouslyAttempted = false, retryUnconfirmed = false, observe = () => {} }) {
  if ([execute, previouslyAttempted, retryUnconfirmed].some((value) => typeof value !== "boolean")) throw new ReleaseError("registry execution/recovery flags must be boolean");
  validateTktCrate(crate);
  async function inspect() {
    const state = await registry.inspect(crate.version);
    if (state?.name !== "tkt" || state.version !== crate.version || !["missing", "published"].includes(state.phase)) throw new ReleaseError("registry response belongs to another crate identity");
    if (state.phase === "published" && (state.sha256 !== crate.sha256 || state.yanked !== false)) throw new ReleaseError("published crate differs or is yanked; never replace or automatically unyank it");
    observe(state); return state;
  }
  let state = await inspect();
  const result = () => ({ status: "pass", mode: execute ? "registry-reconciliation" : "dry-run", phase: state.phase, name: "tkt", version: crate.version, sha256: crate.sha256, url: state.phase === "published" ? `https://crates.io/crates/tkt/${crate.version}` : null });
  if (!execute || state.phase === "published") return result();
  if (previouslyAttempted && !retryUnconfirmed) throw new ReleaseError("prior publication is unconfirmed; inspect the exact version before explicitly retrying");
  if (typeof registry.publish !== "function") throw new ReleaseError("no registry publication adapter is configured");
  observe({ phase: "publication-attempted", name: "tkt", version: crate.version, sha256: crate.sha256 });
  let failure;
  try { await registry.publish(crate); }
  catch (error) { failure = error; }
  state = await inspect();
  if (state.phase !== "published") throw new ReleaseError("crate publication was not observed; retain this identity and inspect before retrying", { cause: failure });
  return result();
}

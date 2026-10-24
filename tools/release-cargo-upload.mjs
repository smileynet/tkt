// Prepare a Linux-only, checksum-bound Cargo transport before the durable intent barrier.
// Usage: await prepareCargoUpload({ repo, candidateSha, crate, cargo, rustc, getToken });
import { spawn, execFileSync } from "node:child_process";
import { isAbsolute } from "node:path";
import { fileURLToPath } from "node:url";
import { parse as parseToml } from "smol-toml";
import { ReleaseError, stableVersion } from "./release-version.mjs";
import { validateTktCrate } from "./release-registry.mjs";

const helper = fileURLToPath(new URL("./release-cargo-transport.py", import.meta.url));
const keys = new Set(["repo", "candidateSha", "crate", "cargo", "rustc", "getToken", "probeIndex"]);
export function cargoUploadPolicy(options) {
  if (!options || Object.keys(options).some((key) => !keys.has(key))) throw new ReleaseError("unknown Cargo transport option; invocation overrides are forbidden");
  validateTktCrate(options.crate);
  const { repo, candidateSha, crate, cargo, rustc, getToken, probeIndex } = options;
  if (![repo, cargo, rustc].every((path) => typeof path === "string" && isAbsolute(path)) || !/^[a-f0-9]{40}$/.test(candidateSha) || typeof getToken !== "function") throw new ReleaseError("Cargo transport requires absolute paths, the exact candidate and a delayed credential callback");
  let index = "https://github.com/rust-lang/crates.io-index", registry = "crates-io";
  if (probeIndex !== undefined) {
    const match = typeof probeIndex === "string" && probeIndex.match(/^sparse\+http:\/\/127\.0\.0\.1:([1-9]\d{0,4})\/index\/$/);
    if (!match || Number(match[1]) > 65535) throw new ReleaseError("probe registry must be the explicit localhost fixture index");
    index = probeIndex; registry = "release-probe";
  }
  return { schema: 1, repo, candidateSha, name: "tkt", version: crate.version, sha256: crate.sha256, size: crate.bytes.byteLength, cargo, rustc, node: process.execPath, index, registry };
}

export function validateCargoUploadManifest(manifest, policy) {
  if (manifest?.package?.name !== "tkt" || manifest.package.version !== policy.version || !stableVersion(policy.version)) throw new ReleaseError("Cargo manifest differs from the sealed tkt identity");
  if (manifest.patch || manifest.replace || ["members", "default-members", "dependencies", "package"].some((key) => manifest.workspace?.[key] !== undefined)) throw new ReleaseError("external/workspace packaging inputs are unsupported by this tkt transport");
  for (const section of ["dependencies", "dev-dependencies", "build-dependencies"]) {
    const tables = [manifest[section], ...Object.values(manifest.target ?? {}).map((target) => target[section])];
    for (const table of tables) for (const value of Object.values(table ?? {})) if (typeof value === "object" && ["path", "git", "registry", "workspace"].some((key) => value[key] !== undefined)) throw new ReleaseError("Cargo dependencies must use the reviewed crates.io lockfile");
  }
  for (const field of ["readme", "license-file", "build"]) {
    const path = manifest.package[field];
    if (typeof path === "string" && (path.startsWith("/") || path.includes("\\") || path.split("/").includes(".."))) throw new ReleaseError("Cargo package inputs must stay inside the candidate");
  }
}

export async function prepareCargoUpload(options) {
  if (process.platform !== "linux") throw new ReleaseError("prepared Cargo upload requires the verified Linux namespace boundary");
  const policy = cargoUploadPolicy(options), getToken = options.getToken;
  let manifest;
  try {
    const text = execFileSync("/usr/bin/git", ["--no-replace-objects", "-C", policy.repo, "show", `${policy.candidateSha}:Cargo.toml`], { encoding: "utf8", maxBuffer: 2 * 1024 * 1024, env: { PATH: "/usr/bin:/bin", GIT_CONFIG_NOSYSTEM: "1", GIT_CONFIG_GLOBAL: "/dev/null" }, stdio: ["ignore", "pipe", "pipe"] });
    manifest = parseToml(text, { unsafeKeyBehaviour: "throw" });
  } catch { throw new ReleaseError("cannot read the exact candidate Cargo manifest"); }
  validateCargoUploadManifest(manifest, policy);
  const child = spawn("/usr/bin/python3", [helper], { detached: true, env: { PATH: "/usr/bin:/bin", LANG: "C" }, stdio: ["pipe", "pipe", "pipe"] });
  let pending = "", waiting, terminal, attempted = false, closed = false;
  const queue = [];
  const fail = (message) => {
    terminal ??= new ReleaseError(message);
    if (waiting) { waiting.reject(terminal); waiting = undefined; }
  };
  child.stdout.setEncoding("utf8");
  child.stdout.on("data", (data) => {
    pending += data;
    if (Buffer.byteLength(pending) > 262144) { fail("Cargo transport exceeded its protocol limit"); return; }
    for (let newline; (newline = pending.indexOf("\n")) !== -1;) {
      let row;
      try { row = JSON.parse(pending.slice(0, newline)); }
      catch { fail("Cargo transport returned invalid protocol JSON"); return; }
      pending = pending.slice(newline + 1);
      if (row?.status !== "pass") { fail(`Cargo transport failed during ${typeof row?.phase === "string" && /^[a-z-]+$/.test(row.phase) ? row.phase : "unknown phase"}`); return; }
      if (waiting) { const item = waiting; waiting = undefined; item.resolve(row); }
      else if (queue.length < 2) queue.push(row);
      else fail("Cargo transport returned unsolicited protocol records");
    }
  });
  // Child diagnostics are untrusted after a credential is released; never echo them.
  child.stderr.resume();
  child.stdin.on("error", () => fail("Cargo transport input closed"));
  child.on("error", () => fail("Cargo transport could not start"));
  const ended = new Promise((resolve) => child.on("close", (code) => { fail("Cargo transport exited before completing its protocol"); resolve(code); }));
  async function next() {
    if (terminal) throw terminal;
    if (queue.length) return queue.shift();
    let timer;
    try { return await new Promise((resolve, reject) => { waiting = { resolve, reject }; timer = setTimeout(() => fail("Cargo transport phase timed out; inspect registry state before retrying"), 900000); }); }
    finally { clearTimeout(timer); }
  }
  async function close() {
    if (closed) return;
    closed = true;
    if (!child.stdin.destroyed) child.stdin.end(JSON.stringify({ action: "close" }) + "\n");
    let timer, deadline;
    const force = new Promise((_, reject) => {
      timer = setTimeout(() => { if (child.pid) { try { process.kill(-child.pid, "SIGKILL"); } catch (error) { if (error.code !== "ESRCH") fail("Cargo transport cleanup failed"); } } }, 5000);
      deadline = setTimeout(() => reject(new ReleaseError("Cargo transport did not exit after cleanup")), 10000);
    });
    try { await Promise.race([ended, force]); }
    finally { clearTimeout(timer); clearTimeout(deadline); }
  }
  try {
    child.stdin.write(JSON.stringify(policy) + "\n");
    const evidence = await next();
    if (evidence.phase !== "prepared" || evidence.candidateSha !== policy.candidateSha || evidence.sha256 !== policy.sha256 || evidence.size !== policy.size || !/^[a-f0-9]{64}$/.test(evidence.inventorySha256) || !Number.isSafeInteger(evidence.files) || evidence.files < 1) throw new ReleaseError("Cargo preflight evidence differs from the reviewed identity");
    return {
      evidence,
      async publish(crate) {
        if (closed || attempted) throw new ReleaseError("prepared Cargo transport is closed or already attempted");
        validateTktCrate(crate);
        if (crate.sha256 !== policy.sha256 || crate.version !== policy.version || crate.bytes.byteLength !== policy.size) throw new ReleaseError("Cargo attempt differs from its prepared seal");
        attempted = true;
        const token = await getToken();
        if (typeof token !== "string" || !/^[!-~]{1,8192}$/.test(token) || (policy.registry === "release-probe" && token !== "local-probe-only")) throw new ReleaseError("release credential is unavailable or forbidden for a probe");
        if (closed || terminal) throw new ReleaseError("Cargo transport closed before credential release");
        child.stdin.write(JSON.stringify({ action: "publish", token }) + "\n");
        const result = await next();
        if (result.phase !== "published" || result.candidateSha !== policy.candidateSha || result.sha256 !== policy.sha256 || result.exit !== 0) throw new ReleaseError("Cargo attempt was not confirmed; inspect registry state before retrying");
        return result;
      },
      close,
    };
  } catch (error) { await close(); throw error; }
}

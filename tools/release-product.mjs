#!/usr/bin/env node
// Run product checks against an exact clean candidate; never publish or install.
// Usage: node scripts/release-product.mjs --repo DIR --sha FULL_SHA --out NEW_DIR [--execute]
import { closeSync, existsSync, mkdirSync, openSync, readFileSync, realpathSync, writeFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { basename, dirname, isAbsolute, join, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { arch, release, platform } from "node:os";
import { git, gitEnvironment } from "./release-git.mjs";
import { inspectVersions, ReleaseError } from "./release-version.mjs";

const encode = (value) => JSON.stringify(value, null, 2) + "\n";
const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");
function canonical(path) {
  if (existsSync(path)) return realpathSync(path);
  const parent = dirname(path);
  if (parent === path) throw new ReleaseError(`path has no accessible filesystem root: ${path}`);
  return join(canonical(parent), basename(path));
}
export function verifyProduct({ repo, sha, out, execute = false, progress = () => {} }) {
  repo = realpathSync(git(resolve(repo), ["rev-parse", "--show-toplevel"]).trim()); out = canonical(resolve(out));
  if (!/^[a-f0-9]{40}$/.test(sha)) throw new ReleaseError("product verification requires a full candidate commit SHA");
  const clean = () => {
    if (git(repo, ["rev-parse", "HEAD"]).trim() !== sha || git(repo, ["status", "--porcelain=v1", "-z"])) throw new ReleaseError("product candidate must be clean at the requested commit");
  };
  clean();
  const tracked = new Set(git(repo, ["ls-tree", "-r", "--name-only", "-z", sha]).split("\0"));
  const identity = inspectVersions((file) => git(repo, ["show", `${sha}:${file}`]), (file) => tracked.has(file));
  let commands;
  if (identity.name === "grey-matter") commands = [[process.execPath, "scripts/check-repo.mjs", "--require-tools"]];
  else if (identity.name === "tkt" && tracked.has("Cargo.toml")) commands = [["mise", "run", "check"]];
  else throw new ReleaseError(`no product release gate for ${identity.name}`);
  const result = { schema: 1, status: "not-run", mode: execute ? "product-evidence" : "dry-run", publishable: false, candidateSha: sha, product: identity.name, version: identity.version, platform: { os: platform(), release: release(), arch: arch(), node: process.version }, commands, checks: [], pending: ["native", "scope", "compatibility", "remaining-platforms"] };
  if (!execute) return result;
  // Evidence is never an input to product checks or an untracked candidate file.
  const distance = relative(repo, out);
  if ((!distance.startsWith(`..${sep}`) && distance !== ".." && !isAbsolute(distance)) || existsSync(out)) throw new ReleaseError("product evidence requires a new directory outside the candidate checkout");
  mkdirSync(out, { recursive: true });
  for (const [index, args] of commands.entries()) {
    progress(`product ${index + 1}/${commands.length}: ${args.join(" ")}`);
    const log = `${index + 1}-product.log`, fd = openSync(join(out, log), "wx"), started = new Date().toISOString();
    let run;
    try { run = spawnSync(args[0], args.slice(1), { cwd: repo, stdio: ["ignore", fd, fd], windowsHide: true, env: gitEnvironment() }); }
    finally { closeSync(fd); }
    result.checks.push({ command: args, started, finished: new Date().toISOString(), exit: run.status, signal: run.signal, error: run.error?.message ?? null, log, sha256: digest(readFileSync(join(out, log))) });
    result.status = run.error || run.status !== 0 ? "fail" : "pass";
    try { clean(); }
    catch (error) { if (!(error instanceof ReleaseError)) throw error; result.status = "fail"; result.candidateError = error.message; }
    writeFileSync(join(out, "product-evidence.json"), encode(result));
    if (result.status !== "pass") return result;
  }
  return result;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const args = process.argv.slice(2), options = {};
    while (args.length) {
      const key = args.shift();
      if (key === "--execute") options.execute = true;
      else if (["--repo", "--sha", "--out"].includes(key) && args.length) options[key.slice(2)] = args.shift();
      else throw new ReleaseError(`unknown/incomplete option: ${key}`);
    }
    if (!options.repo || !options.sha || !options.out) throw new ReleaseError("usage: --repo DIR --sha FULL_SHA --out NEW_DIR [--execute]");
    options.progress = (message) => console.error(message);
    const result = verifyProduct(options);
    console.log(encode(result)); process.exitCode = result.status === "fail" ? 1 : 0;
  } catch (error) {
    console.error(encode({ status: error instanceof ReleaseError ? "fail" : "error", errors: [error.message] }));
    process.exitCode = error instanceof ReleaseError ? 1 : 2;
  }
}

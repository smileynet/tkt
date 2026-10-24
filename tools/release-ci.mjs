#!/usr/bin/env node
// Validate CI candidate identity and seal previously built release files.
// Usage: node tools/release-ci.mjs identity | collect INPUT OUTPUT
import { appendFileSync, copyFileSync, existsSync, lstatSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { basename, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import * as tar from "tar";
import { parse as parseToml } from "smol-toml";
import { inspectVersions, ReleaseError } from "./release-version.mjs";
import { gitEnvironment } from "./release-git.mjs";
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
export function ciIdentity(repo, env = process.env) {
  const git = (...args) => execFileSync("git", ["-C", repo, ...args], { encoding: "utf8", env: gitEnvironment() }).trim();
  const sha = git("rev-parse", "HEAD");
  if (!/^[a-f0-9]{40}$/.test(env.EXPECTED_CANDIDATE_SHA ?? "") || sha !== env.EXPECTED_CANDIDATE_SHA || sha !== env.GITHUB_SHA || env.GITHUB_REPOSITORY !== "smileynet/tkt") throw new ReleaseError("CI checkout, run and expected candidate must identify smileynet/tkt at the same commit");
  const product = inspectVersions((file) => readFileSync(join(repo, file), "utf8"), (file) => existsSync(join(repo, file)));
  if (product.name !== "tkt" || product.kind !== "cargo") throw new ReleaseError("CI staging requires the tkt Cargo product");
  return { product: "tkt", version: product.version, candidateSha: sha, tag: `v${product.version}` };
}
export function sealStage({ repo, input, output, env = process.env }) {
  const identity = ciIdentity(repo, env), plan = JSON.parse(readFileSync(join(input, "plan-dist-manifest.json"), "utf8"));
  if (plan.dist_version !== "0.32.0" || plan.announcement_tag !== identity.tag || plan.releases?.length !== 1 || plan.releases[0].app_name !== "tkt" || plan.releases[0].app_version !== identity.version) throw new ReleaseError("dist plan identity differs from the CI candidate");
  const names = plan.releases[0].artifacts;
  if (!Array.isArray(names) || !names.length || new Set(names).size !== names.length || names.some((name) => typeof name !== "string" || basename(name) !== name || !/^[A-Za-z0-9][A-Za-z0-9._-]*$/.test(name) || !Object.hasOwn(plan.artifacts ?? {}, name))) throw new ReleaseError("invalid dist asset inventory");
  const configured = parseToml(readFileSync(join(repo, "Cargo.toml"), "utf8")).workspace.metadata.dist.targets;
  const targets = names.flatMap((name) => plan.artifacts[name].kind === "executable-zip" ? plan.artifacts[name].target_triples : []);
  if (JSON.stringify([...targets].sort()) !== JSON.stringify([...configured].sort()) || Object.keys(plan.artifacts).sort().join("\n") !== [...names].sort().join("\n")) throw new ReleaseError("dist plan omits or duplicates configured release assets/targets");
  const crate = `tkt-${identity.version}.crate`, expected = new Set([...names, crate]);
  const files = [], manifests = [];
  for (const name of readdirSync(input).sort()) {
    const path = join(input, name), stat = lstatSync(path);
    if (!stat.isFile() || stat.isSymbolicLink()) throw new ReleaseError(`stage input must be a regular file: ${name}`);
    if (name.endsWith("-dist-manifest.json")) {
      const manifest = JSON.parse(readFileSync(path, "utf8"));
      if (manifest.dist_version !== plan.dist_version || manifest.announcement_tag !== identity.tag || manifest.releases?.length !== 1 || manifest.releases.some((release) => release.app_name !== "tkt" || release.app_version !== identity.version)) throw new ReleaseError(`foreign dist manifest: ${name}`);
      manifests.push({ name, sha256: hash(readFileSync(path)), distVersion: manifest.dist_version, systems: manifest.systems ?? {} });
      continue;
    }
    if (!expected.has(name)) throw new ReleaseError(`unexpected staged asset: ${name}`);
    const bytes = readFileSync(path); if (!bytes.length) throw new ReleaseError(`empty staged asset: ${name}`);
    files.push({ name, size: bytes.length, sha256: hash(bytes) }); expected.delete(name);
  }
  if (expected.size) throw new ReleaseError(`missing staged assets: ${[...expected].join(", ")}`);
  for (const file of files.filter((file) => file.name.endsWith(".sha256"))) {
    const parent = files.find((item) => item.name === file.name.slice(0, -7));
    const line = /^([a-f0-9]{64})(?:[ \t]+\*?([^\r\n]+))?$/.exec(readFileSync(join(input, file.name), "utf8").trim());
    if (!parent || !line || line[1] !== parent.sha256 || (line[2] && line[2] !== parent.name)) throw new ReleaseError(`checksum differs: ${file.name}`);
  }
  const vcsChunks = [], prefix = `tkt-${identity.version}/`;
  tar.list({ file: join(input, crate), sync: true, strict: true, onReadEntry(entry) {
    const name = entry.path.slice(prefix.length);
    if (!entry.path.startsWith(prefix) || entry.type !== "File" || name.split("/").includes("..") || !(name.startsWith("src/") || [".cargo_vcs_info.json", "Cargo.toml", "Cargo.toml.orig", "Cargo.lock", "build.rs", "README.md", "CHANGELOG.md", "LICENSE"].includes(name))) throw new ReleaseError(`unexpected crate entry: ${entry.path}`);
    if (name === ".cargo_vcs_info.json") entry.on("data", (chunk) => vcsChunks.push(chunk));
  } });
  const vcs = JSON.parse(Buffer.concat(vcsChunks).toString("utf8"));
  if (vcs.git?.sha1 !== identity.candidateSha || vcs.git.dirty === true || vcs.path_in_vcs !== "") throw new ReleaseError("crate VCS identity differs from the clean candidate");
  if (!/^\d+$/.test(env.GITHUB_RUN_ID ?? "") || !/^\d+$/.test(env.GITHUB_RUN_ATTEMPT ?? "")) throw new ReleaseError("missing CI run identity");
  if (existsSync(output)) throw new ReleaseError("stage output already exists; use a fresh directory");
  const record = { schema: 1, status: "pass", ...identity, repository: env.GITHUB_REPOSITORY, runId: env.GITHUB_RUN_ID, runAttempt: env.GITHUB_RUN_ATTEMPT, workflow: ".github/workflows/release.yml", manifests, files, publicationPerformed: false };
  mkdirSync(output, { recursive: true });
  for (const file of files) copyFileSync(join(input, file.name), join(output, file.name));
  writeFileSync(join(output, "tkt-stage.json"), JSON.stringify(record, null, 2) + "\n", { flag: "wx" });
  return record;
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const [command, input, output, ...extra] = process.argv.slice(2); let result;
    if (command === "identity" && !input) {
      result = ciIdentity(process.cwd());
      if (!process.env.GITHUB_OUTPUT) throw new ReleaseError("identity requires GITHUB_OUTPUT");
      appendFileSync(process.env.GITHUB_OUTPUT, `tag=${result.tag}\nversion=${result.version}\n`);
    } else if (command === "collect" && input && output && !extra.length) result = sealStage({ repo: process.cwd(), input: resolve(input), output: resolve(output) });
    else throw new ReleaseError("usage: identity | collect INPUT OUTPUT");
    console.log(JSON.stringify(result, null, 2));
  } catch (error) { console.error(JSON.stringify({ status: "fail", errors: [error.message] })); process.exitCode = 1; }
}

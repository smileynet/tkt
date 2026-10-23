#!/usr/bin/env node
// Build/verify closed plugin bundles from a committed source revision; never publish.
// Usage: node scripts/release-artifacts.mjs build --repo DIR --ref SHA --out DIR
//        node scripts/release-artifacts.mjs verify --out DIR
import { chmodSync, cpSync, existsSync, lstatSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, renameSync, rmSync, writeFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { git } from "./release-git.mjs";
import { tmpdir } from "node:os";
import { dirname, isAbsolute, join, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";
import * as tar from "tar";
import { ReleaseError as ArtifactError, inspectVersions } from "./release-version.mjs";
export { ArtifactError };
import { loadPack, lint, agyResources, PackError } from "./deploy.mjs";
import { checkPack } from "./check-skill-syntax.mjs";

const hosts = ["codex", "claude", "agy"];
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
function read(path, encoding) {
  try { return readFileSync(path, encoding); }
  catch (error) {
    if (error.code === "ENOENT") throw new ArtifactError(`missing artifact file: ${path}`);
    throw error;
  }
}
function json(path) {
  try { return JSON.parse(read(path, "utf8")); }
  catch (error) {
    if (error instanceof SyntaxError) throw new ArtifactError(`invalid JSON artifact: ${path}`);
    throw error;
  }
}
const encoded = (value) => JSON.stringify(value, null, 2) + "\n";

function safePath(path) {
  if (!path || isAbsolute(path) || /[\\:\x00-\x1f]/.test(path) || path.replace(/\/$/, "").split("/").some((part) => !part || part === ".." || part === ".")) throw new ArtifactError(`unsafe archive path: ${path}`);
}

function distributable(path) {
  safePath(path);
  if (path.split("/").some((part) => [".git", ".tickets", ".scratch", "node_modules", "target", ".DS_Store", ".npmrc", ".netrc", "credentials.json", "id_rsa", "id_ed25519"].includes(part) || /^\.env(?:\.|$)/.test(part))) throw new ArtifactError(`excluded development/credential path: ${path}`);
}

function extract(file, dest, expected) {
  const files = new Set();
  const entries = new Set();
  let total = 0;
  tar.list({ file, sync: true, strict: true, onReadEntry(entry) {
    safePath(entry.path);
    if (entries.has(entry.path)) throw new ArtifactError(`duplicate archive entry: ${entry.path}`);
    entries.add(entry.path);
    total += entry.size;
    if (!["File", "Directory"].includes(entry.type)) throw new ArtifactError(`unsupported archive entry: ${entry.path} (${entry.type})`);
    if (entry.size > 64 * 1024 * 1024 || total > 256 * 1024 * 1024 || entries.size > 50000) throw new ArtifactError("archive resource limit exceeded");
    if (entry.type === "File") {
      files.add(entry.path);
      if (expected && !Object.hasOwn(expected, entry.path)) throw new ArtifactError(`unexpected archive entry: ${entry.path}`);
    }
  } });
  if (expected && (files.size !== Object.keys(expected).length || Object.keys(expected).some((path) => !files.has(path)))) throw new ArtifactError("archive is missing expected files");
  mkdirSync(dest, { recursive: true });
  tar.extract({ file, cwd: dest, sync: true, strict: true, preserveOwner: false, chmod: true });
  if (expected) for (const path of files) {
    if (sha256(read(join(dest, path))) !== expected[path].sha256) throw new ArtifactError(`extracted file hash differs: ${path}`);
    if (process.platform !== "win32" && (lstatSync(join(dest, path)).mode & 0o111) !== expected[path].executable) throw new ArtifactError(`executable mode differs: ${path}`);
  }
}

export { extract as extractArtifact };

function inventory(dir) {
  const files = {};
  const walk = (at) => {
    for (const name of readdirSync(at).sort()) {
      const path = join(at, name), stat = lstatSync(path);
      const rel = relative(dir, path).split(sep).join("/");
      distributable(rel);
      if (stat.isDirectory()) walk(path);
      else {
        if (!stat.isFile()) throw new ArtifactError(`non-regular package file: ${rel}`);
        files[rel] = { sha256: sha256(read(path)), executable: stat.mode & 0o111 };
      }
    }
  };
  walk(dir);
  return files;
}

function validatePack(dir, host, identity) {
  if (read(join(dir, "README.md"), "utf8").trim().split(/\s+/).length < 40) throw new ArtifactError(`missing or incomplete ${host} plugin README`);
  const manifest = json(join(dir, host === "agy" ? "plugin.json" : `.${host === "codex" ? "codex" : "claude"}-plugin/plugin.json`));
  if (manifest.name !== identity.name || manifest.version !== identity.version) throw new ArtifactError(`wrong ${host} bundle identity/version`);
  const syntax = checkPack(dir);
  if (syntax.status !== "pass" || syntax.metrics.skills !== identity.skills) throw new ArtifactError(`extracted ${host} skill validation failed: ${syntax.errors.join("; ")}`);
  if (host !== "agy") {
    const pack = loadPack(dir);
    const errors = lint(pack);
    if (errors.length) throw new ArtifactError(errors.join("; "));
    agyResources(pack);
  } else {
    const skills = readdirSync(join(dir, "skills")).map((name) => ({ name, src: join(dir, "skills", name) }));
    agyResources({ dir, skills, deploy: {} });
  }
}

export function verifyArtifacts(out) {
  const metadata = json(join(out, "release-metadata.json"));
  if (metadata.schema !== 1 || !/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(metadata.name) || !/^\d+\.\d+\.\d+$/.test(metadata.version) || !/^[a-f0-9]{40}$/.test(metadata.candidateSha) || metadata.assets?.length !== 3) throw new ArtifactError("invalid release metadata identity");
  const expectedNames = ["release-metadata.json", "SHA256SUMS"];
  const temp = mkdtempSync(join(tmpdir(), "plugin-verify-"));
  try {
    for (const host of hosts) {
      const asset = metadata.assets.find((item) => item.host === host);
      const name = `${metadata.name}-${metadata.version}-${host}.tar.gz`;
      if (!asset || asset.name !== name || !asset.files || !Object.keys(asset.files).length) throw new ArtifactError(`invalid ${host} asset metadata`);
      expectedNames.push(name);
      for (const path of Object.keys(asset.files)) distributable(path);
      if (sha256(read(join(out, name))) !== asset.sha256) throw new ArtifactError(`stale or corrupted asset: ${name}`);
      const dest = join(temp, host);
      extract(join(out, name), dest, asset.files);
      validatePack(dest, host, metadata);
    }
    if (encoded(readdirSync(out).sort()) !== encoded(expectedNames.sort())) throw new ArtifactError("unexpected release output entries");
    const sums = [...metadata.assets.map((asset) => `${asset.sha256}  ${asset.name}`), `${sha256(read(join(out, "release-metadata.json")))}  release-metadata.json`].join("\n") + "\n";
    if (read(join(out, "SHA256SUMS"), "utf8") !== sums) throw new ArtifactError("checksum index differs from release identity");
    return metadata;
  } finally { rmSync(temp, { recursive: true, force: true }); }
}

export function buildArtifacts({ repo, ref, out, compatibility = { status: "not-run", pairs: [] } }) {
  const candidateSha = git(repo, ["rev-parse", "--verify", "--end-of-options", `${ref}^{commit}`]).trim();
  const temp = mkdtempSync(join(tmpdir(), "plugin-bundle-"));
  try {
    const sourceTar = join(temp, "source.tar");
    writeFileSync(sourceTar, git(repo, ["archive", "--format=tar", candidateSha], { encoding: null, maxBuffer: 128 * 1024 * 1024 }));
    const source = join(temp, "source");
    extract(sourceTar, source);
    const pack = loadPack(source);
    const errors = lint(pack);
    if (errors.length) throw new ArtifactError(errors.join("; "));
    const versionState = inspectVersions((file) => read(join(source, file), "utf8"), (file) => existsSync(join(source, file)));
    const identity = { name: versionState.name, version: versionState.version, skills: pack.skills.length };
    const resources = agyResources(pack);
    const staged = join(temp, "assets");
    mkdirSync(staged);
    const metadata = { schema: 1, ...identity, candidateSha, tag: `v${identity.version}`, compatibility, verification: { structural: "pass", native: "not-run", platforms: [] }, assets: [] };
    for (const host of hosts) {
      const tree = join(temp, host);
      mkdirSync(tree);
      if (host === "agy") {
        const script = ["scripts/deploy.mjs", "tools/deploy.mjs"].find((path) => existsSync(join(source, path)));
        if (!script) throw new ArtifactError("source has no canonical deploy engine");
        execFileSync(process.execPath, [join(source, script), "agy", "build", "--out", tree], { encoding: "utf8" });
      } else {
        const files = new Set(resources.map((path) => relative(source, path).split(sep).join("/")));
        for (const root of [".claude-plugin/plugin.json", ".codex-plugin/plugin.json", ".agents/plugins/marketplace.json", ".claude-plugin/marketplace.json"]) if (existsSync(join(source, root))) files.add(root);
        for (const path of [...files].sort()) {
          distributable(path);
          const dest = join(tree, path);
          mkdirSync(dirname(dest), { recursive: true });
          cpSync(join(source, path), dest);
          chmodSync(dest, lstatSync(join(source, path)).mode & 0o777);
        }
      }
      let install = "agy plugin install /absolute/path/to/extracted-plugin";
      if (host !== "agy") {
        const catalog = host === "codex" && existsSync(join(tree, ".agents/plugins/marketplace.json")) ? ".agents/plugins/marketplace.json" : ".claude-plugin/marketplace.json";
        const market = json(join(tree, catalog)).name;
        if (!/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(market)) throw new ArtifactError(`invalid ${host} marketplace name`);
        install = `${host} plugin marketplace add /absolute/path/to/extracted-plugin\n${host} plugin ${host === "codex" ? "add" : "install"} ${identity.name}@${market}`;
      }
      const helpers = pack.deploy.runtimeFiles?.includes("package.json") ? "\n## Maintainer helpers\n\nBundled maintenance helpers need Node 22+, Git and their pinned Node dependencies. Run `npm ci --ignore-scripts` in a disposable extracted copy before invoking them. Preparation and product checks take an explicit --repo pointing to a clean Git checkout. Installing this plugin alone does not prepare or publish a release.\n" : "";
      writeFileSync(join(tree, "README.md"), `# ${identity.name}\n\n${pack.manifest.description}\n\nThis ${host} plugin archive contains version ${identity.version}, with ${identity.skills} workflow skills and their supporting resources. It was built from commit ${candidateSha}.\n\n## Install\n\nExtract this archive into its own directory, then use the host's native plugin installer:\n\n\`\`\`sh\n${install}\n\`\`\`\n\nRestart or reload the host after installing. Read each bundled SKILL.md for workflow-specific prerequisites; install any required command-line tools separately. This archive supplies skills and references, not third-party executables. Retain all bundled license and attribution notices when redistributing it.\n${helpers}`);
      const files = inventory(tree);
      const name = `${identity.name}-${identity.version}-${host}.tar.gz`;
      tar.create({ cwd: tree, file: join(staged, name), sync: true, gzip: true, portable: true, noMtime: true, strict: true, noDirRecurse: true }, Object.keys(files).sort());
      metadata.assets.push({ host, name, sha256: sha256(read(join(staged, name))), files });
    }
    writeFileSync(join(staged, "release-metadata.json"), encoded(metadata));
    writeFileSync(join(staged, "SHA256SUMS"), [...metadata.assets.map((asset) => `${asset.sha256}  ${asset.name}`), `${sha256(read(join(staged, "release-metadata.json")))}  release-metadata.json`].join("\n") + "\n");
    verifyArtifacts(staged);
    if (existsSync(out)) {
      verifyArtifacts(out);
      for (const name of readdirSync(staged)) if (!read(join(out, name)).equals(read(join(staged, name)))) throw new ArtifactError(`existing output belongs to different source, compatibility or artifact bytes: ${name}`);
    } else {
      mkdirSync(dirname(out), { recursive: true });
      const pending = mkdtempSync(join(dirname(out), ".release-artifacts-"));
      try { cpSync(staged, pending, { recursive: true }); renameSync(pending, out); }
      finally { rmSync(pending, { recursive: true, force: true }); }
    }
    return metadata;
  } finally { rmSync(temp, { recursive: true, force: true }); }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const args = process.argv.slice(2), mode = args.shift(), options = {};
    while (args.length) {
      const key = args.shift();
      if (!["--repo", "--ref", "--out"].includes(key) || !args.length) throw new ArtifactError(`unknown/incomplete option: ${key}`);
      options[key.slice(2)] = args.shift();
    }
    if (!options.out || !["build", "verify"].includes(mode) || (mode === "build" && (!options.repo || !options.ref))) throw new ArtifactError("usage: build --repo DIR --ref SHA --out DIR | verify --out DIR");
    options.out = resolve(options.out);
    const metadata = mode === "build" ? buildArtifacts(options) : verifyArtifacts(options.out);
    console.log(encoded({ status: "pass", metrics: { assets: metadata.assets.length, skills: metadata.skills }, candidateSha: metadata.candidateSha, version: metadata.version, output: options.out, nativeVerification: metadata.verification.native }));
  } catch (error) {
    const validation = error instanceof ArtifactError || error instanceof PackError;
    console.error(encoded({ status: validation ? "fail" : "error", metrics: {}, errors: [error.message] }));
    process.exitCode = validation ? 1 : 2;
  }
}

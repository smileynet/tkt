#!/usr/bin/env node
// Validate and locally load sealed plugin bundles in disposable native harness homes.
// Usage: node scripts/release-native.mjs --artifacts DIR --out NEW_DIR [--execute]
//        [--codex BINARY] [--claude BINARY] [--agy BINARY]
import { existsSync, mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { execFile, spawn } from "node:child_process";
import { promisify } from "node:util";
import { createInterface } from "node:readline";
import { tmpdir, arch, release } from "node:os";
import { basename, dirname, join, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { parse } from "yaml";
import { verifyArtifacts, extractArtifact } from "./release-artifacts.mjs";
import { ReleaseError } from "./release-version.mjs";
const executeFile = promisify(execFile), here = dirname(fileURLToPath(import.meta.url));
const encode = (value) => JSON.stringify(value, null, 2) + "\n";
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const profile = '(version 1)(allow default)(deny network*)(allow network-outbound (remote ip "localhost:*"))(allow network-inbound (local ip "localhost:*"))';
function canonical(path) {
  if (existsSync(path)) return realpathSync(path);
  const parent = dirname(path);
  if (parent === path) throw new ReleaseError(`path has no accessible filesystem root: ${path}`);
  return join(canonical(parent), basename(path));
}
export function isolatedEnvironment(home) {
  const env = { HOME: home, USERPROFILE: home, XDG_CONFIG_HOME: join(home, ".config"), CODEX_HOME: join(home, ".codex"), CLAUDE_CONFIG_DIR: join(home, ".claude"), DISABLE_UPDATES: "1", CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC: "1" };
  for (const key of ["PATH", "SystemRoot", "WINDIR", "COMSPEC", "PATHEXT", "TMP", "TEMP"]) if (process.env[key]) env[key] = process.env[key];
  return env;
}
function command(binary, args) {
  return process.platform === "darwin" ? { binary: "/usr/bin/sandbox-exec", args: ["-p", profile, binary, ...args] } : { binary, args };
}
export function requireNativeSuccess(result) {
  const checks = result.runs;
  if (result.status !== "pass" || checks?.length !== 3 || !checks.every((check) => check.exit === 0 && check.result?.success && !check.result.directory?.incomplete && !check.result.directory?.failure && !(check.result.directory?.findings ?? []).some((finding) => finding.blocking)) || !result.seed?.detected || !result.readmeControl?.detected) throw new ReleaseError("Claude native validation or seeded controls failed");
}
export function requireExactNames(actual, expected) {
  if (encode([...actual].sort()) !== encode([...expected].sort())) throw new ReleaseError("native skill names differ from sealed artifact, or skills load more than once");
}
async function catalog(binary, env, cwd) {
  const wrapped = command(binary, ["app-server"]), child = spawn(wrapped.binary, wrapped.args, { env, cwd, stdio: ["pipe", "pipe", "pipe"], windowsHide: true });
  const lines = createInterface({ input: child.stdout }), pending = new Map();
  let stderr = "", ended = false;
  child.stderr.on("data", (data) => { stderr = (stderr + data).slice(-32000); });
  const finish = (error) => { ended = true; for (const { reject } of pending.values()) reject(error); pending.clear(); };
  child.on("error", (error) => finish(error));
  child.stdin.on("error", (error) => finish(error));
  child.on("exit", () => finish(new ReleaseError(`Codex app-server exited: ${stderr}`)));
  lines.on("line", (line) => {
    let message;
    try { message = JSON.parse(line); }
    catch { finish(new ReleaseError("Codex app-server emitted invalid JSON")); return; }
    const request = pending.get(message.id);
    if (request) { pending.delete(message.id); if (message.error) request.reject(new ReleaseError(encode(message.error))); else request.resolve(message.result); }
  });
  const rpc = (id, method, params) => new Promise((resolve, reject) => {
    if (ended) { reject(new ReleaseError("Codex app-server is no longer running")); return; }
    const timer = setTimeout(() => { pending.delete(id); reject(new ReleaseError(`Codex ${method} timed out`)); }, 25000);
    pending.set(id, { resolve: (value) => { clearTimeout(timer); resolve(value); }, reject: (error) => { clearTimeout(timer); reject(error); } });
    child.stdin.write(JSON.stringify({ id, method, params }) + "\n");
  });
  try {
    await rpc(1, "initialize", { clientInfo: { name: "release-native", version: "1.0.0" } });
    child.stdin.write('{"method":"initialized"}\n');
    return await rpc(2, "skills/list", { cwds: [cwd], forceReload: true });
  } finally {
    lines.close();
    if (child.pid && child.exitCode === null && child.signalCode === null) {
      const exited = new Promise((resolve) => child.once("exit", resolve)); child.kill();
      const timer = setTimeout(() => child.kill("SIGKILL"), 3000);
      await exited; clearTimeout(timer);
    }
  }
}

export async function verifyNative({ artifacts, out, execute = false, codex = "codex", claude = "claude", agy = "agy", progress = () => {} }) {
  artifacts = realpathSync(resolve(artifacts)); out = canonical(resolve(out));
  const metadata = verifyArtifacts(artifacts);
  const result = { schema: 1, status: "not-run", mode: execute ? "native-evidence" : "dry-run", publishable: false, candidateSha: metadata.candidateSha, product: metadata.name, version: metadata.version, platform: { os: process.platform, release: release(), arch: arch() }, isolation: { homes: "disposable", credentials: "environment allowlist; no user configuration", network: process.platform === "darwin" ? "loopback-only sandbox" : "not-enforced" }, plan: metadata.assets.map((asset) => ({ host: asset.host, asset: asset.name, sha256: asset.sha256, binary: { codex, claude, agy }[asset.host] })), rows: [], pending: ["product", "scope", "compatibility", "upgrade", "remaining-platforms"] };
  if (!execute) return result;
  if (existsSync(out) || out === artifacts || out.startsWith(artifacts + sep)) throw new ReleaseError("native evidence needs a new directory outside sealed artifacts");
  result.started = new Date().toISOString();
  result.tooling = Object.fromEntries(["release-native.mjs", "release-artifacts.mjs", "probe-claude-validation.mjs"].map((name) => [name, hash(readFileSync(join(here, name)))]));
  mkdirSync(out, { recursive: true });
  const temporary = mkdtempSync(join(tmpdir(), "release-native-"));
  try {
    for (const asset of metadata.assets) {
      progress(`${metadata.name}/${asset.host}`);
      const pack = join(temporary, asset.host), home = join(temporary, `${asset.host}-home`), workspace = join(home, "workspace");
      extractArtifact(join(artifacts, asset.name), pack, asset.files); mkdirSync(workspace, { recursive: true });
      const env = isolatedEnvironment(home), expected = Object.keys(asset.files).filter((file) => file.endsWith("/SKILL.md")).map((file) => basename(dirname(file))).sort();
      const row = { host: asset.host, asset: asset.name, assetSha256: asset.sha256, status: "fail", expectedSkills: expected, checks: [] };
      result.rows.push(row);
      const run = async (binary, args) => {
        const wrapped = command(binary, args);
        // Native failures retain their output in the evidence before stopping.
        try {
          const output = await executeFile(wrapped.binary, wrapped.args, { env, cwd: workspace, encoding: "utf8", timeout: 30000, maxBuffer: 8 * 1024 * 1024, windowsHide: true });
          row.checks.push({ command: [binary, ...args], exit: 0, ...output }); return output.stdout;
        } catch (error) {
          row.checks.push({ command: [binary, ...args], exit: error.code, signal: error.signal, stdout: error.stdout, stderr: error.stderr });
          throw new ReleaseError(`native command failed: ${binary} ${args[0]}: ${error.message}`);
        }
      };
      const binary = { codex, claude, agy }[asset.host];
      row.nativeVersion = (await run(binary, ["--version"])).trim();
      if (asset.host === "codex") {
        mkdirSync(env.CODEX_HOME);
        const market = JSON.parse(readFileSync(join(pack, ".agents/plugins/marketplace.json"))).name;
        await run(codex, ["plugin", "marketplace", "add", pack, "--json"]);
        const installed = JSON.parse(await run(codex, ["plugin", "add", `${metadata.name}@${market}`, "--json"]));
        const cache = realpathSync(installed.installedPath);
        if (!cache.startsWith(realpathSync(home) + sep)) throw new ReleaseError("Codex install escaped the disposable home");
        for (const [file, record] of Object.entries(asset.files)) if (hash(readFileSync(join(cache, file))) !== record.sha256) throw new ReleaseError(`Codex installed bytes differ: ${file}`);
        const response = await catalog(codex, env, workspace);
        const errors = response.data.flatMap((group) => group.errors);
        if (errors.length) throw new ReleaseError(`Codex skill catalog errors: ${encode(errors)}`);
        const loaded = response.data.flatMap((group) => group.skills).filter((skill) => realpathSync(skill.path).startsWith(cache + sep));
        requireExactNames(loaded.map((skill) => basename(dirname(skill.path))), expected);
        row.catalog = { skills: loaded.map((skill) => ({ name: skill.name, path: skill.path })), errors, installedFilesMatched: Object.keys(asset.files).length };
      } else if (asset.host === "claude") {
        const probe = JSON.parse(await run(process.execPath, [join(here, "probe-claude-validation.mjs"), "--pack", pack, "--snapshot", "--binary", claude]));
        requireNativeSuccess(probe); requireExactNames(probe.hashes.map((skill) => skill.name), expected);
        row.seededProbe = probe;
        const details = await run(claude, ["--plugin-dir", pack, "plugin", "details", metadata.name]);
        if (Number(/Skills \((\d+)\)/.exec(details)?.[1]) !== expected.length) throw new ReleaseError("Claude inline discovery count differs from artifact");
      } else {
        const validation = JSON.parse(await run(agy, ["plugin", "validate", pack, "--strict", "--json"]));
        if (validation.components?.skills !== expected.length || validation.findings?.length) throw new ReleaseError("agy native validation failed or omitted skills");
        await run(agy, ["plugin", "install", pack]);
        mkdirSync(join(home, ".gemini/antigravity-cli"), { recursive: true });
        writeFileSync(join(home, ".gemini/antigravity-cli/settings.json"), '{"modelProvider":"gemini"}\n');
        env.GEMINI_API_KEY = "release-probe-dummy-not-a-real-key"; env.GOOGLE_GEMINI_BASE_URL = "http://127.0.0.1:9";
        const listing = JSON.parse(await run(agy, ["--output-format", "json", "--print", "/skills"]));
        if (listing.num_turns !== 0) throw new ReleaseError("agy catalog unexpectedly used a model turn");
        const loaded = listing.command.data.skills.filter((skill) => skill.plugin === metadata.name);
        requireExactNames(loaded.map((skill) => skill.name), expected.map((name) => `${metadata.name}:${name}`));
        for (const name of expected) {
          const text = readFileSync(join(pack, "skills", name, "SKILL.md"), "utf8"), fields = parse(text.split(/^---\s*$/m)[1]);
          if (loaded.find((skill) => skill.name === `${metadata.name}:${name}`).model_invocable !== !fields["disable-model-invocation"]) throw new ReleaseError(`agy invocation policy differs: ${name}`);
        }
        const cache = realpathSync(dirname(dirname(dirname(loaded[0].path))));
        if (!cache.startsWith(realpathSync(home) + sep)) throw new ReleaseError("agy install escaped the disposable home");
        for (const [file, record] of Object.entries(asset.files)) if (hash(readFileSync(join(cache, file))) !== record.sha256) throw new ReleaseError(`agy installed bytes differ: ${file}`);
        row.catalog = { skills: loaded, numTurns: listing.num_turns, invocationPoliciesMatched: true, installedFilesMatched: Object.keys(asset.files).length };
      }
      row.status = "pass";
      writeFileSync(join(out, "native-evidence.json"), encode(result));
    }
    result.status = "pass";
  } catch (error) {
    result.status = "fail"; result.errors = [error.message];
  } finally {
    result.finished = new Date().toISOString();
    try { writeFileSync(join(out, "native-evidence.json"), encode(result)); }
    finally { rmSync(temporary, { recursive: true, force: true }); }
  }
  return result;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const args = process.argv.slice(2), options = {};
    while (args.length) {
      const key = args.shift();
      if (key === "--execute") options.execute = true;
      else if (["--artifacts", "--out", "--codex", "--claude", "--agy"].includes(key) && args.length) options[key.slice(2)] = args.shift();
      else throw new ReleaseError(`unknown/incomplete option: ${key}`);
    }
    if (!options.artifacts || !options.out) throw new ReleaseError("usage: --artifacts DIR --out NEW_DIR [--execute] [--HOST BINARY]");
    options.progress = (phase) => console.error(`native: ${phase}`);
    const result = await verifyNative(options); console.log(encode(result)); process.exitCode = result.status === "fail" ? 1 : 0;
  } catch (error) { const expected = error instanceof ReleaseError; console.error(encode({ status: expected ? "fail" : "error", errors: [error.message] })); process.exitCode = expected ? 1 : 2; }
}

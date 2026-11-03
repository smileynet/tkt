import assert from "node:assert/strict";
import { chmodSync, copyFileSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { execFileSync, spawnSync } from "node:child_process";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

const tools = dirname(fileURLToPath(import.meta.url));
function fixture(run) {
  const root = mkdtempSync(join(tmpdir(), "tkt binary freshness "));
  try {
    const repo = join(root, "repo with spaces");
    const bin = join(root, "bin with spaces");
    mkdirSync(repo);
    mkdirSync(bin);
    const fixtureTools = join(repo, "tools");
    mkdirSync(fixtureTools);
    copyFileSync(join(tools, "check-binary-fresh.sh"), join(fixtureTools, "check-binary-fresh.sh"));
    copyFileSync(join(tools, "check-binary-fresh.mjs"), join(fixtureTools, "check-binary-fresh.mjs"));
    const git = (...args) => execFileSync("git", ["-C", repo, ...args], { encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] }).trim();
    git("init", "-q");
    git("-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit", "--allow-empty", "-m", "fixture", "-q");
    return run({ root, repo, bin, git });
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

function installFixtureTkt(bin, version) {
  const script = join(bin, process.platform === "win32" ? "tkt.cmd" : "tkt");
  if (process.platform === "win32") {
    writeFileSync(script, `@echo off\necho tkt ${version}\n`);
  } else {
    writeFileSync(script, `#!/bin/sh\nprintf '%s\\n' 'tkt ${version}'\n`);
    chmodSync(script, 0o755);
  }
  return script;
}

function invoke(repo, bin = "", options = {}) {
  const env = { ...process.env, DO_NOT_TRACK: "1" };
  const pathKey = process.platform === "win32" && Object.hasOwn(env, "Path") ? "Path" : "PATH";
  const currentPath = env[pathKey] ?? env.PATH ?? "";
  env[pathKey] = bin ? `${bin}${process.platform === "win32" ? ";" : ":"}${currentPath}` : "";
  const command = "bash";
  const wrapper = join(repo, "tools", "check-binary-fresh.sh");
  const args = [wrapper];
  return spawnSync(command, args, {
    cwd: repo,
    encoding: "utf8",
    env,
    ...options,
  });
}

function invokeNode(repo, path, options = {}) {
  const env = { ...process.env, DO_NOT_TRACK: "1" };
  const pathKey = process.platform === "win32" && Object.hasOwn(env, "Path") ? "Path" : "PATH";
  env[pathKey] = path;
  return spawnSync(process.execPath, [join(repo, "tools", "check-binary-fresh.mjs")], {
    cwd: repo,
    encoding: "utf8",
    env,
    ...options,
  });
}

test("freshness wrapper reports a matching installed hash as fresh", () => fixture(({ repo, bin, git }) => {
  installFixtureTkt(bin, `0.4.0 (${git("rev-parse", "--short", "HEAD")})`);
  const result = invoke(repo, bin);
  assert.equal(result.status, 0, `${result.stdout} ${result.stderr}`);
  assert.equal(JSON.parse(result.stdout).status, "fresh");
}));

test("freshness wrapper reports a different installed hash as stale", () => fixture(({ repo, bin }) => {
  installFixtureTkt(bin, "0.4.0 (0000000)");
  const result = invoke(repo, bin);
  assert.equal(result.status, 1, `${result.stdout} ${result.stderr}`);
  assert.equal(JSON.parse(result.stdout).status, "stale");
}));

test("freshness wrapper returns operational error JSON when tkt is missing", () => fixture(({ repo }) => {
  const gitPath = execFileSync(process.platform === "win32" ? "where.exe" : "which", ["git"], { encoding: "utf8" }).trim().split(/\r?\n/)[0];
  const path = [dirname(process.execPath), dirname(gitPath)].join(process.platform === "win32" ? ";" : ":");
  const result = invokeNode(repo, path);
  assert.equal(result.status, 2);
  assert.equal(JSON.parse(result.stdout).status, "error");
}));

test("freshness wrapper returns operational error JSON outside a git repository", () => {
  const root = mkdtempSync(join(tmpdir(), "tkt binary outside git "));
  try {
    const fixtureTools = join(root, "tools");
    mkdirSync(fixtureTools);
    copyFileSync(join(tools, "check-binary-fresh.sh"), join(fixtureTools, "check-binary-fresh.sh"));
    copyFileSync(join(tools, "check-binary-fresh.mjs"), join(fixtureTools, "check-binary-fresh.mjs"));
    const gitPath = execFileSync(process.platform === "win32" ? "where.exe" : "which", ["git"], { encoding: "utf8" }).trim().split(/\r?\n/)[0];
    const path = [dirname(process.execPath), dirname(gitPath)].join(process.platform === "win32" ? ";" : ":");
    const result = invokeNode(root, path);
    assert.equal(result.status, 2);
    assert.equal(JSON.parse(result.stdout).status, "error");
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

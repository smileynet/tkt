#!/usr/bin/env node
// Run cheap tooling fixtures plus source-selected native regressions; --release runs both.
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repo = fileURLToPath(new URL("../", import.meta.url));
const shared = ["package.json", "package-lock.json"];
const identity = ["build.rs", "Cargo.toml", "Cargo.lock", ".cargo/", ".cargo_vcs_info.json", "src/cli.rs", "tools/build-identity.test.mjs"];
const generation = ["Cargo.toml", "Cargo.lock", "dist-workspace.toml", ".github/workflows/release.yml", "tools/generate-release-workflow.mjs", "tools/release-generation.test.mjs"];
const matches = (path, scopes) => scopes.some(scope => scope.endsWith("/") ? path.startsWith(scope) : path === scope);

export function selectChecks(paths, release = false) {
  const all = release || paths === null || paths.some(path => matches(path, shared));
  return {
    identity: all || paths.some(path => matches(path, identity)),
    generation: all || paths.some(path => matches(path, generation)),
  };
}

export function changedPaths(root, base) {
  const env = Object.fromEntries(Object.entries(process.env).filter(([key]) => !/^GIT_/i.test(key)));
  const git = (...args) => {
    const result = spawnSync("git", ["--no-optional-locks", "--no-replace-objects", "-C", root, ...args], { encoding: "utf8", env });
    if (result.error || result.status !== 0) throw new Error(result.error?.message ?? result.stderr.trim());
    return result.stdout;
  };
  let from;
  if (base) from = git("rev-parse", "--verify", `${base}^{commit}`).trim();
  else {
    try { from = git("merge-base", "HEAD", "origin/main").trim(); }
    catch { return null; }
  }
  return [...new Set([
    ...git("diff", "--name-only", "-z", "--no-renames", from, "HEAD").split("\0"),
    ...git("diff", "--name-only", "-z", "--no-renames", "HEAD").split("\0"),
    ...git("ls-files", "--others", "--exclude-standard", "-z").split("\0"),
  ].filter(Boolean))];
}

export function executionPlan(paths, release = false) {
  const selected = selectChecks(paths, release);
  const files = ["tools/release-tools.test.mjs", "tools/release-ci.test.mjs", "tools/check-binary-fresh.test.mjs", "tools/check-tools.test.mjs", "tools/kimi-plugin.test.mjs"];
  if (selected.identity) files.push("tools/build-identity.test.mjs");
  if (selected.generation) files.push("tools/release-generation.test.mjs");
  return { status: "selected", scope: release ? "release" : paths === null ? "unknown" : "changes", selected, files };
}

export function main(args) {
  let base, release = false, plan = false;
  while (args.length) {
    const arg = args.shift();
    if (arg === "--base" && args.length) base = args.shift();
    else if (arg === "--release") release = true;
    else if (arg === "--plan") plan = true;
    else throw new Error("usage: check-tools.mjs [--base REF] [--release] [--plan]");
  }
  const scope = executionPlan(release ? null : changedPaths(repo, base), release);
  console.log(JSON.stringify(scope));
  if (plan) return 0;
  const result = spawnSync(process.execPath, ["--test", ...scope.files], { cwd: repo, stdio: "inherit", env: { ...process.env, DO_NOT_TRACK: "1" } });
  if (result.error) throw result.error;
  return result.status ?? 2;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try { process.exitCode = main(process.argv.slice(2)); }
  catch (error) { console.error(JSON.stringify({ status: "error", errors: [error.message] })); process.exitCode = 2; }
}

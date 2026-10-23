import { test } from "node:test";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { cpSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { execFileSync, spawnSync } from "node:child_process";
import { dirname, join } from "node:path";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";
const tools = dirname(fileURLToPath(import.meta.url)), repo = dirname(tools);
const sha = (bytes) => createHash("sha256").update(bytes).digest("hex");

test("vendored release files match their reviewed ownership inventory", () => {
  const inventory = JSON.parse(readFileSync(join(tools, "release-vendor.json")));
  assert.equal(inventory.owner, "smileynet/grey-matter-release-tools");
  assert.equal(Object.keys(inventory.files).length, 13);
  assert.ok(inventory.files["tools/release-publish.mjs"]);
  assert.deepEqual(JSON.parse(readFileSync(join(repo, "package.json"))).devDependencies, inventory.dependencies);
  for (const [file, expected] of Object.entries(inventory.files)) {
    assert.match(file, /^tools\/[a-zA-Z0-9.-]+$/); assert.equal(sha(readFileSync(join(repo, file))), expected, file);
  }
});
test("standalone publication core refuses a competing local tkt publisher", async () => {
  const { reconcileGithubRelease } = await import("./release-publish.mjs");
  await assert.rejects(reconcileGithubRelease({ candidate: { product: "tkt", version: "0.4.0", candidateSha: "a".repeat(40), repository: "example/tkt" }, publisher: "local", api: {} }), /sole publisher/);
});
test("release entry points contain no legacy publisher or implicit bump path", () => {
  const shell = readFileSync(join(tools, "release.sh"), "utf8"), mise = readFileSync(join(repo, "mise.toml"), "utf8");
  assert.doesNotMatch(shell + mise, /cargo release|gh release create|git push|release:patch|release:minor|release:major/);
  assert.match(shell, /prepare\|verify\|verify-product\|verify-native/);
  const failed = spawnSync("bash", [join(tools, "release.sh"), "patch", "--execute"], { encoding: "utf8" }); assert.equal(failed.status, 2); assert.match(failed.stderr, /legacy publication is disabled/);
});
test("Node dependencies are pinned maintainer prerequisites, outside Rust dependencies", () => {
  const pkg = JSON.parse(readFileSync(join(repo, "package.json"))), lock = JSON.parse(readFileSync(join(repo, "package-lock.json")));
  assert.equal(pkg.private, true); assert.equal(pkg.engines.node, ">=22"); assert.equal(lock.name, pkg.name);
  for (const [name, version] of Object.entries(pkg.devDependencies)) { assert.match(version, /^\d+\.\d+\.\d+$/); assert.equal(lock.packages[`node_modules/${name}`].version, version); }
});
test("standalone Node, bash and mise preparation inspect without output or version changes", () => {
  const temp = mkdtempSync(join(tmpdir(), "tkt release entry ")), candidate = join(temp, "source with spaces"), output = join(temp, "output with spaces");
  const git = (...args) => execFileSync("git", ["-C", candidate, ...args], { encoding: "utf8", stdio: "pipe" }).trim();
  try {
    mkdirSync(candidate); git("init", "-b", "main"); git("config", "user.name", "Fixture"); git("config", "user.email", "fixture@example.invalid"); git("config", "core.autocrlf", "false");
    for (const file of ["Cargo.toml", "Cargo.lock", ".claude-plugin/plugin.json", ".codex-plugin/plugin.json", "plugin.json"]) { mkdirSync(dirname(join(candidate, file)), { recursive: true }); cpSync(join(repo, file), join(candidate, file)); }
    writeFileSync(join(candidate, "CHANGELOG.md"), "# Changelog\n\n## [Unreleased]\n\n### Added\n- Fixture change.\n");
    git("add", "."); git("-c", `core.hooksPath=${join(temp, "no hooks")}`, "commit", "-m", "Fixture");
    const before = git("rev-parse", "HEAD"), version = "999.0.0", args = ["prepare", "--repo", candidate, "--version", version, "--out", output];
    const cases = [[process.execPath, [join(tools, "release.mjs"), ...args]], ["bash", [join(tools, "release.sh"), ...args]], ["mise", ["run", "release", "--", ...args]]];
    for (const [binary, invocation] of cases) {
      const result = spawnSync(binary, invocation, { cwd: repo, encoding: "utf8", env: { ...process.env, MISE_TRUSTED_CONFIG_PATHS: repo, MISE_STATE_DIR: join(temp, "mise-state"), MISE_CACHE_DIR: join(temp, "mise-cache") } });
      assert.equal(result.status, 0, `${binary}: ${result.stdout}${result.stderr}`); const report = JSON.parse(result.stdout); assert.equal(report.mode, "dry-run"); assert.equal(report.publishable, false); assert.equal(report.sourceSha, before); assert.equal(existsSync(output), false); assert.equal(git("rev-parse", "HEAD"), before); assert.equal(git("status", "--porcelain"), ""); assert.equal(git("tag", "--list"), "");
    }
  } finally { rmSync(temp, { recursive: true, force: true }); }
});

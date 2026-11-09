import "./deploy-validation.test.mjs";
import { test } from "node:test";
import "./claude-hooks.test.mjs";
import "./migrate-legacy-skills.test.mjs";
import "./release-publisher.test.mjs";
import { parse as parseToml } from "smol-toml";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { cpSync, existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { execFileSync, spawnSync } from "node:child_process";
import { dirname, join } from "node:path";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";
import { credentialReply } from "./release-cargo-credentials.mjs";
import { cargoUploadPolicy, validateCargoUploadManifest } from "./release-cargo-upload.mjs";
import { removeTree } from "./test-fixtures.mjs";

const tools = dirname(fileURLToPath(import.meta.url)), repo = dirname(tools);
const sha = (bytes) => createHash("sha256").update(bytes).digest("hex");

test("vendored release files match their reviewed ownership inventory", () => {
  const inventory = JSON.parse(readFileSync(join(tools, "release-vendor.json")));
  assert.equal(inventory.owner, "smileynet/grey-matter-release-tools");
  assert.equal(Object.keys(inventory.files).length, 22);
  assert.ok(inventory.files["tools/release-publish.mjs"]);
  assert.ok(inventory.files["tools/release-ci-artifacts.mjs"]);
  assert.ok(inventory.files["tools/release-tkt-publication.mjs"]);
  assert.ok(inventory.files["tools/release-tkt-ci.mjs"]);
  assert.ok(inventory.files["tools/release-cargo-credentials.mjs"]);
  assert.ok(inventory.files["tools/release-cargo-upload.mjs"]);
  assert.ok(inventory.files["tools/release-cargo-transport.py"]);
  assert.deepEqual(JSON.parse(readFileSync(join(repo, "package.json"))).devDependencies, inventory.dependencies);
  for (const [file, expected] of Object.entries(inventory.files)) {
    assert.match(file, /^tools\/[a-zA-Z0-9.-]+$/); assert.equal(sha(readFileSync(join(repo, file))), expected, file);
  }
});
test("standalone publication core refuses a competing local tkt publisher", async () => {
  const { reconcileGithubRelease } = await import("./release-publish.mjs");
  await assert.rejects(reconcileGithubRelease({ candidate: { product: "tkt", version: "0.4.0", candidateSha: "a".repeat(40), repository: "example/tkt" }, publisher: "local", api: {} }), /sole publisher/);
});
test("standalone Cargo credential provider refuses changed seals and never caches read credentials", () => {
  const policy = { name: "tkt", version: "0.5.0", sha256: "a".repeat(64), index: "sparse+https://index.crates.io/" };
  const request = { v: 1, kind: "get", operation: "read", registry: { "index-url": policy.index }, args: [] };
  assert.deepEqual(credentialReply(request, policy, "fixture-token").Ok, { kind: "get", token: "fixture-token", cache: "never", operation_independent: false });
  assert.equal(credentialReply({ ...request, operation: "publish", name: "tkt", vers: "0.5.0", cksum: "b".repeat(64) }, policy, "fixture-token").Err.kind, "other");
});
test("standalone prepared Cargo policy accepts this manifest and rejects token overrides", () => {
  const manifest = parseToml(readFileSync(join(repo, "Cargo.toml"), "utf8"));
  const bytes = Buffer.from("policy fixture only");
  const options = { repo, candidateSha: "a".repeat(40), crate: { name: "tkt", version: manifest.package.version, bytes, sha256: sha(bytes) }, cargo: join(tools, "fixture-cargo"), rustc: join(tools, "fixture-rustc"), getToken: () => { throw Error("no credential during policy checks"); } };
  assert.doesNotThrow(() => validateCargoUploadManifest(manifest, cargoUploadPolicy(options)));
  assert.throws(() => cargoUploadPolicy({ ...options, token: "override" }), /overrides are forbidden/);
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
test("standalone Node, bash and mise preparation inspect without output or version changes", (t) => {
  const temp = mkdtempSync(join(tmpdir(), "tkt release entry ")), candidate = join(temp, "source with spaces"), output = join(temp, "output with spaces");
  const git = (...args) => execFileSync("git", ["-C", candidate, ...args], { encoding: "utf8", stdio: "pipe" }).trim();
  try {
    mkdirSync(candidate); git("init", "-b", "main"); git("config", "user.name", "Fixture"); git("config", "user.email", "fixture@example.invalid"); git("config", "core.autocrlf", "false");
    for (const file of ["Cargo.toml", "Cargo.lock", ".claude-plugin/plugin.json", ".codex-plugin/plugin.json", "plugin.json"]) { mkdirSync(dirname(join(candidate, file)), { recursive: true }); cpSync(join(repo, file), join(candidate, file)); }
    writeFileSync(join(candidate, "CHANGELOG.md"), "# Changelog\n\n## [Unreleased]\n\n### Added\n- Fixture change.\n");
    git("add", "."); git("-c", `core.hooksPath=${join(temp, "no hooks")}`, "commit", "-m", "Fixture");
    const before = git("rev-parse", "HEAD"), version = "999.0.0", args = ["prepare", "--repo", candidate, "--version", version, "--out", output];
    const cases = [[process.execPath, [join(tools, "release.mjs"), ...args]], ["bash", [join(tools, "release.sh"), ...args]]];
    // mise is an optional entry point: skip (with a message) when it is not on PATH,
    // matching the repo's convention for absent native tools.
    if (spawnSync("mise", ["--version"], { encoding: "utf8" }).status === 0) cases.push(["mise", ["run", "release", "--", ...args]]);
    else t.diagnostic("mise not on PATH; skipping the mise entry-point case");
    for (const [binary, invocation] of cases) {
      const result = spawnSync(binary, invocation, { cwd: repo, encoding: "utf8", env: { ...process.env, MISE_TRUSTED_CONFIG_PATHS: repo, MISE_STATE_DIR: join(temp, "mise-state"), MISE_CACHE_DIR: join(temp, "mise-cache") } });
      assert.equal(result.status, 0, `${binary}: ${result.stdout}${result.stderr}`); const report = JSON.parse(result.stdout); assert.equal(report.mode, "dry-run"); assert.equal(report.publishable, false); assert.equal(report.sourceSha, before); assert.equal(existsSync(output), false); assert.equal(git("rev-parse", "HEAD"), before); assert.equal(git("status", "--porcelain"), ""); assert.equal(git("tag", "--list"), "");
    }
  } finally { removeTree(temp); }
});

test("Cargo package excludes tracked scratch while retaining source and license", () => {
  const temp = mkdtempSync(join(tmpdir(), "tkt package exclusions "));
  const git = (...args) => execFileSync("git", ["-C", temp, ...args], { encoding: "utf8", stdio: "pipe" });
  const pkg = parseToml(readFileSync(join(repo, "Cargo.toml"), "utf8")).package;
  const policy = Object.fromEntries(["include", "exclude"].filter((key) => Object.hasOwn(pkg, key)).map((key) => [key, pkg[key]]));
  const oldExcludes = [".scratch/", ".tickets/", ".memory/", ".codex/", ".github/", "tests/"];
  try {
    for (const dir of ["src", ".scratch", "tools", ".claude-plugin", "skills/demo"]) mkdirSync(join(temp, dir), { recursive: true });
    writeFileSync(join(temp, "src/main.rs"), "fn main() {}\n");
    writeFileSync(join(temp, "LICENSE"), "Fixture license\n");
    for (const file of ["README.md", "CHANGELOG.md", "build.rs"]) writeFileSync(join(temp, file), "Fixture\n");
    for (const file of ["tools/maintainer.mjs", ".claude-plugin/plugin.json", "skills/demo/SKILL.md", "package.json", "tools/README.md", "tools/LICENSE", "tools/build.rs"]) writeFileSync(join(temp, file), "Maintainer fixture\n");
    writeFileSync(join(temp, ".scratch/HANDOFF.md"), "Internal planning fixture\n");
    writeFileSync(join(temp, "Cargo.toml"), '[package]\nname="package-exclusions-fixture"\nversion="0.0.0"\nedition="2021"\nlicense="MIT"\n');
    git("init"); git("add", ".");
    git("-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "-c", `core.hooksPath=${join(temp, "no-hooks")}`, "commit", "-m", "Tracked scratch fixture");
    const list = (selection) => {
      writeFileSync(join(temp, "Cargo.toml"), `[package]\nname="package-exclusions-fixture"\nversion="0.0.0"\nedition="2021"\nlicense="MIT"\n${Object.entries(selection).map(([key, value]) => `${key}=${JSON.stringify(value)}`).join("\n")}\n`);
      return execFileSync("cargo", ["package", "--list", "--offline", "--allow-dirty"], { cwd: temp, encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] }).trim().split(/\r?\n/).map((file) => file.replaceAll("\\", "/"));
    };
    assert.ok(list({ exclude: oldExcludes.filter((entry) => entry !== ".scratch/") }).includes(".scratch/HANDOFF.md"), "seed reproduces old archive leak");
    assert.ok(list({ exclude: oldExcludes }).includes("tools/maintainer.mjs"), "previous exclusions leaked maintainer files");
    const actual = list(policy);
    assert.ok(!actual.some((entry) => entry.startsWith(".scratch/")), "actual Cargo policy excludes tracked scratch");
    for (const file of ["src/main.rs", "LICENSE", "README.md", "CHANGELOG.md", "build.rs", "Cargo.toml", "Cargo.lock"]) assert.ok(actual.includes(file), `required ${file}`);
    for (const file of ["tools/maintainer.mjs", ".claude-plugin/plugin.json", "skills/demo/SKILL.md", "package.json", "tools/README.md", "tools/LICENSE", "tools/build.rs"]) assert.ok(!actual.includes(file), `unrelated ${file}`);
    assert.equal(readFileSync(join(temp, ".scratch/HANDOFF.md"), "utf8"), "Internal planning fixture\n");
  } finally { removeTree(temp); }
});

test("distributed skills do not require the external handoff command", () => {
  const skills = join(repo, "skills");
  for (const entry of readdirSync(skills, { withFileTypes: true })) {
    if (!entry.isDirectory()) continue;
    const file = join(skills, entry.name, "SKILL.md");
    assert.doesNotMatch(readFileSync(file, "utf8"), /`\/handoff`/, `${entry.name} must support a tkt-only adopter`);
  }
});

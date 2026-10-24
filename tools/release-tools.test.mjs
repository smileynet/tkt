import { test } from "node:test";
import { parse as parseToml } from "smol-toml";
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
  assert.equal(Object.keys(inventory.files).length, 17);
  assert.ok(inventory.files["tools/release-publish.mjs"]);
  assert.ok(inventory.files["tools/release-ci-artifacts.mjs"]);
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
      return execFileSync("cargo", ["package", "--list", "--offline", "--allow-dirty"], { cwd: temp, encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] }).trim().split(/\r?\n/);
    };
    assert.ok(list({ exclude: oldExcludes.filter((entry) => entry !== ".scratch/") }).includes(".scratch/HANDOFF.md"), "seed reproduces old archive leak");
    assert.ok(list({ exclude: oldExcludes }).includes("tools/maintainer.mjs"), "previous exclusions leaked maintainer files");
    const actual = list(policy);
    assert.ok(!actual.some((entry) => entry.startsWith(".scratch/")), "actual Cargo policy excludes tracked scratch");
    for (const file of ["src/main.rs", "LICENSE", "README.md", "CHANGELOG.md", "build.rs", "Cargo.toml", "Cargo.lock"]) assert.ok(actual.includes(file), `required ${file}`);
    for (const file of ["tools/maintainer.mjs", ".claude-plugin/plugin.json", "skills/demo/SKILL.md", "package.json", "tools/README.md", "tools/LICENSE", "tools/build.rs"]) assert.ok(!actual.includes(file), `unrelated ${file}`);
    assert.equal(readFileSync(join(temp, ".scratch/HANDOFF.md"), "utf8"), "Internal planning fixture\n");
  } finally { rmSync(temp, { recursive: true, force: true }); }
});

test("built identity belongs to packaged source or its own checkout across rebuilds", () => {
  const temp = mkdtempSync(join(tmpdir(), "tkt build identity ")), parent = join(temp, "parent"), source = join(parent, "source");
  const git = (dir, ...args) => execFileSync("git", ["-C", dir, ...args], { encoding: "utf8", stdio: "pipe" }).trim();
  const init = (dir) => { git(dir, "init", "-b", "main"); git(dir, "config", "user.name", "Fixture"); git(dir, "config", "user.email", "fixture@example.invalid"); git(dir, "config", "core.hooksPath", join(temp, "no-hooks")); };
  const commit = (dir, message) => { git(dir, "add", "."); git(dir, "commit", "--allow-empty", "-m", message); return git(dir, "rev-parse", "HEAD").slice(0, 7); };
  const makeSource = (dir) => {
    mkdirSync(join(dir, "src"), { recursive: true });
    cpSync(join(repo, "build.rs"), join(dir, "build.rs"));
    writeFileSync(join(dir, "Cargo.toml"), '[package]\nname="identity-fixture"\nversion="0.0.0"\nedition="2021"\n[build-dependencies]\nserde_json="1"\n');
    writeFileSync(join(dir, "src/main.rs"), 'fn main() { print!("{}", env!("TKT_GIT_HASH")); }\n');
    writeFileSync(join(dir, ".gitignore"), "target/\n");
  };
  const build = (dir, extra = {}) => {
    const env = Object.fromEntries(Object.entries(process.env).filter(([key]) => !key.startsWith("GIT_")));
    const result = spawnSync("cargo", ["run", "--quiet", "--offline", "--manifest-path", join(dir, "Cargo.toml"), "--target-dir", join(temp, "target")], { cwd: dir, encoding: "utf8", env: { ...env, ...extra } });
    assert.equal(result.status, 0, result.stderr); return result.stdout;
  };
  try {
    mkdirSync(parent); init(parent); writeFileSync(join(parent, "foreign"), "foreign"); const foreign = commit(parent, "Foreign ancestor");
    makeSource(source);
    const metadata = join(source, ".cargo_vcs_info.json"), packaged = "a".repeat(40);
    writeFileSync(metadata, JSON.stringify({ git: { sha1: packaged, dirty: false }, path_in_vcs: "" }));
    assert.equal(build(source), packaged.slice(0, 7), "packaged metadata wins over ancestor");
    writeFileSync(metadata, JSON.stringify({ git: { sha1: packaged, dirty: true } }));
    assert.equal(build(source), "aaaaaaa-dirty", "dirty metadata must remain visible");
    for (const invalid of ['{', '{}', JSON.stringify({ git: { sha1: "abc" } }), JSON.stringify({ git: { sha1: packaged, dirty: "false" } })]) {
      writeFileSync(metadata, invalid); assert.equal(build(source), "", "malformed metadata never falls through");
    }
    rmSync(metadata); assert.equal(build(source), "", "unknown source cannot borrow ancestor");
    init(source); const own = commit(source, "Own source");
    assert.notEqual(own, foreign); assert.equal(build(source, { GIT_DIR: join(parent, ".git"), GIT_WORK_TREE: parent, GIT_CONFIG_COUNT: "1", GIT_CONFIG_KEY_0: "core.worktree", GIT_CONFIG_VALUE_0: parent }), own, "ambient overrides cannot redirect identity");
    const next = commit(source, "Next source"); assert.equal(build(source), next, "same target rebuild sees branch movement");
    git(source, "pack-refs", "--all", "--prune"); assert.equal(build(source), next);
    git(source, "update-ref", "refs/heads/main", git(source, "rev-parse", "HEAD~1")); assert.equal(build(source), own, "packed to loose ref change rebuilds");
    const worktree = join(temp, "worktree"); git(source, "worktree", "add", "-b", "linked", worktree);
    assert.equal(build(worktree), own, "linked worktree uses its own Git paths");
    const linked = commit(worktree, "Linked commit"); assert.equal(build(worktree), linked, "linked branch change rebuilds");
    git(worktree, "checkout", "--detach", git(source, "rev-parse", "main")); assert.equal(build(worktree), own, "detached switch rebuilds");
  } finally { rmSync(temp, { recursive: true, force: true }); }
});

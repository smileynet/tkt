import {test} from "node:test";
import assert from "node:assert/strict";
import {cpSync, mkdirSync, mkdtempSync, rmSync, writeFileSync} from "node:fs";
import {execFileSync, spawnSync} from "node:child_process";
import {tmpdir} from "node:os";
import {join} from "node:path";
import {fileURLToPath} from "node:url";
import { removeTree } from "./test-fixtures.mjs";

const repo = fileURLToPath(new URL("../", import.meta.url));
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
    cpSync(join(repo, "Cargo.lock"), join(dir, "Cargo.lock"));
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
  } finally { removeTree(temp); }
});

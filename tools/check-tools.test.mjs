import { test } from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { changedPaths, executionPlan, selectChecks } from "./check-tools.mjs";
import { parse } from "smol-toml";

test("source selection keeps unrelated work cheap and selects native regressions conservatively", () => {
  assert.deepEqual(selectChecks(["tools/deploy.mjs", "src/core/tickets.rs"]), { identity: false, generation: false });
  assert.deepEqual(selectChecks(["build.rs"]), { identity: true, generation: false });
  assert.deepEqual(selectChecks(["src/cli.rs"]), { identity: true, generation: false });
  assert.deepEqual(selectChecks(["dist-workspace.toml"]), { identity: false, generation: true });
  for (const paths of [null, ["package-lock.json"], ["Cargo.lock"]]) {
    assert.deepEqual(selectChecks(paths), { identity: true, generation: true });
  }
  assert.deepEqual(selectChecks([], true), { identity: true, generation: true });
});

test("execution plans keep gate edits cheap and retain native source and release coverage", () => {
  const cheap = executionPlan(["tools/check-tools.mjs", "tools/check-tools.test.mjs", "mise.toml", "tools/deploy.mjs"]);
  assert.deepEqual(cheap.selected, { identity: false, generation: false });
  assert.deepEqual(cheap.files, ["tools/release-tools.test.mjs", "tools/release-ci.test.mjs", "tools/check-binary-fresh.test.mjs", "tools/check-tools.test.mjs", "tools/kimi-plugin.test.mjs"]);
  assert.ok(executionPlan(["build.rs"]).files.includes("tools/build-identity.test.mjs"));
  assert.ok(!executionPlan(["build.rs"]).files.includes("tools/release-generation.test.mjs"));
  assert.ok(executionPlan(["tools/generate-release-workflow.mjs"]).files.includes("tools/release-generation.test.mjs"));
  assert.deepEqual(executionPlan([], true).selected, { identity: true, generation: true });
});

test("task wiring forces native regressions for releases and preserves the Rust gate", () => {
  const { tasks } = parse(readFileSync(new URL("../mise.toml", import.meta.url), "utf8"));
  assert.match(tasks.check.run, /^node tools\/check-tools\.mjs\s*&&/);
  assert.match(tasks["check:release"].run, /^node tools\/check-tools\.mjs --release\s*&&/);
  for (const name of ["check", "check:release"]) {
    assert.match(tasks[name].run, /cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test\s*$/);
  }
});

test("explicit base includes intermediate commits, dirty edits, deletions and untracked inputs", t => {
  const repo = mkdtempSync(join(tmpdir(), "tool-scope-"));
  t.after(() => rmSync(repo, { recursive: true, force: true }));
  const git = (...args) => execFileSync("git", ["-C", repo, ...args], { encoding: "utf8", env: { ...process.env, GIT_CONFIG_NOSYSTEM: "1" } }).trim();
  git("init", "-b", "main");
  git("config", "user.name", "Fixture"); git("config", "user.email", "fixture@example.invalid");
  git("config", "commit.gpgsign", "false"); git("config", "core.hooksPath", join(repo, "no-hooks"));
  writeFileSync(join(repo, "build.rs"), "before"); git("add", "."); git("commit", "-m", "base");
  const base = git("rev-parse", "HEAD");
  assert.equal(changedPaths(repo), null);
  writeFileSync(join(repo, "build.rs"), "after"); git("add", "."); git("commit", "-m", "change");
  mkdirSync(join(repo, "tools")); writeFileSync(join(repo, "tools/unrelated.mjs"), "fixture");
  assert.deepEqual(changedPaths(repo, base).sort(), ["build.rs", "tools/unrelated.mjs"]);
  const foreign = join(repo, "foreign");
  git("clone", "--shared", repo, foreign);
  writeFileSync(join(repo, ".gitignore"), "foreign/\n");
  writeFileSync(join(repo, "build.rs"), "dirty identity input");
  const contexts = [
    { GIT_DIR: join(foreign, ".git"), GIT_WORK_TREE: foreign, GIT_INDEX_FILE: join(foreign, ".git/index") },
    { GIT_CONFIG_COUNT: "1", GIT_CONFIG_KEY_0: "core.worktree", GIT_CONFIG_VALUE_0: foreign },
  ];
  for (const context of contexts) {
    const saved = Object.fromEntries(Object.keys(context).map(key => [key, process.env[key]]));
    Object.assign(process.env, context);
    try {
      assert.ok(changedPaths(repo, "HEAD").includes("build.rs"), "foreign clean state must not hide a dirty identity input");
    } finally {
      for (const [key, value] of Object.entries(saved)) {
        if (value === undefined) delete process.env[key]; else process.env[key] = value;
      }
    }
  }
  writeFileSync(join(repo, "build.rs"), "after");
  git("rm", "build.rs");
  assert.ok(changedPaths(repo, base).includes("build.rs"));
  assert.throws(() => changedPaths(repo, "missing-base"));
});

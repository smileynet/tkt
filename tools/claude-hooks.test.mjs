import { test } from "node:test";
import assert from "node:assert/strict";
import { chmodSync, cpSync, mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { basename, join } from "node:path";
import { tmpdir } from "node:os";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const handler = fileURLToPath(new URL("../.claude-plugin/hooks/session-start.mjs", import.meta.url));
const body = `
const fs = require('node:fs');
fs.appendFileSync(process.env.FIXTURE_CALLS, JSON.stringify({args:process.argv.slice(1),cwd:process.cwd()})+'\\n');
if (process.env.FIXTURE_FAIL && process.argv.includes('query')) process.exit(1);
console.log(process.argv.includes('query') ? 'WIP fixture' : 'Frontier fixture');
process.exit(0);
`;
// Node resolves its first argument as a script path; a native tkt executable receives a subcommand.
const nativePreload = (executable) => `if (require('node:path').basename(process.execPath).toLowerCase() === ${JSON.stringify(executable.toLowerCase())}) { process.argv[1] = require('node:path').basename(process.argv[1]); ${body} }`;
function fixture(fn) {
  const dir = mkdtempSync(join(tmpdir(), "tkt hook's fixture-"));
  const project = join(dir, "project with spaces"), bin = join(dir, "bin"), calls = join(dir, "calls");
  mkdirSync(join(project, ".tickets"), { recursive: true });
  mkdirSync(bin);
  const env = { ...process.env, CLAUDE_PROJECT_DIR: project, FIXTURE_CALLS: calls, PATH: bin, DO_NOT_TRACK: "1" };
  if (process.platform === "win32") {
    cpSync(process.execPath, join(bin, "tkt.exe"));
    const preload = join(dir, "preload.cjs");
    writeFileSync(preload, nativePreload("tkt.exe"));
    env.NODE_OPTIONS = `--require ${JSON.stringify(preload)}`;
  } else {
    writeFileSync(join(bin, "tkt"), `#!${process.execPath}\n${body.replace("process.argv.slice(1)", "process.argv.slice(2)")}\n`);
    chmodSync(join(bin, "tkt"), 0o755);
  }
  const run = (overrides = {}, input = { hook_event_name: "SessionStart", source: "startup" }) => spawnSync(process.execPath, [handler], { cwd: dir, env: { ...env, ...overrides }, encoding: "utf8", input: JSON.stringify(input) });
  try { fn({ project, dir, calls, run }); }
  finally { rmSync(dir, { recursive: true, force: true }); }
}

test("SessionStart gathers frontier and WIP from the launch project before emitting context", () => fixture(({ project, calls, run }) => {
  const result = run();
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /tkt frontier\nFrontier fixture/);
  assert.match(result.stdout, /tkt in-progress tickets\nWIP fixture/);
  const observed = readFileSync(calls, "utf8").trim().split("\n").map(JSON.parse);
  assert.deepEqual(observed.map((call) => call.args), [["ready", "--json"], ["query", "--status", "in_progress"]]);
  assert.ok(observed.every((call) => call.cwd === realpathSync(project)));
}));

test("missing project tickets or tkt produces no context", () => fixture(({ project, dir, run }) => {
  for (const env of [{ PATH: "" }, { CLAUDE_PROJECT_DIR: dir }]) {
    const result = run(env);
    assert.equal(result.status, 0, result.stderr);
    assert.equal(result.stdout, "");
  }
  rmSync(join(project, ".tickets"), { recursive: true });
  assert.equal(run().stdout, "");
}));

test("failed second query never leaks partial frontier context", () => fixture(({ run }) => {
  const result = run({ FIXTURE_FAIL: "1" });
  assert.equal(result.status, 1);
  assert.equal(result.stdout, "");
  assert.match(result.stderr, /query --status in_progress failed/);
}));

test("native executable fixture preserves the first tkt argument", () => fixture(({ dir, calls }) => {
  const preload = join(dir, "argument-control.cjs");
  writeFileSync(preload, nativePreload(basename(process.execPath)));
  const result = spawnSync(process.execPath, ["--require", preload, "query", "--status", "in_progress"], { cwd: dir, encoding: "utf8", env: { ...process.env, FIXTURE_CALLS: calls } });
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /WIP fixture/);
  const observed = readFileSync(calls, "utf8").trim().split("\n").map(JSON.parse);
  assert.deepEqual(observed.map((call) => call.args), [["query", "--status", "in_progress"]]);
}));

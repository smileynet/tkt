import { test } from "node:test";
import assert from "node:assert/strict";
import { chmodSync, cpSync, mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { basename, join } from "node:path";
import { tmpdir } from "node:os";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { removeTree } from "./test-fixtures.mjs";


const handler = fileURLToPath(new URL("../.claude-plugin/hooks/session-start.mjs", import.meta.url));
const manifest = fileURLToPath(new URL("../.claude-plugin/claude-hooks.json", import.meta.url));
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
  // The host dialect is chosen by ZCODE_* markers; strip them so a ZCode-hosted
  // test runner cannot flip the plain-text (Claude Code) expectations.
  delete env.ZCODE_SESSION_ID;
  delete env.ZCODE_PLUGIN_ROOT;
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
  try { fn({ project, dir, bin, calls, run }); }
  finally { removeTree(dir); }
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
  // A tkt-free HOME keeps this deterministic now that the hook appends
  // ~/.cargo/bin and ~/.local/bin fallbacks to a stripped PATH.
  const bareHome = join(dir, "bare-home");
  mkdirSync(bareHome);
  for (const env of [{ PATH: "", HOME: bareHome, USERPROFILE: bareHome }, { CLAUDE_PROJECT_DIR: dir }]) {
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

test("ZCode-hosted runs emit the strict hookEventName JSON envelope", () => fixture(({ run }) => {
  for (const marker of [{ ZCODE_SESSION_ID: "sess-fixture" }, { ZCODE_PLUGIN_ROOT: "/plugins/tkt" }]) {
    const result = run(marker);
    assert.equal(result.status, 0, result.stderr);
    const envelope = JSON.parse(result.stdout);
    assert.equal(envelope.hookEventName, "SessionStart");
    assert.match(envelope.additionalContext, /tkt frontier\nFrontier fixture/);
    assert.match(envelope.additionalContext, /tkt in-progress tickets\nWIP fixture/);
    assert.deepEqual(Object.keys(envelope).sort(), ["additionalContext", "hookEventName"]);
  }
}));

test("GUI-minimal PATH still finds tkt via HOME install roots", () => fixture(({ run, dir }) => {
  // A GUI-launched host passes PATH=/usr/bin:/bin:… with no user install roots;
  // the hook must still discover ~/.cargo/bin/tkt (live 2026-11-13: the fresh
  // ZCode session ran the hook, found no tkt, and silently injected nothing).
  const home = join(dir, "fixture-home");
  const cargoBin = join(home, ".cargo", "bin");
  mkdirSync(cargoBin, { recursive: true });
  if (process.platform === "win32") {
    // fixture() already installs the NODE_OPTIONS preload for basename tkt.exe;
    // a node copy under that name picks the fixture body up the same way.
    cpSync(process.execPath, join(cargoBin, "tkt.exe"));
  } else {
    writeFileSync(join(cargoBin, "tkt"), `#!${process.execPath}\n${body.replace("process.argv.slice(1)", "process.argv.slice(2)")}\n`);
    chmodSync(join(cargoBin, "tkt"), 0o755);
  }
  const result = run({ PATH: "/usr/bin:/bin", HOME: home, USERPROFILE: home });
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /tkt frontier\nFrontier fixture/, "tkt must resolve from $HOME/.cargo/bin when PATH lacks it");
}));

test("inherited PATH keeps precedence over appended install roots", () => fixture(({ run, dir }) => {
  const home = join(dir, "fixture-home");
  const cargoBin = join(home, ".cargo", "bin");
  mkdirSync(cargoBin, { recursive: true });
  const homeBody = body.replace("'Frontier fixture'", "'HOME-root fixture'");
  writeFileSync(join(cargoBin, "tkt"), `#!${process.execPath}\n${homeBody.replace("process.argv.slice(1)", "process.argv.slice(2)")}\n`);
  chmodSync(join(cargoBin, "tkt"), 0o755);
  const result = run({ HOME: home, USERPROFILE: home });
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /Frontier fixture/);
  assert.doesNotMatch(result.stdout, /HOME-root fixture/, "an explicit PATH tkt must win over the appended roots");
}));

test("manifest keeps a single shell-string command with no args field", () => {
  const entry = JSON.parse(readFileSync(manifest, "utf8")).hooks.SessionStart[0].hooks[0];
  assert.equal(entry.type, "command");
  assert.equal(entry.args, undefined, "args is a Claude Code direct-spawn field that ZCode's command type ignores, spawning bare node (ticket 286)");
  assert.match(entry.command, /^node ".+\.claude-plugin\/hooks\/session-start\.mjs"$/);
  assert.equal(entry.timeout, 15);
});

test("native executable fixture preserves the first tkt argument", () => fixture(({ dir, calls }) => {
  const preload = join(dir, "argument-control.cjs");
  writeFileSync(preload, nativePreload(basename(process.execPath)));
  const result = spawnSync(process.execPath, ["--require", preload, "query", "--status", "in_progress"], { cwd: dir, encoding: "utf8", env: { ...process.env, FIXTURE_CALLS: calls } });
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /WIP fixture/);
  const observed = readFileSync(calls, "utf8").trim().split("\n").map(JSON.parse);
  assert.deepEqual(observed.map((call) => call.args), [["query", "--status", "in_progress"]]);
}));

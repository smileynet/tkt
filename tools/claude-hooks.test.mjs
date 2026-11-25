import { test } from "node:test";
import assert from "node:assert/strict";
import { chmodSync, cpSync, mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { basename, join } from "node:path";
import { tmpdir } from "node:os";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { removeTree } from "./test-fixtures.mjs";


const handler = fileURLToPath(new URL("../.claude-plugin/hooks/session-start.mjs", import.meta.url));
const preToolUseHandler = fileURLToPath(new URL("../.claude-plugin/hooks/pre-tool-use.mjs", import.meta.url));
const manifest = fileURLToPath(new URL("../.claude-plugin/claude-hooks.json", import.meta.url));
import { apply as applyDshHook, guardTicketStatusDone } from "../.claude-plugin/hooks/dsh-session-start.mjs";
const body = `
const fs = require('node:fs');
fs.appendFileSync(process.env.FIXTURE_CALLS, JSON.stringify({args:process.argv.slice(1),cwd:process.cwd()})+'\\n');
if (process.env.FIXTURE_FAIL && process.argv.includes('query')) process.exit(1);
console.log(process.argv.includes('query') ? 'WIP fixture' : 'Frontier fixture');
process.exit(0);
`;
// Node resolves its first argument as a script path; a native tkt executable receives a subcommand.
const nativePreload = (executable) => `if (require('node:path').basename(process.execPath).toLowerCase() === ${JSON.stringify(executable.toLowerCase())}) { process.argv[1] = require('node:path').basename(process.argv[1]); ${body} }`;
async function fixture(fn) {
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
  try { return await fn({ project, dir, bin, calls, env, run }); }
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
  const parsed = JSON.parse(readFileSync(manifest, "utf8")).hooks;
  const entry = parsed.SessionStart[0].hooks[0];
  assert.equal(entry.type, "command");
  assert.equal(entry.args, undefined, "args is a Claude Code direct-spawn field that ZCode's command type ignores, spawning bare node (ticket 286)");
  assert.match(entry.command, /^node ".+\.claude-plugin\/hooks\/session-start\.mjs"$/);
  assert.equal(entry.timeout, 15);

  const pre = parsed.PreToolUse[0];
  assert.equal(pre.matcher, "^Edit$|^Write$|^edit$|^write_file$");
  assert.equal(pre.hooks[0].type, "command");
  assert.equal(pre.hooks[0].args, undefined);
  assert.match(pre.hooks[0].command, /^node ".+\.claude-plugin\/hooks\/pre-tool-use\.mjs"$/);
  assert.equal(pre.hooks[0].timeout, 10);
});

test("PreToolUse blocks direct status: done edits in .tickets/*.md across Claude, Qwen, and ZCode", () => {
  const runPre = (payload, extraEnv = {}) => {
    const env = { ...process.env, DO_NOT_TRACK: "1", ...extraEnv };
    if (!("ZCODE_SESSION_ID" in extraEnv)) delete env.ZCODE_SESSION_ID;
    if (!("ZCODE_PLUGIN_ROOT" in extraEnv)) delete env.ZCODE_PLUGIN_ROOT;
    return spawnSync(process.execPath, [preToolUseHandler], { encoding: "utf8", env, input: JSON.stringify(payload) });
  };

  for (const payload of [
    { tool_name: "Edit", tool_input: { file_path: "/repo/.tickets/01-task.md", old_string: "status: in_progress", new_string: "status: done" } },
    { tool_name: "write_file", tool_input: { path: ".tickets\\02-win.md", content: "---\nid: \"02\"\nstatus: \"done\"\n---\n" } },
    { tool_name: "edit", tool_input: { filePath: "project/.tickets/03-qwen.md", edits: [{ newText: "status: 'done'" }] } },
  ]) {
    const res = runPre(payload);
    assert.equal(res.status, 0, res.stderr);
    const body = JSON.parse(res.stdout);
    assert.equal(body.hookSpecificOutput.hookEventName, "PreToolUse");
    assert.equal(body.hookSpecificOutput.permissionDecision, "deny");
    assert.match(body.hookSpecificOutput.permissionDecisionReason, /tkt close/);
  }

  // ZCode uses top-level envelope without hookSpecificOutput.
  const zcode = runPre({ tool_name: "Edit", tool_input: { file_path: ".tickets/04-z.md", new_string: "status: done" } }, { ZCODE_SESSION_ID: "z-1" });
  assert.equal(zcode.status, 0, zcode.stderr);
  const zBody = JSON.parse(zcode.stdout);
  assert.equal(zBody.hookEventName, "PreToolUse");
  assert.equal(zBody.permissionDecision, "deny");
  assert.equal(zBody.hookSpecificOutput, undefined);

  // Allowed cases: non-done status, non-ticket file, and inline prose mentioning status: done.
  for (const allowed of [
    { tool_name: "Edit", tool_input: { file_path: ".tickets/01-task.md", new_string: "status: in_progress" } },
    { tool_name: "Write", tool_input: { file_path: "docs/notes.md", content: "status: done\n" } },
    { tool_name: "Edit", tool_input: { file_path: ".tickets/01-task.md", new_string: "- Never hand-edit `status: done` directly." } },
  ]) {
    const res = runPre(allowed);
    assert.equal(res.status, 0, res.stderr);
    assert.equal(res.stdout, "");
  }
});

test("DSH hook skips resumed sessions, guards status: done edits, and registers read-only tkt tools", async () => {
  await fixture(async ({ project, bin, calls, env }) => {
    const handlers = new Map();
    const guards = [];
    const registered = [];
    const injected = [];
    const prevPath = process.env.PATH;
    const prevCalls = process.env.FIXTURE_CALLS;
    const prevNodeOptions = process.env.NODE_OPTIONS;
    const prevDoNotTrack = process.env.DO_NOT_TRACK;
    process.env.PATH = bin;
    process.env.FIXTURE_CALLS = calls;
    process.env.DO_NOT_TRACK = "1";
    if (env.NODE_OPTIONS !== undefined) process.env.NODE_OPTIONS = env.NODE_OPTIONS;
    try {
      const toolCtx = {
        workingDirectory: { ensure: async () => project },
        tools: {
          guard: (fn) => guards.push(fn),
          register: (def, opts) => registered.push({ def, opts }),
        },
      };
      const ctx = {
        workingDirectory: toolCtx.workingDirectory,
        on: (event, fn) => handlers.set(event, fn),
        inject: (_deps, fn) => fn(toolCtx),
      };
      applyDshHook(ctx);
      const onCreated = handlers.get("agent/created");
      assert.equal(typeof onCreated, "function");

      // Resume must skip injection to preserve V4.1 KV-cache prefix.
      await onCreated({ agent: { inject: (m) => injected.push(m) }, source: "resume" });
      assert.equal(injected.length, 0);

      // Fresh session injects frontier + WIP.
      await onCreated({ agent: { inject: (m) => injected.push(m) }, source: "new" });
      assert.equal(injected.length, 1);
      assert.match(injected[0].content[0].text, /Frontier fixture/);

      // Monotonic guard blocks .tickets/*.md status: done edits and allows others.
      assert.equal(guards.length, 1);
      assert.match(guardTicketStatusDone({ name: "edit", arguments: { file_path: ".tickets/10-a.md", new_string: "status: done" } }), /tkt close/);
      assert.equal(guardTicketStatusDone({ name: "edit", arguments: { file_path: ".tickets/10-a.md", new_string: "status: in_progress" } }), undefined);
      assert.equal(guardTicketStatusDone({ name: "write", arguments: { path: "README.md", content: "status: done\n" } }), undefined);

      // Read-only parallel-safe tools registered and executable.
      assert.deepEqual(registered.map((r) => r.def.name).sort(), ["tkt_query", "tkt_ready"]);
      for (const { def, opts } of registered) {
        assert.equal(def.isConcurrencySafe(), true);
        assert.equal(await opts.when({ agent: {} }), true);
      }
      const readyTool = registered.find((r) => r.def.name === "tkt_ready").def;
      const queryTool = registered.find((r) => r.def.name === "tkt_query").def;
      const readyOut = await readyTool.execute({}, { agent: {} });
      assert.equal(readyOut.stdout, "Frontier fixture");
      assert.deepEqual(readyTool.output.render({}, readyOut), [{ type: "text", text: "Frontier fixture" }]);
      const queryOut = await queryTool.execute({ status: "in_progress", allContext: true }, { agent: {} });
      assert.equal(queryOut.stdout, "WIP fixture");
    } finally {
      process.env.PATH = prevPath;
      if (prevCalls === undefined) delete process.env.FIXTURE_CALLS;
      else process.env.FIXTURE_CALLS = prevCalls;
      if (prevNodeOptions === undefined) delete process.env.NODE_OPTIONS;
      else process.env.NODE_OPTIONS = prevNodeOptions;
      if (prevDoNotTrack === undefined) delete process.env.DO_NOT_TRACK;
      else process.env.DO_NOT_TRACK = prevDoNotTrack;
    }
  });
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

test("Antigravity SessionStart hook reads workspacePaths and emits injectSteps", () => fixture(({ project, bin, dir }) => {
  const agyHandler = fileURLToPath(new URL("../.claude-plugin/hooks/agy-session-start.mjs", import.meta.url));
  const env = { ...process.env, PATH: bin, DO_NOT_TRACK: "1", FIXTURE_CALLS: join(dir, "agy-calls") };
  if (process.platform === "win32") env.NODE_OPTIONS = `--require ${JSON.stringify(join(dir, "preload.cjs"))}`;
  const ok = spawnSync(process.execPath, [agyHandler], { cwd: dir, env, encoding: "utf8", input: JSON.stringify({ workspacePaths: [dir, project] }) });
  assert.equal(ok.status, 0, ok.stderr);
  const payload = JSON.parse(ok.stdout);
  assert.match(payload.injectSteps[0].systemMessage.systemMessage, /tkt frontier\nFrontier fixture/);
  assert.match(payload.injectSteps[0].systemMessage.systemMessage, /tkt in-progress tickets\nWIP fixture/);
  const empty = spawnSync(process.execPath, [agyHandler], { cwd: dir, env, encoding: "utf8", input: JSON.stringify({ workspacePaths: [dir] }) });
  assert.equal(empty.status, 0, empty.stderr);
  assert.equal(empty.stdout, "");
}));

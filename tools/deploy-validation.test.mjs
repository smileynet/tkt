import { test } from "node:test";
import assert from "node:assert/strict";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const installer = fileURLToPath(new URL("./deploy.mjs", import.meta.url));

test("static manifest checks preserve bytes and reject invalid input without native tools", (t) => {
  const root = mkdtempSync(join(tmpdir(), "tkt-static-validation-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const pack = join(root, "pack"), manifest = join(pack, ".claude-plugin/plugin.json");
  mkdirSync(join(pack, ".claude-plugin"), { recursive: true });
  mkdirSync(join(pack, "skills/example"), { recursive: true });
  writeFileSync(manifest, JSON.stringify({ name: "example", version: "1.0.0", skills: "./skills/" }));
  const skill = join(pack, "skills/example/SKILL.md");
  writeFileSync(skill, "---\nname: example\ndescription: Example fixture\n---\nExample.\n");
  const before = readFileSync(manifest), skillBefore = readFileSync(skill);
  const marker = join(root, "native-called"), preload = join(root, "native.mjs");
  // Intercept by executable name so the fixture also covers Windows native shims.
  writeFileSync(preload, `import cp from "node:child_process";
import {syncBuiltinESMExports} from "node:module";
import {writeFileSync} from "node:fs";
const original = cp.spawnSync;
cp.spawnSync = (command, ...args) => {
  if (command !== "claude") return original(command, ...args);
  writeFileSync(${JSON.stringify(marker)}, "called");
  return {status: 1, stdout: "", stderr: "fixture native rejection"};
};
syncBuiltinESMExports();\n`);
  const run = (...args) => spawnSync(process.execPath, ["--import", preload, installer, "check", "--pack", pack, ...args], {
    encoding: "utf8", env: { ...process.env, DO_NOT_TRACK: "1" },
  });
  const clean = run("--static");
  assert.equal(clean.status, 0, clean.stdout + clean.stderr);
  assert.equal(existsSync(marker), false);
  assert.deepEqual(readFileSync(manifest), before);
  assert.deepEqual(readFileSync(skill), skillBefore);

  const native = run();
  assert.equal(native.status, 1);
  assert.match(native.stderr, /fixture native rejection/);
  assert.equal(existsSync(marker), true);
  rmSync(marker);
  writeFileSync(manifest, JSON.stringify({ name: "example", version: "invalid", skills: "./skills/" }));
  const invalid = run("--static");
  assert.equal(invalid.status, 1);
  assert.match(invalid.stderr, /version/);
  assert.equal(existsSync(marker), false);
});

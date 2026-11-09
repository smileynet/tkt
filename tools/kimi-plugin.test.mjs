import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { removeTree } from "./test-fixtures.mjs";


const root = dirname(dirname(fileURLToPath(import.meta.url)));
const builder = join(root, "tools", "build-kimi-plugin.mjs");
const skillNames = ["tkt-core", "tkt-orient", "tkt-reconcile", "tkt-work"];

function fixture(t) {
  const dir = mkdtempSync(join(tmpdir(), "tkt-kimi-plugin-"));
  t.after(() => removeTree(dir));
  return join(dir, "plugin");
}

function build(out) {
  const result = spawnSync(process.execPath, [builder, "--pack", root, "--out", out], {
    encoding: "utf8",
    env: { ...process.env, DO_NOT_TRACK: "1" },
  });
  assert.equal(result.status, 0, result.stdout + result.stderr);
  return result;
}

test("kimi: build writes the four self-contained skills and catalog metadata", (t) => {
  const out = fixture(t);
  build(out);

  const skillsRoot = join(out, "skills");
  const discovered = readdirSync(skillsRoot).filter((name) => existsSync(join(skillsRoot, name, "SKILL.md")));
  assert.deepEqual(discovered.sort(), skillNames);
  const manifest = JSON.parse(readFileSync(join(out, "kimi.plugin.json"), "utf8"));
  const source = JSON.parse(readFileSync(join(root, ".claude-plugin", "plugin.json"), "utf8"));
  assert.equal(manifest.name, source.name);
  assert.equal(manifest.version, source.version);
  assert.equal(manifest.license, source.license);
  assert.equal(manifest.skills, "./skills/");
  assert.deepEqual(Object.keys(manifest.interface.skillOverrides).sort(), skillNames);
  assert.equal(manifest.hooks, undefined);
  assert.equal(manifest.mcp, undefined);
  assert.equal(manifest.commands, undefined);

  const locale = JSON.parse(readFileSync(join(out, "locales", "zh-CN.json"), "utf8"));
  assert.deepEqual(Object.keys(locale.skillOverrides).sort(), skillNames);
  assert.notEqual(locale.longDescription, manifest.interface.longDescription);
  assert(readFileSync(join(out, "icon.png")).equals(readFileSync(join(root, ".kimi-plugin", "icon.png"))));

  for (const name of skillNames) {
    assert(existsSync(join(out, "skills", name, "SKILL.md")), name);
    assert(existsSync(join(out, "skills", name, "LICENSE")), `${name} license`);
  }
  for (const resource of ["commands.md", "capabilities.md", "native-hooks.md", "roles.md", "ticket-standards.md"]) {
    assert(existsSync(join(out, "skills", "tkt-core", "references", resource)), resource);
  }
  assert.equal(existsSync(join(out, "setup.sh")), false);
  assert.equal(existsSync(join(out, "commands")), false);
  assert.equal(existsSync(join(out, "hooks")), false);
});

test("kimi: unchanged rebuild preserves output and later builds repair drift", (t) => {
  const out = fixture(t);
  build(out);
  const manifest = join(out, "kimi.plugin.json");
  const before = statSync(manifest, { bigint: true }).mtimeNs;
  const second = build(out);
  assert.match(second.stdout, /up to date/);
  assert.equal(statSync(manifest, { bigint: true }).mtimeNs, before);

  const skill = join(out, "skills", "tkt-core", "SKILL.md");
  const resource = join(out, "skills", "tkt-core", "references", "commands.md");
  const expected = readFileSync(join(root, "skills", "tkt-core", "SKILL.md"));
  writeFileSync(skill, "damaged\n");
  rmSync(resource);
  build(out);
  assert(readFileSync(skill).equals(expected));
  assert(existsSync(resource));
});

import { test } from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, writeFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { execFileSync, spawnSync } from "node:child_process";

const INSTALLER = join(dirname(fileURLToPath(import.meta.url)), "deploy.mjs");
function runInstaller(args) {
  const result = spawnSync(process.execPath, [INSTALLER, ...args], { encoding: "utf8", env: { ...process.env, DO_NOT_TRACK: "1" } });
  return { status: result.status, stdout: result.stdout + result.stderr };
}

// Exercise native warning semantics rather than duplicating Claude's validator.
test("deployment check rejects root context and opt-in catalog warnings", (t) => {
  try { execFileSync("claude", ["--version"], { stdio: "pipe" }); }
  catch { t.skip("Claude CLI unavailable"); return; }
  const pack = mkdtempSync(join(tmpdir(), "deploy-validation-"));
  t.after(() => rmSync(pack, { recursive: true, force: true }));
  mkdirSync(join(pack, ".claude-plugin"));
  mkdirSync(join(pack, "skills", "example"), { recursive: true });
  writeFileSync(join(pack, ".claude-plugin", "plugin.json"), JSON.stringify({
    name: "example", version: "1.0.0", description: "Example plugin for deployment validation.",
    author: { name: "Example" }, license: "MIT", skills: "./skills/",
  }));
  writeFileSync(join(pack, ".claude-plugin", "marketplace.json"), JSON.stringify({ name: "example", metadata: { description: "Example marketplace for deployment validation." }, owner: { name: "Example" }, plugins: [{ name: "example", source: "./" }] }));
  writeFileSync(join(pack, "README.md"), "# Example\n" + "This plugin provides example workflows for local development and testing. ".repeat(6));
  writeFileSync(join(pack, "skills", "example", "SKILL.md"), "---\nname: example\ndescription: An example skill for validation.\n---\nDo example work.\n");
  const check = (...args) => runInstaller(["check", "--pack", pack, ...args]);
  const clean = check();
  assert.equal(clean.status, 0, clean.stdout);
  writeFileSync(join(pack, "CLAUDE.md"), "Project context\n");
  const context = check();
  assert.notEqual(context.status, 0, context.stdout);
  assert.match(context.stdout, /CLAUDE.md at the plugin root/);
  rmSync(join(pack, "CLAUDE.md"));
  const catalog = check("--profile", "directory");
  assert.notEqual(catalog.status, 0, catalog.stdout);
  assert.match(catalog.stdout, /No icon|ICON_MISSING/);
  // A valid 512px PNG removes the only catalog warning.
  writeFileSync(join(pack, ".claude-plugin", "icon.png"), Buffer.from("iVBORw0KGgoAAAANSUhEUgAAAgAAAAIACAIAAAB7GkOtAAAHIElEQVR4nO3VMQ0AMAzAsEIZtEEdtMHoEUsGkC9z7gMgaNYLAFhhAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABBlAABRBgAQZQAAUQYAEGUAAFEGABD1AUEfSwGgnVe6AAAAAElFTkSuQmCC", "base64"));
  const catalogClean = check("--profile", "directory");
  assert.equal(catalogClean.status, 0, catalogClean.stdout);
});

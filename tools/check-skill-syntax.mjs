#!/usr/bin/env node
// Parse skill frontmatter and Codex sidecars independently of host scanner behavior.
// Usage: npm ci; node scripts/check-skill-syntax.mjs [--pack DIR]
import { readFileSync, readdirSync, existsSync, realpathSync } from "node:fs";
import { dirname, join, resolve, relative, basename } from "node:path";
import { fileURLToPath } from "node:url";
import { parseDocument, isMap } from "yaml";

export function checkPack(pack) {
  const errors = [];
  const metrics = { skills: 0, sidecars: 0 };
  const readYaml = (file, text) => {
    const doc = parseDocument(text, { strict: true, uniqueKeys: true });
    const diagnostics = [...doc.errors, ...doc.warnings];
    for (const error of diagnostics) errors.push(`${relative(pack, file)}: ${error.message}`);
    if (diagnostics.length) return null;
    if (!isMap(doc.contents)) { errors.push(`${relative(pack, file)}: expected a YAML mapping`); return null; }
    return doc.toJS();
  };
  const string = (data, key, file, required = false, max = Infinity) => {
    if (!required && !(key in data)) return;
    if (typeof data[key] !== "string" || !data[key].trim() || data[key].length > max) errors.push(`${relative(pack, file)}: ${key} must be a nonempty string${max === Infinity ? "" : ` of at most ${max} characters`}`);
  };
  const boolean = (data, key, file) => {
    if (key in data && typeof data[key] !== "boolean") errors.push(`${relative(pack, file)}: ${key} must be boolean`);
  };
  const mapping = (value) => value !== null && typeof value === "object" && !Array.isArray(value);
  const walk = (dir) => {
    const entries = readdirSync(dir, { withFileTypes: true });
    const file = join(dir, "SKILL.md");
    if (!entries.some((e) => e.isFile() && e.name === "SKILL.md")) {
      for (const e of entries.filter((e) => e.isDirectory() && !e.name.startsWith("."))) walk(join(dir, e.name));
      return;
    }
    metrics.skills++;
    const match = readFileSync(file, "utf8").match(/^---\r?\n([\s\S]*?)\r?\n---(?:\r?\n|$)/);
    if (!match) { errors.push(`${relative(pack, file)}: missing YAML frontmatter delimiters`); return; }
    const data = readYaml(file, match[1]);
    if (data) {
      string(data, "name", file, true, 64);
      string(data, "description", file, true, 1024);
      if (typeof data.name === "string" && (!/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(data.name) || data.name !== basename(dir))) errors.push(`${relative(pack, file)}: name must match its lowercase hyphenated directory`);
      string(data, "license", file);
      string(data, "compatibility", file, false, 500);
      for (const key of ["disable-model-invocation", "user-invocable", "disable-slash-command"]) boolean(data, key, file);
      if ("metadata" in data && (!mapping(data.metadata) || Object.values(data.metadata).some((v) => typeof v !== "string"))) errors.push(`${relative(pack, file)}: metadata must map strings to strings`);
    }
    const sidecar = join(dir, "agents", "openai.yaml");
    if (!existsSync(sidecar)) return;
    metrics.sidecars++;
    const yaml = readYaml(sidecar, readFileSync(sidecar, "utf8"));
    if (!yaml) return;
    if ("policy" in yaml) {
      if (!mapping(yaml.policy)) errors.push(`${relative(pack, sidecar)}: policy must be a mapping`);
      else boolean(yaml.policy, "allow_implicit_invocation", sidecar);
    }
    if ("interface" in yaml) {
      if (!mapping(yaml.interface)) errors.push(`${relative(pack, sidecar)}: interface must be a mapping`);
      else for (const key of ["display_name", "short_description", "default_prompt", "icon_small", "icon_large", "brand_color"]) string(yaml.interface, key, sidecar);
    }
  };
  walk(join(pack, "skills"));
  if (!metrics.skills) errors.push("pack contains no skills");
  return { status: errors.length ? "fail" : "pass", metrics, errors };
}

if (process.argv[1] && existsSync(resolve(process.argv[1])) && realpathSync(fileURLToPath(import.meta.url)) === realpathSync(resolve(process.argv[1]))) {
  try {
    const args = process.argv.slice(2);
    if (args.length && (args.length !== 2 || args[0] !== "--pack")) throw new Error("usage: check-skill-syntax.mjs [--pack DIR]");
    const result = checkPack(args.length ? resolve(args[1]) : resolve(dirname(fileURLToPath(import.meta.url)), ".."));
    console.log(JSON.stringify(result, null, 2));
    if (result.status === "fail") process.exitCode = 1;
  } catch (error) {
    console.error(JSON.stringify({ status: "error", metrics: {}, errors: [error.message] }));
    process.exitCode = 2;
  }
}

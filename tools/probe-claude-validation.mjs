#!/usr/bin/env node
// Validate manifests and exact skill components, including a seeded scanner-coverage check.
// Usage: node scripts/probe-claude-validation.mjs [--pack DIR] [--binary CLAUDE_BINARY] [--snapshot]
// Requires Claude >=2.1.289. No model, install, authentication or network request is needed.
import { mkdtempSync, mkdirSync, readdirSync, readFileSync, writeFileSync, cpSync, rmSync, lstatSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve, basename } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";

let pack = resolve(dirname(fileURLToPath(import.meta.url)), "..");
let binary = "claude", archived = false;
const home = mkdtempSync(join(tmpdir(), "claude-validation-"));
try {
  const args = process.argv.slice(2);
  for (let i = 0; i < args.length; i++) {
    if (args[i] === "--pack" && args[i + 1]) pack = resolve(args[++i]);
    else if (args[i] === "--binary" && args[i + 1]) binary = args[++i];
    else if (args[i] === "--snapshot") archived = true;
    else throw new Error(`unknown or incomplete argument: ${args[i]}`);
  }
  const env = { PATH: process.env.PATH, HOME: home, USERPROFILE: home, CLAUDE_CONFIG_DIR: join(home, "config"), DISABLE_UPDATES: "1", CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC: "1" };
  const run = (args) => {
    const r = spawnSync(binary, args, { cwd: home, env, encoding: "utf8", timeout: 30000 });
    if (r.error) throw r.error;
    return { args, exit: r.status, stdout: r.stdout, stderr: r.stderr };
  };
  const versionRun = run(["--version"]);
  const version = versionRun.stdout.trim();
  const numbers = version.match(/^(\d+)\.(\d+)\.(\d+)/)?.slice(1).map(Number);
  if (versionRun.exit !== 0 || !numbers || numbers[0] < 2 || (numbers[0] === 2 && numbers[1] === 1 && numbers[2] < 289) || (numbers[0] === 2 && numbers[1] < 1)) throw new Error(`Claude >=2.1.289 required; found ${version || versionRun.stderr}`);
  // Full checkouts can exceed the native directory scan cap because of target/ or .git.
  const tracked = [];
  if (archived) {
    const inventory = (relative = "") => {
      for (const name of readdirSync(join(pack, relative))) {
        const file = join(relative, name), stat = lstatSync(join(pack, file));
        if (stat.isDirectory()) inventory(file);
        else if (stat.isFile()) tracked.push(file);
        else throw new Error(`snapshot contains a non-regular path: ${file}`);
      }
    };
    inventory();
  } else {
    const listing = spawnSync("git", ["ls-files", "-z"], { cwd: pack, encoding: "utf8" });
    if (listing.error || listing.status !== 0) throw new Error(`cannot enumerate tracked plugin inputs: ${listing.error?.message ?? listing.stderr}`);
    tracked.push(...listing.stdout.split("\0").filter(Boolean));
  }
  const snapshot = join(home, "source");
  for (const file of tracked) {
    const dest = join(snapshot, file);
    mkdirSync(dirname(dest), { recursive: true });
    cpSync(join(pack, file), dest, { recursive: true });
  }
  const skills = [];
  const walk = (dir) => {
    const entries = readdirSync(dir, { withFileTypes: true });
    if (entries.some((e) => e.name === "SKILL.md" && e.isFile())) { skills.push(dir); return; }
    for (const e of entries.filter((e) => e.isDirectory() && !e.name.startsWith("."))) walk(join(dir, e.name));
  };
  walk(join(pack, "skills"));
  if (!skills.length) throw new Error("pack contains no skill components");
  const flat = join(home, "components", "skills");
  mkdirSync(flat, { recursive: true });
  const hashes = [];
  for (const dir of skills.sort()) {
    const dest = join(flat, basename(dir));
    cpSync(dir, dest, { recursive: true, errorOnExist: true, force: false });
    const source = readFileSync(join(dir, "SKILL.md"));
    if (!source.equals(readFileSync(join(dest, "SKILL.md")))) throw new Error(`component copy differs: ${dir}`);
    hashes.push({ name: basename(dir), sha256: createHash("sha256").update(source).digest("hex") });
  }
  const runs = [];
  for (const target of [join(snapshot, ".claude-plugin", "plugin.json"), join(snapshot, ".claude-plugin", "marketplace.json")]) {
    const r = run(["plugin", "validate", target, "--json"]);
    runs.push({ ...r, result: JSON.parse(r.stdout), stdout: undefined });
  }
  const clean = run(["plugin", "validate", flat, "--strict", "--json"]);
  runs.push({ ...clean, result: JSON.parse(clean.stdout), stdout: undefined });
  const seeded = join(flat, basename(skills[0]), "SKILL.md");
  const original = readFileSync(seeded, "utf8");
  if (!/^description:.*$/m.test(original)) throw new Error("seed control requires a description field");
  writeFileSync(seeded, original.replace(/^description:.*$/m, "description: [seeded-invalid-array]"));
  const defect = run(["plugin", "validate", flat, "--strict", "--json"]);
  writeFileSync(seeded, original);
  const seedDetected = defect.exit !== 0 && /description must be a string/.test(defect.stdout);
  let readmeControl = { detected: false, reason: "source README.md missing" };
  const readme = join(snapshot, "README.md");
  if (tracked.includes("README.md")) {
    const bytes = readFileSync(readme);
    rmSync(readme);
    const result = run(["plugin", "validate", join(snapshot, ".claude-plugin/plugin.json"), "--json"]);
    writeFileSync(readme, bytes);
    const parsed = JSON.parse(result.stdout);
    readmeControl = { exit: result.exit, detected: parsed.directory?.findings?.some((finding) => finding.code === "README_MISSING" && finding.blocking) ?? false, result: parsed };
  }
  const pass = runs.every((r) => r.exit === 0 && r.result.success && !r.result.directory?.incomplete && !r.result.directory?.failure && !(r.result.directory?.findings ?? []).some((finding) => finding.blocking)) && seedDetected && readmeControl.detected;
  console.log(JSON.stringify({ status: pass ? "pass" : "fail", metrics: { skills: skills.length, nativeChecks: runs.length, copiedSkillsMatched: hashes.length, trackedSnapshotFiles: tracked.length }, version, pack, sourceSnapshot: archived ? "extracted snapshot bytes" : "tracked working-tree bytes, excluding untracked build output", runs, seed: { exit: defect.exit, detected: seedDetected, result: JSON.parse(defect.stdout) }, readmeControl, hashes, errors: pass ? [] : ["native validation, directory blockers/completeness or seeded controls failed"] }, null, 2));
  if (!pass) process.exitCode = 1;
} catch (error) {
  console.error(JSON.stringify({ status: "error", metrics: {}, errors: [error.message] }));
  process.exitCode = 2;
} finally {
  rmSync(home, { recursive: true, force: true });
}

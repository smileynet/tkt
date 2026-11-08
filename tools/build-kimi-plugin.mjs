#!/usr/bin/env node
// Build a self-contained Kimi skill-only plugin without changing the shared deployer.
// The deployer's explicit-root copy mode owns skill flattening and resource links;
// this driver adds Kimi metadata and atomically replaces the generated package.

import { execFileSync } from "node:child_process";
import {
  cpSync, existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync,
  renameSync, rmSync, writeFileSync,
} from "node:fs";
import { basename, dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");

function usage() {
  console.log("Usage: node tools/build-kimi-plugin.mjs [--pack DIR] [--out DIR]");
}

function parseArgs(argv) {
  const args = { pack: ROOT, out: null };
  for (let i = 0; i < argv.length; i++) {
    if (argv[i] === "--pack") args.pack = resolve(argv[++i]);
    else if (argv[i] === "--out") args.out = resolve(argv[++i]);
    else if (argv[i] === "--help") { usage(); process.exit(0); }
    else throw new Error(`unknown argument: ${argv[i]}`);
  }
  args.out ??= join(args.pack, "dist", "kimi-plugin");
  if (args.out === args.pack) throw new Error("--out must not be the pack root");
  return args;
}

function filesUnder(root) {
  if (!existsSync(root)) return null;
  return readdirSync(root, { recursive: true, withFileTypes: true })
    .filter((entry) => entry.isFile())
    .map((entry) => join(entry.parentPath ?? entry.path, entry.name))
    .sort();
}

function sameTree(left, right) {
  const a = filesUnder(left), b = filesUnder(right);
  if (!a || !b || a.length !== b.length) return false;
  return a.every((file, index) => {
    const other = b[index];
    return relative(left, file) === relative(right, other) && readFileSync(file).equals(readFileSync(other));
  });
}

function replacePackage(stage, out) {
  const parent = dirname(out);
  const backup = join(parent, `.${basename(out)}-previous-${process.pid}`);
  let retained = false;
  try {
    if (existsSync(out)) {
      renameSync(out, backup);
      retained = true;
    }
    renameSync(stage, out);
    if (retained) rmSync(backup, { recursive: true, force: true });
  } catch (error) {
    if (!existsSync(out) && retained && existsSync(backup)) renameSync(backup, out);
    throw error;
  } finally {
    rmSync(stage, { recursive: true, force: true });
    if (existsSync(backup)) rmSync(backup, { recursive: true, force: true });
  }
}

function build(pack, out) {
  const source = join(pack, ".kimi-plugin");
  const manifestPath = join(source, "kimi.plugin.json");
  const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
  if (manifest.skills !== "./skills/") throw new Error("Kimi manifest skills must be ./skills/");
  if (!existsSync(join(source, "locales"))) throw new Error("Kimi localization directory is missing");
  if (!existsSync(join(source, "icon.png"))) throw new Error("Kimi icon is missing");

  const parent = dirname(out);
  mkdirSync(parent, { recursive: true });
  const stage = mkdtempSync(join(parent, ".kimi-stage-"));
  try {
    const skills = join(stage, "skills");
    const deployer = join(pack, "tools", "deploy.mjs");
    const output = execFileSync(process.execPath, [deployer, "--pack", pack, "--root", skills], { encoding: "utf8" });
    process.stdout.write(output);
    rmSync(join(skills, `.installed-by-${manifest.name}.json`), { force: true });
    cpSync(join(source, "locales"), join(stage, "locales"), { recursive: true });
    cpSync(join(source, "icon.png"), join(stage, "icon.png"));
    writeFileSync(join(stage, "kimi.plugin.json"), JSON.stringify(manifest, null, 2) + "\n");

    if (sameTree(stage, out)) {
      console.log(`Kimi plugin ${manifest.name} is up to date: ${out}`);
      return;
    }
    replacePackage(stage, out);
    console.log(`Kimi plugin built: ${out}`);
  } finally {
    rmSync(stage, { recursive: true, force: true });
  }
}

try {
  const args = parseArgs(process.argv.slice(2));
  build(args.pack, args.out);
} catch (error) {
  console.error(`Kimi plugin build failed: ${error.message}`);
  process.exit(1);
}

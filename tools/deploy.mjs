#!/usr/bin/env node
// deploy.mjs 1.1.0 (grey-matter) — deploy a skill pack to every agent harness on this machine.
//
// One deployer for every project that ships skills (proposal 08). The project declares WHAT
// it ships in .claude-plugin/plugin.json (name, version, `skills` dirs; deploy-only settings
// under `metadata.deploy`). This file owns WHERE and HOW: the harness table below, copy or
// link into skill roots, an ownership manifest per root, plugin packaging, and the rule that
// no harness sees a skill twice. Other projects vendor this file at tools/deploy.mjs; it is
// dependency-free and finds its pack as the parent of its own directory (or --pack DIR).
//
// Usage:
//   node deploy.mjs [install] [--dry-run] [--force] [--link] [--allow-dual] [--no-agy]
//                   [--project] [--root DIR]... [--pack DIR]
//   node deploy.mjs list | uninstall [--dry-run] [--root DIR]...
//   node deploy.mjs check            # manifests consistent with disk and tags (exit 1 on drift)
//   node deploy.mjs audit [--strict] # unowned dirs, shared names, skills a harness sees twice
//   node deploy.mjs agy [build|install] [--out DIR]
//
// Install with no --root picks roots from the harness table: of the harnesses present here
// (their home dir exists), each one sees each skill exactly once, counting a plugin of this
// pack as its copy, and roots a harness is only *claimed* to read never count as its copy.
// Copies this pack left in roots it no longer needs are removed (manifest-tracked only). It
// also packages the agy plugin when ~/.gemini exists. With --root, it installs exactly
// there, except a root read by a harness that has this pack's plugin (--allow-dual overrides).
//
// Safety: only manifest-tracked dirs are ever removed or overwritten; a foreign dir with the
// same name is refused unless --force. Codex refresh requires its CLI and a local marketplace.

import {
  cpSync, existsSync, lstatSync, mkdirSync, mkdtempSync, renameSync, unlinkSync, readdirSync, readFileSync, realpathSync, rmSync, statSync,
  symlinkSync, writeFileSync,
} from "node:fs";
import { execFileSync, spawnSync } from "node:child_process";
import { homedir, platform } from "node:os";
import { basename, dirname, join, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

const DEPLOY_VERSION = "1.1.0";
const HOME = homedir();
const isDir = (p) => { try { return statSync(p).isDirectory(); } catch { return false; } };
const isFile = (p) => { try { return statSync(p).isFile(); } catch { return false; } };
const exists = (p) => { try { lstatSync(p); return true; } catch { return false; } };
const readJson = (p) => { try { return JSON.parse(readFileSync(p, "utf8")); } catch { return null; } };
// Roots are compared by where they really are: one harness's root can be a symlink to
// another's (~/.codex/skills -> ~/.kiro/skills), and then both harnesses share every copy.
const real = (p) => { try { return realpathSync(p); } catch { return resolve(p); } };
const uniqReal = (roots) => roots.filter((r, i) => roots.findIndex((q) => real(q) === real(r)) === i);
const tilde = (p) => (p.startsWith(HOME + sep) ? "~" + p.slice(HOME.length) : p);

// --- The pack ----------------------------------------------------------------------------
function loadPack(dir) {
  const manifestPath = join(dir, ".claude-plugin", "plugin.json");
  const manifest = readJson(manifestPath);
  if (!manifest?.name) { console.error(`no pack manifest (name) at ${manifestPath}`); process.exit(1); }
  const listed = [].concat(manifest.skills ?? []);
  const dirs = [...new Set(["./skills", ...listed].map((p) => resolve(dir, p)))];
  const skills = [];
  for (const d of dirs) {
    if (isFile(join(d, "SKILL.md"))) { skills.push({ name: basename(d), src: d }); continue; }
    if (!isDir(d)) continue;
    for (const n of readdirSync(d).sort()) {
      if (isFile(join(d, n, "SKILL.md"))) skills.push({ name: n, src: join(d, n) });
    }
  }
  const deploy = manifest.metadata?.deploy ?? {};
  return { dir, manifest, manifestPath, name: manifest.name, version: manifest.version, listed, dirs, skills, deploy };
}

function frontmatterName(skillMd) {
  const m = readFileSync(skillMd, "utf8").match(/^---\s*[\r\n]([\s\S]*?)[\r\n]---/);
  const line = m?.[1].split(/\r?\n/).find((l) => /^name\s*:/.test(l));
  return line ? line.replace(/^name\s*:\s*/, "").trim().replace(/^["']|["']$/g, "") : null;
}

// name == dir, names unique once flattened, and no reference to a skill outside the pack
// (metadata.deploy.forbidRefs: a pack that must work on its own lists its neighbours here).
function lint(pack) {
  const errors = [];
  const seen = new Map();
  const forbid = (pack.deploy.forbidRefs ?? []).map((r) => new RegExp(`\\b${r.replace(/[-/\\^$*+?.()|[\]{}]/g, "\\$&")}\\b`));
  for (const s of pack.skills) {
    const rel = relative(pack.dir, s.src);
    const fm = frontmatterName(join(s.src, "SKILL.md"));
    if (fm && fm !== s.name) errors.push(`name mismatch: ${rel}/SKILL.md declares "${fm}" but its folder is "${s.name}"`);
    if (seen.has(s.name)) errors.push(`duplicate skill name "${s.name}": ${seen.get(s.name)} and ${rel} (names must be unique once flattened)`);
    else seen.set(s.name, rel);
    const text = readFileSync(join(s.src, "SKILL.md"), "utf8");
    const hits = forbid.filter((re) => re.test(text)).map((re) => re.source.replace(/\\b|\\/g, ""));
    if (hits.length) errors.push(`${rel}/SKILL.md references skills outside the pack: ${hits.join(", ")}`);
  }
  return errors;
}

// --- Harnesses ---------------------------------------------------------------------------
// `own`: the root the deployer copies into for this harness. `reads`: every user-level root
// it scans; `claimed: true` marks one we have not seen it load (it counts against duplicates,
// never as the harness's copy). `project`: project-level dir names it scans (<proj>/<d>/skills).
// `plugin`: how to tell whether this pack is installed as the harness's plugin.
const CODEX_HOME = () => process.env.CODEX_HOME || join(HOME, ".codex");
const SHARED = () => join(HOME, ".agents", "skills");
const HARNESSES = () => [
  { id: "kiro", home: join(HOME, ".kiro"), own: join(HOME, ".kiro", "skills"),
    reads: [{ root: join(HOME, ".kiro", "skills") }], project: [".kiro"] },
  { id: "codex", home: CODEX_HOME(), own: join(CODEX_HOME(), "skills"),
    reads: [{ root: join(CODEX_HOME(), "skills") }, { root: SHARED(), claimed: true }],
    project: [".codex", ".agents"], plugin: codexPlugin },
  { id: "claude", home: join(HOME, ".claude"), own: join(HOME, ".claude", "skills"),
    reads: [{ root: join(HOME, ".claude", "skills") }], project: [".claude", ".agents"], plugin: claudePlugin },
  { id: "opencode", home: join(HOME, ".config", "opencode"), own: join(HOME, ".config", "opencode", "skills"),
    reads: [{ root: join(HOME, ".config", "opencode", "skills") }, { root: join(HOME, ".claude", "skills") }, { root: SHARED() }],
    project: [".opencode", ".claude", ".agents"] },
];

// Claude Code: installed_plugins.json lists `<name>@<marketplace>` ids with their install path.
function claudePlugin(pack) {
  const dir = process.env.CLAUDE_CODE_PLUGIN_CACHE_DIR || join(HOME, ".claude", "plugins");
  const data = readJson(join(dir, "installed_plugins.json"));
  const id = Object.keys(data?.plugins ?? {}).find((k) => k.startsWith(`${pack.name}@`));
  return id ? `Claude plugin ${id}` : null;
}

// Codex: enabled marketplace plugins are `[plugins."<name>@<marketplace>"] enabled = true`.
function codexPlugin(pack) {
  const config = join(CODEX_HOME(), "config.toml");
  if (!isFile(config)) return null;
  for (const section of readFileSync(config, "utf8").split(/(?=^\[)/m)) {
    const id = /^\[plugins\.["']([^"']+)["']\]/.exec(section)?.[1];
    if (id?.startsWith(`${pack.name}@`) && /^enabled\s*=\s*true\s*(?:#.*)?$/m.test(section)) return `Codex plugin ${id}`;
  }
  return null;
}

// Refresh before pruning copies: a failed plugin update must leave the old deployment intact.
function refreshCodex(pack, args) {
  const label = codexPlugin(pack);
  if (!label) return;
  const id = label.slice("Codex plugin ".length);
  console.log(`\n${args.dryRun ? "would refresh" : "refreshing"} Codex plugin ${id}`);
  if (args.dryRun) return;
  const listing = JSON.parse(execFileSync("codex", ["plugin", "list", "--json"], { encoding: "utf8" }));
  const installed = listing.installed?.find((p) => p.pluginId === id && p.enabled);
  if (installed?.source?.source !== "local" || real(installed.source.path) !== real(pack.dir)) {
    throw new Error(`${id} must use this checkout as its local marketplace before deploy: codex plugin marketplace add ${pack.dir}`);
  }
  execFileSync("codex", ["plugin", "add", id, "--json"], { stdio: "inherit" });
}

// A plugin harness must not inherit another harness's bare copies through a root symlink.
function separatePluginRoots(pack, args) {
  for (const h of HARNESSES()) {
    if (!h.plugin?.(pack) || !exists(h.own) || !lstatSync(h.own).isSymbolicLink()) continue;
    console.log(`\n${args.dryRun ? "would separate" : "separating"} ${h.id} skill root ${h.own} from ${real(h.own)}`);
    if (args.dryRun) continue;
    const target = real(h.own);
    const staged = mkdtempSync(join(dirname(h.own), ".skills-detached-"));
    try {
      cpSync(target, staged, { recursive: true });
      unlinkSync(h.own);
      try { renameSync(staged, h.own); }
      catch (error) {
        symlinkSync(target, h.own, platform() === "win32" ? "junction" : "dir");
        throw error;
      }
    } finally { rmSync(staged, { recursive: true, force: true }); }
  }
}

// Which harnesses read an arbitrary root: exact user roots, or <proj>/<dir>/skills patterns.
function readersOf(root, harnesses) {
  const r = resolve(root);
  return harnesses.filter((h) => h.reads.some((x) => real(x.root) === real(r)) ||
    (basename(r) === "skills" && h.project.includes(basename(dirname(r))) && resolve(dirname(dirname(r))) !== resolve(HOME)));
}

// Pick the roots: every present harness gets exactly one verified copy (none if this pack is
// its plugin) and never two of any kind. A missing skill costs more than a duplicate one, so
// when they conflict (two harnesses sharing a root, one with the plugin) the copy wins.
// Smallest cost, then fewest roots, wins. Roots are compared by real path.
function chooseRoots(present, candidates) {
  let best = null;
  for (let mask = 0; mask < 1 << candidates.length; mask++) {
    const chosen = candidates.filter((_, i) => mask & (1 << i)).map(real);
    let violations = 0;
    for (const h of present) {
      const reads = h.reads.filter((x, i) => h.reads.findIndex((y) => real(y.root) === real(x.root)) === i);
      const all = reads.filter((x) => chosen.includes(real(x.root))).length;
      const verified = reads.filter((x) => !x.claimed && chosen.includes(real(x.root))).length;
      violations += h.covered ? all : (verified < 1 ? 2 : 0) + Math.max(0, all - 1);
    }
    const score = [violations, chosen.length, mask];
    if (!best || score[0] < best.score[0] || (score[0] === best.score[0] && score[1] < best.score[1])) {
      best = { chosen: candidates.filter((_, i) => mask & (1 << i)), score };
    }
  }
  return best;
}

// --- Ownership manifest ------------------------------------------------------------------
const manifestName = (pack) => `.installed-by-${pack.name}.json`;
const readOwned = (pack, root) => readJson(join(root, manifestName(pack)))?.installed ?? [];
function writeOwned(pack, root, names, mode) {
  writeFileSync(join(root, manifestName(pack)), JSON.stringify({
    installer: `deploy.mjs ${DEPLOY_VERSION}`, pack: pack.name, repo: pack.dir, mode,
    installedAt: new Date().toISOString(), installed: names.slice().sort(),
  }, null, 2) + "\n");
}
function rmIfExists(p) { if (exists(p)) { rmSync(p, { recursive: true, force: true }); return ""; } return " (already gone)"; }

// Take back every copy this pack put in `root`, and its manifest. Never touches foreign dirs.
function release(pack, root, why, dryRun) {
  const names = readOwned(pack, root);
  if (!names.length) return;
  console.log(`\nroot: ${root}  (${why})`);
  for (const n of names) console.log(`  remove  ${n}${dryRun ? "" : rmIfExists(join(root, n))}`);
  if (!dryRun) rmSync(join(root, manifestName(pack)), { force: true });
}

function installRoot(pack, root, args) {
  if (!args.dryRun) mkdirSync(root, { recursive: true });
  const owned = new Set(readOwned(pack, root));
  const desired = new Set(pack.skills.map((s) => s.name));
  console.log(`\nroot: ${root}  (${args.dryRun ? "dry-run" : args.link ? "link" : "copy"})`);
  for (const n of owned) if (!desired.has(n)) console.log(`  prune   ${n}${args.dryRun ? "" : rmIfExists(join(root, n))}`);
  // Record ONLY what was installed: a refused foreign dir must never become "owned" (ticket 26).
  const installed = [];
  let refused = 0;
  for (const s of pack.skills) {
    const dest = join(root, s.name);
    if (exists(dest) && !owned.has(s.name) && !args.force) {
      console.log(`  refused ${s.name} (exists, not installed by ${pack.name}; --force to override)`);
      refused++;
      continue;
    }
    installed.push(s.name);
    if (args.dryRun) { console.log(`  ${exists(dest) ? "replace" : "create "} ${s.name}`); continue; }
    rmIfExists(dest);
    let action = "copied ";
    if (args.link) {
      try { symlinkSync(s.src, dest, platform() === "win32" ? "junction" : "dir"); action = "linked "; }
      catch { cpSync(s.src, dest, { recursive: true }); action = "copied  (link failed, fell back to copy)"; }
    } else cpSync(s.src, dest, { recursive: true });
    console.log(`  ${action} ${s.name}`);
  }
  if (!args.dryRun) writeOwned(pack, root, installed, args.link ? "link" : "copy");
  if (refused) console.log(`  ${refused} refused (foreign dirs); re-run with --force to overwrite.`);
}

// --- Commands ----------------------------------------------------------------------------
function doInstall(pack, args) {
  if (!pack.skills.length) { console.error(`no skills found for ${pack.name}`); process.exit(1); }
  const errors = lint(pack);
  if (errors.length) { console.error("install aborted, lint failed:"); for (const e of errors) console.error("  - " + e); process.exit(1); }
  if (!args.roots.length) {
    refreshCodex(pack, args);
    if (!args.allowDual) separatePluginRoots(pack, args);
  }
  const harnesses = HARNESSES().map((h) => ({ ...h, covered: !args.allowDual && h.plugin ? h.plugin(pack) : null }));

  if (args.roots.length) {
    for (const root of args.roots) {
      const via = readersOf(root, harnesses).find((h) => h.covered);
      if (!via) { installRoot(pack, root, args); continue; }
      const why = `skipped: ${via.covered} is installed; --allow-dual to copy anyway`;
      if (readOwned(pack, root).length) release(pack, root, why, args.dryRun);
      else console.log(`\nroot: ${root}  (${why})`);
    }
  } else {
    const present = harnesses.filter((h) => isDir(h.home));
    const candidates = uniqReal([...present.map((h) => h.own), SHARED()]);
    let chosen;
    if (!present.length) { chosen = [SHARED()]; console.log("no known harness found; using the shared ~/.agents/skills"); }
    else {
      const best = chooseRoots(present, candidates);
      chosen = best.chosen;
      for (const h of present) {
        const copies = chosen.filter((r) => h.reads.some((x) => real(x.root) === real(r)));
        const via = h.covered ?? (chosen.filter((r) => h.reads.some((x) => !x.claimed && real(x.root) === real(r))).map(tilde).join(", ") || "nothing");
        const dup = h.covered && copies.length ? `  (also sees the copy in ${copies.map(tilde).join(", ")}: twice)` : "";
        console.log(`  ${h.id.padEnd(8)} <- ${via}${dup}`);
      }
      if (best.score[0]) console.log("warning: no set of roots gives every harness exactly one copy here (a shared or symlinked root); see `deploy.mjs audit`");
    }
    for (const root of chosen) installRoot(pack, root, args);
    const known = [...new Set(HARNESSES().flatMap((h) => h.reads.map((x) => x.root)))];
    const chosenReal = chosen.map(real);
    for (const root of known) if (!chosenReal.includes(real(root))) release(pack, root, "not needed: each harness that reads it has its copy elsewhere", args.dryRun);
    if (!args.noAgy && isDir(join(HOME, ".gemini"))) agyDeploy(pack, { ...args, cmd: "install" });
  }
  console.log(`\n${args.dryRun ? "(dry-run) would install" : "installed"} ${pack.skills.length} skills.`);
}

function knownRoots(args) {
  return args.roots.length ? args.roots : [...new Set(HARNESSES().flatMap((h) => h.reads.map((x) => x.root)))];
}

function doList(pack, args) {
  for (const root of knownRoots(args)) {
    const names = readOwned(pack, root);
    if (!names.length && !args.roots.length) continue;
    console.log(`\nroot: ${root}`);
    if (!names.length) { console.log(`  (nothing installed by ${pack.name})`); continue; }
    const m = readJson(join(root, manifestName(pack)));
    console.log(`  installed ${names.length} (mode: ${m.mode}, at ${m.installedAt}):`);
    for (const n of names) console.log(`    ${n}`);
  }
}

function doUninstall(pack, args) {
  for (const root of knownRoots(args)) {
    if (!readOwned(pack, root).length) { if (args.roots.length) console.log(`\nroot: ${root}  (nothing to uninstall)`); continue; }
    release(pack, root, args.dryRun ? "dry-run" : "uninstall", args.dryRun);
  }
}

// agy discovers skills from plugins in ~/.gemini/config/plugins/<name>/ (plugin.json + skills/).
function agyDeploy(pack, args) {
  const out = args.out ?? join(pack.dir, "dist", "agy-plugin");
  const errors = lint(pack);
  if (errors.length) { console.error("agy build aborted, lint failed:"); for (const e of errors) console.error("  - " + e); process.exit(1); }
  const m = pack.manifest;
  const agyManifest = {
    name: m.name, displayName: m.displayName ?? m.name, version: m.version, description: m.description,
    author: m.author, keywords: m.keywords, repository: m.repository, license: m.license,
  };
  console.log(`\nagy plugin: ${tilde(out)}  (${pack.skills.length} skills, v${m.version})`);
  if (args.dryRun) return;
  rmSync(out, { recursive: true, force: true });
  mkdirSync(join(out, "skills"), { recursive: true });
  for (const s of pack.skills) cpSync(s.src, join(out, "skills", s.name), { recursive: true });
  writeFileSync(join(out, "plugin.json"), JSON.stringify(agyManifest, null, 2) + "\n");
  if (args.cmd !== "install") return;
  const hasAgy = spawnSync("agy", ["--version"], { stdio: "ignore" }).error?.code !== "ENOENT";
  if (hasAgy) {
    execFileSync("agy", ["plugin", "validate", out, "--strict"], { stdio: "inherit" });
    execFileSync("agy", ["plugin", "install", out], { stdio: "inherit" });
  } else {
    const dest = join(HOME, ".gemini", "config", "plugins", pack.name);
    rmSync(dest, { recursive: true, force: true });
    cpSync(out, dest, { recursive: true });
    console.log(`  copied to ${tilde(dest)} (agy not on PATH)`);
  }
}

// Manifests agree with disk, with each other, and with the latest release tag.
function doCheck(pack) {
  const problems = [];
  // Coverage: every SKILL.md under skills/ (up to two levels) is one the manifest loads.
  const loaded = new Set(pack.skills.map((s) => resolve(s.src)));
  const walk = (d, depth) => {
    if (!isDir(d) || depth > 2) return;
    for (const n of readdirSync(d)) {
      const p = join(d, n);
      if (isFile(join(p, "SKILL.md"))) {
        if (!loaded.has(resolve(p))) {
          const parent = "./" + relative(pack.dir, d).split(sep).join("/");
          const msg = `plugin.json skills is missing "${parent}" (its skills would not load)`;
          if (!problems.includes(msg)) problems.push(msg);
        }
      } else walk(p, depth + 1);
    }
  };
  walk(join(pack.dir, "skills"), 1);
  for (const l of pack.listed) {
    const d = resolve(pack.dir, l);
    if (!pack.skills.some((s) => s.src === d || dirname(s.src) === d)) problems.push(`plugin.json skills lists "${l}", which holds no skills`);
  }
  // Version: not behind the latest vX.Y.Z tag (plugins cache by version).
  const semver = (v) => (/^v?(\d+)\.(\d+)\.(\d+)$/.exec(v ?? "") || []).slice(1).map(Number);
  const behind = (a, b) => { for (let i = 0; i < 3; i++) if (a[i] !== b[i]) return a[i] < b[i]; return false; };
  const v = semver(pack.version);
  if (v.length !== 3) problems.push(`plugin.json version "${pack.version}" is not X.Y.Z`);
  else {
    const tags = (spawnSync("git", ["tag", "--list", "v*"], { cwd: pack.dir, encoding: "utf8" }).stdout || "").split(/\r?\n/);
    const latest = tags.map((t) => ({ t, v: semver(t) })).filter((x) => x.v.length === 3)
      .reduce((a, b) => (!a || behind(a.v, b.v) ? b : a), null);
    if (latest && behind(v, latest.v)) problems.push(`plugin.json version ${pack.version} is behind the latest tag ${latest.t} (installed users would not update)`);
  }
  // Claude marketplace: lists this pack at the repo root, version kept only in plugin.json.
  const claudeMarket = readJson(join(pack.dir, ".claude-plugin", "marketplace.json"));
  if (claudeMarket) {
    const e = claudeMarket.plugins?.find((p) => p.name === pack.name);
    if (!e) problems.push(`marketplace.json has no entry for plugin "${pack.name}"`);
    else {
      if (e.source !== "./") problems.push(`marketplace.json entry "${pack.name}" source is ${JSON.stringify(e.source)}, expected "./"`);
      if ("version" in e) problems.push(`marketplace.json entry "${pack.name}" sets version; keep it only in plugin.json`);
    }
  }
  // Codex: same identity and version; its marketplace points at the repo root.
  const codex = readJson(join(pack.dir, ".codex-plugin", "plugin.json"));
  if (codex) {
    if (codex.name !== pack.name || codex.version !== pack.version) problems.push("Codex and Claude plugin identities/versions differ");
    if (codex.skills !== "./skills/") problems.push("Codex skills must point at ./skills/");
    const e = readJson(join(pack.dir, ".agents", "plugins", "marketplace.json"))?.plugins?.find((p) => p.name === codex.name);
    if (e?.source?.path !== "./" || e?.source?.source !== "local") problems.push("Codex marketplace must point at the repository root");
    if (e?.policy?.installation !== "AVAILABLE" || e?.policy?.authentication !== "ON_INSTALL") problems.push("Codex marketplace installation/authentication policy missing");
  }
  // claude plugin validate, when claude is on PATH (not --strict: a repo-root CLAUDE.md warns).
  for (const t of [pack.manifestPath, claudeMarket && join(pack.dir, ".claude-plugin", "marketplace.json")].filter(Boolean)) {
    const r = spawnSync("claude", ["plugin", "validate", t], { cwd: pack.dir, encoding: "utf8" });
    if (r.error?.code === "ENOENT") { console.log("(claude not on PATH: skipped `claude plugin validate`)"); break; }
    if (r.status !== 0) problems.push(`claude plugin validate ${relative(pack.dir, t)} failed:\n${r.stdout}${r.stderr}`);
  }
  if (problems.length) {
    console.error(`plugin-manifest check FAILED (${problems.length}):`);
    for (const p of problems) console.error("  - " + p);
    process.exit(1);
  }
  console.log(`plugin-manifest check OK: ${pack.name} v${pack.version}, ${pack.skills.length} skills from ${pack.dirs.filter(isDir).length} dirs.`);
}

// What is in every skill root here, who owns it, and what each harness ends up seeing.
function doAudit(pack, args) {
  const harnesses = HARNESSES();
  const roots = uniqReal([...knownRoots({ roots: [] }), ...[".agents", ".claude", ".codex", ".opencode", ".kiro"].map((d) => resolve(d, "skills"))].filter(isDir));
  const findings = [];
  const contents = new Map(); // root -> Map(name -> [packs])
  for (const root of roots.filter(isDir)) {
    const owners = new Map();
    for (const f of readdirSync(root).filter((f) => /^\.installed-by-.+\.json$/.test(f))) {
      const p = f.slice(".installed-by-".length, -".json".length);
      for (const n of readJson(join(root, f))?.installed ?? []) owners.set(n, [...(owners.get(n) ?? []), p]);
    }
    const names = readdirSync(root).filter((n) => !n.startsWith("."));
    contents.set(root, new Map(names.map((n) => [n, owners.get(n) ?? []])));
    const unowned = names.filter((n) => !owners.has(n));
    if (unowned.length) findings.push(`${tilde(root)}: ${unowned.length} unowned: ${unowned.join(" ")}`);
    for (const [n, ps] of owners) if (ps.length > 1) findings.push(`${tilde(root)}: "${n}" claimed by ${ps.join(", ")}`);
  }
  // Claude plugins: read each installed plugin's skills from its install path.
  const cache = process.env.CLAUDE_CODE_PLUGIN_CACHE_DIR || join(HOME, ".claude", "plugins");
  const claudeSkills = new Map();
  for (const [id, installs] of Object.entries(readJson(join(cache, "installed_plugins.json"))?.plugins ?? {})) {
    const at = installs?.[0]?.installPath;
    const m = at && readJson(join(at, ".claude-plugin", "plugin.json"));
    if (!m) continue;
    const pk = loadPackQuiet(at, m);
    for (const s of pk) claudeSkills.set(s, [...(claudeSkills.get(s) ?? []), `plugin ${id}`]);
  }
  for (const h of harnesses.filter((h) => isDir(h.home))) {
    const seen = new Map(h.id === "claude" ? claudeSkills : []);
    const codex = h.id === "codex" && codexPlugin(pack);
    if (codex) for (const s of pack.skills) seen.set(s.name, [codex]);
    const rootsRead = uniqReal([...h.reads.map((x) => x.root), ...roots.filter((r) => readersOf(r, [h]).length)].filter(isDir));
    for (const r of rootsRead) for (const n of contents.get(r)?.keys() ?? []) seen.set(n, [...(seen.get(n) ?? []), tilde(r)]);
    const dupes = [...seen].filter(([, src]) => src.length > 1);
    if (dupes.length) findings.push(`${h.id} sees ${dupes.length} skill(s) twice: ${dupes.map(([n, s]) => `${n} (${s.join(" + ")})`).join("; ")}`);
  }
  if (!findings.length) console.log("audit: clean (every dir owned, no harness sees a skill twice)");
  else { console.log(`audit: ${findings.length} finding(s)`); for (const f of findings) console.log("  - " + f); }
  if (args.strict && findings.length) process.exit(1);
}

function loadPackQuiet(dir, manifest) {
  const dirs = [...new Set(["./skills", ...[].concat(manifest.skills ?? [])].map((p) => resolve(dir, p)))];
  const names = [];
  for (const d of dirs) {
    if (isFile(join(d, "SKILL.md"))) names.push(basename(d));
    else if (isDir(d)) for (const n of readdirSync(d)) if (isFile(join(d, n, "SKILL.md"))) names.push(n);
  }
  return names;
}

// --- Main --------------------------------------------------------------------------------
function parseArgs(argv) {
  const args = { cmd: "install", sub: null, dryRun: false, force: false, link: false, allowDual: false,
    noAgy: false, strict: false, roots: [], pack: resolve(dirname(fileURLToPath(import.meta.url)), ".."), out: null };
  const rest = argv.slice(2);
  for (let i = 0; i < rest.length; i++) {
    const a = rest[i];
    if (["install", "list", "uninstall", "check", "audit", "agy"].includes(a) && args.cmd === "install" && !args.sub) args.cmd = a;
    else if (args.cmd === "agy" && (a === "build" || a === "install")) args.sub = a;
    else if (a === "--dry-run") args.dryRun = true;
    else if (a === "--force") args.force = true;
    else if (a === "--link") args.link = true;
    else if (a === "--allow-dual") args.allowDual = true;
    else if (a === "--no-agy") args.noAgy = true;
    else if (a === "--strict") args.strict = true;
    else if (a === "--project") args.roots.push(resolve(process.cwd(), ".agents", "skills"));
    else if (a === "--root") args.roots.push(resolve(rest[++i]));
    else if (a === "--pack") args.pack = resolve(rest[++i]);
    else if (a === "--out") args.out = resolve(rest[++i]);
    else if (a === "--version") { console.log(DEPLOY_VERSION); process.exit(0); }
    else { console.error(`unknown argument: ${a}`); process.exit(2); }
  }
  return args;
}

const args = parseArgs(process.argv);
const pack = loadPack(args.pack);
if (args.cmd === "list") doList(pack, args);
else if (args.cmd === "uninstall") doUninstall(pack, args);
else if (args.cmd === "check") doCheck(pack);
else if (args.cmd === "audit") doAudit(pack, args);
else if (args.cmd === "agy") agyDeploy(pack, { ...args, cmd: args.sub ?? "build" });
else doInstall(pack, args);

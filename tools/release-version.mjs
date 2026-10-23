// One version authority per product, shared by preparation and artifact validation.
import { parse as parseToml, TomlError } from "smol-toml";

export class ReleaseError extends Error {}
export const stableVersion = (value) => typeof value === "string" && /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/.test(value);
export function greaterVersion(version, previous) {
  const old = previous.split(".").map(BigInt), next = version.split(".").map(BigInt);
  const differing = next.findIndex((value, index) => value !== old[index]);
  return differing !== -1 && next[differing] > old[differing];
}

function parsed(read, file, format) {
  try { return format === "json" ? JSON.parse(read(file)) : parseToml(read(file), { unsafeKeyBehaviour: "throw" }); }
  catch (error) {
    if (error instanceof SyntaxError || error instanceof TomlError) throw new ReleaseError(`invalid ${format} in ${file}: ${error.message}`);
    throw error;
  }
}

export function inspectVersions(read, has) {
  const files = [".claude-plugin/plugin.json", ".codex-plugin/plugin.json", ...(has("plugin.json") ? ["plugin.json"] : [])];
  const manifests = files.map((file) => ({ file, data: parsed(read, file, "json") }));
  const cargo = has("Cargo.toml") ? parsed(read, "Cargo.toml", "toml").package : null;
  const authority = cargo ?? manifests[0].data;
  if (!authority || typeof authority.name !== "string" || !/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(authority.name) || !stableVersion(authority.version)) throw new ReleaseError("invalid product identity or stable release version");
  for (const { file, data } of manifests) if (data.name !== authority.name || data.version !== authority.version) throw new ReleaseError(cargo ? `source Cargo authority differs from plugin identity/version: ${file}` : `source version mismatch: ${file}`);
  if (cargo) {
    const lock = parsed(read, "Cargo.lock", "toml");
    const roots = Array.isArray(lock.package) ? lock.package.filter((entry) => entry.name === cargo.name && entry.source === undefined) : [];
    if (roots.length !== 1 || roots[0].version !== cargo.version) throw new ReleaseError("source Cargo.lock root package version differs from Cargo authority");
  }
  for (const file of [".claude-plugin/marketplace.json", ".agents/plugins/marketplace.json"]) if (has(file)) {
    const catalog = parsed(read, file, "json");
    const entry = catalog.plugins?.find((item) => item.name === authority.name);
    if (entry && Object.hasOwn(entry, "version")) throw new ReleaseError(`competing marketplace version in ${file}`);
  }
  return { kind: cargo ? "cargo" : "skills", name: authority.name, version: authority.version, manifests, authorityFile: cargo ? "Cargo.toml" : files[0] };
}

function replaceVersion(block, oldVersion, version, file) {
  const matches = [...block.matchAll(/^(\s*version\s*=\s*)(["'])([^"'\r\n]+)\2/gm)];
  if (matches.length !== 1 || matches[0][3] !== oldVersion) throw new ReleaseError(`expected one literal version field in ${file}`);
  const match = matches[0];
  return block.slice(0, match.index) + match[1] + match[2] + version + match[2] + block.slice(match.index + match[0].length);
}

export function versionEdits(read, has, version, date) {
  const product = inspectVersions(read, has);
  if (!stableVersion(version)) throw new ReleaseError("version must be stable X.Y.Z without leading zeros");
  if (!greaterVersion(version, product.version)) throw new ReleaseError("candidate version must be greater than the source authority");
  const timestamp = Date.parse(`${date}T00:00:00Z`);
  if (!/^\d{4}-\d{2}-\d{2}$/.test(date) || !Number.isFinite(timestamp) || new Date(timestamp).toISOString().slice(0, 10) !== date) throw new ReleaseError("release date must be a valid YYYY-MM-DD");
  const files = new Map();
  for (const { file, data } of product.manifests) files.set(file, JSON.stringify({ ...data, version }, null, 2) + "\n");
  // Literal field replacement retains Cargo comments and formatting after semantic parsing.
  if (product.kind === "cargo") {
    const cargo = read("Cargo.toml"), section = /^\[package\][^\r\n]*\r?\n([\s\S]*?)(?=^\[|$(?![\s\S]))/m.exec(cargo);
    if (!section) throw new ReleaseError("Cargo.toml needs a literal [package] section");
    files.set("Cargo.toml", cargo.slice(0, section.index) + replaceVersion(section[0], product.version, version, "Cargo.toml [package]") + cargo.slice(section.index + section[0].length));
    const lock = read("Cargo.lock");
    let changed = 0;
    const rewritten = lock.replace(/^\[\[package\]\][^\r\n]*\r?\n[\s\S]*?(?=^\[|$(?![\s\S]))/gm, (block) => {
      const entry = parseToml(block).package[0];
      if (entry.name !== product.name || entry.source !== undefined) return block;
      changed++;
      return replaceVersion(block, product.version, version, "Cargo.lock root package");
    });
    if (changed !== 1) throw new ReleaseError("expected one root Cargo.lock package block");
    files.set("Cargo.lock", rewritten);
  }
  const changelog = read("CHANGELOG.md");
  const headers = [...changelog.matchAll(/^## \[Unreleased\][ \t]*\r?$/gm)];
  if (headers.length !== 1 || new RegExp(`^## \\[${version.replaceAll(".", "\\.")}\\]`, "m").test(changelog)) throw new ReleaseError("expected one Unreleased heading and no existing candidate section");
  const start = headers[0].index + headers[0][0].length;
  const following = changelog.slice(start), end = following.search(/^## \[/m);
  const notes = following.slice(0, end === -1 ? undefined : end).replace(/<!--[\s\S]*?-->/g, "").replace(/^\[(?:Unreleased|\d+\.\d+\.\d+)\]:.*$/gm, "").trim();
  if (!notes.replace(/^#+.*$|^\[[^\]]+\]:.*$/gm, "").trim()) throw new ReleaseError("Unreleased needs user-facing entries");
  let promoted = changelog.slice(0, start) + `\n\n## [${version}] - ${date}` + changelog.slice(start);
  promoted = promoted.replace(/^(\[Unreleased\]:\s+)(\S+\/compare\/)v([^\s]+)\.\.\.HEAD[ \t]*$/m, (_, prefix, url, previous) => `${prefix}${url}v${version}...HEAD\n[${version}]: ${url}v${previous}...v${version}`);
  files.set("CHANGELOG.md", promoted);
  const verified = inspectVersions((file) => files.get(file) ?? read(file), has);
  if (verified.version !== version) throw new ReleaseError("version edits did not update every authority surface");
  return { product, files, notes: notes + "\n", version, date };
}

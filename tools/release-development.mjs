// Construct/check development integration in an owned clone; never push or move user branches.
import { closeSync, existsSync, mkdirSync, openSync, readFileSync, realpathSync, renameSync, writeFileSync } from "node:fs";
import { createHash, randomUUID } from "node:crypto";
import { spawnSync } from "node:child_process";
import { basename, dirname, join, resolve, sep } from "node:path";
import { command, git } from "./release-git.mjs";
import { greaterVersion, inspectVersions, ReleaseError, stableVersion } from "./release-version.mjs";
import { productEnvironment, verifyProduct } from "./release-product.mjs";
const encode = (value) => JSON.stringify(value, null, 2) + "\n";
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
function versions(repo, sha) {
  const files = new Set(git(repo, ["ls-tree", "-r", "--name-only", "-z", sha]).split("\0"));
  return inspectVersions((file) => git(repo, ["show", `${sha}:${file}`]), (file) => files.has(file));
}
function ancestor(repo, older, newer) {
  const result = command(repo, ["merge-base", "--is-ancestor", older, newer]);
  if (result.status !== 0 && result.status !== 1) throw new ReleaseError("development ancestry unavailable; fetch complete history");
  return result.status === 0;
}
export function prepareDevelopment({ repo, remote = "origin", candidateSha, product, version, out, execute = false }) {
  if (typeof execute !== "boolean" || !/^[a-f0-9]{40}$/.test(candidateSha) || !["grey-matter", "tkt"].includes(product) || !stableVersion(version) || !/^[A-Za-z0-9][A-Za-z0-9._-]*$/.test(remote)) throw new ReleaseError("invalid development integration identity");
  repo = realpathSync.native(git(resolve(repo), ["rev-parse", "--show-toplevel"]).trim()); out = resolve(out);
  const candidate = versions(repo, candidateSha);
  if (candidate.name !== product || candidate.version !== version) throw new ReleaseError("development candidate identity differs");
  if (git(repo, ["rev-parse", "HEAD"]).trim() !== candidateSha || git(repo, ["status", "--porcelain=v1", "-z"])) throw new ReleaseError("development source must remain clean at the candidate commit");
  if (!execute) return { status: "pass", mode: "dry-run", candidateSha, plan: "fetch main, merge unchanged candidate in an owned clone, check versions and full product gate", writes: false };
  // Reject aliases and existing foreign output before writing even an ownership receipt.
  const parent = realpathSync.native(dirname(out)); out = join(parent, basename(out));
  if (out === repo || out.startsWith(repo + sep)) throw new ReleaseError("development output must be outside the candidate checkout");
  const urls = git(repo, ["remote", "get-url", "--push", "--all", remote]).trim().split("\n");
  if (urls.length !== 1 || !urls[0] || urls[0].startsWith("-")) throw new ReleaseError("development integration requires one push destination");
  const identity = hash(encode({ repo, candidateSha, product, version, remote, destination: urls[0] })), receipt = join(out, "development.json"), worktree = join(out, "worktree");
  let state;
  if (existsSync(out)) {
    if (!existsSync(receipt) || realpathSync.native(out) !== out || realpathSync.native(receipt) !== receipt) throw new ReleaseError("foreign or aliased development output");
    try { state = JSON.parse(readFileSync(receipt, "utf8")); }
    catch (error) { if (error instanceof SyntaxError) throw new ReleaseError("invalid development receipt"); throw error; }
    if (state?.owner !== "release-development" || state.schema !== 1 || state.identity !== identity) throw new ReleaseError("foreign development receipt or changed identity");
  } else {
    mkdirSync(out); state = { owner: "release-development", schema: 1, identity, candidateSha, phase: "initializing" };
  }
  const save = (fields) => { Object.assign(state, fields); const temp = join(out, `${randomUUID()}.tmp`); writeFileSync(temp, encode(state), { flag: "wx" }); renameSync(temp, receipt); };
  save({});
  if (!existsSync(worktree)) git(out, ["clone", "--no-local", "--no-checkout", repo, worktree]);
  if (realpathSync.native(worktree) !== worktree) throw new ReleaseError("aliased development clone");
  git(worktree, ["config", "core.hooksPath", join(out, "disabled-hooks")]);
  const head = git(worktree, ["rev-parse", "HEAD"]).trim();
  const expected = state.phase === "initializing" ? head === candidateSha && !git(worktree, ["ls-files", "-z"]) && !git(worktree, ["ls-files", "--others", "--exclude-standard", "-z"]) : !git(worktree, ["status", "--porcelain=v1", "-z"]) && (head === state.developmentSha || (state.phase === "checking" && head === state.checkoutFrom));
  if (!expected) throw new ReleaseError("owned development checkout changed; preserve it for inspection");
  git(worktree, ["fetch", "--no-tags", urls[0], "refs/heads/main"]);
  const mainSha = git(worktree, ["rev-parse", "FETCH_HEAD^{commit}"]).trim();
  let developmentSha;
  if (state.mainSha === mainSha && state.developmentSha) developmentSha = state.developmentSha;
  else if (ancestor(worktree, mainSha, candidateSha)) developmentSha = candidateSha;
  else if (ancestor(worktree, candidateSha, mainSha)) developmentSha = mainSha;
  else {
    const merge = command(worktree, ["merge-tree", "--write-tree", mainSha, candidateSha]);
    if (merge.status !== 0) throw new ReleaseError(`development merge conflicts or Git merge-tree is unavailable; inspect the owned clone before publication: ${merge.stdout}${merge.stderr}`);
    const tree = merge.stdout.trim();
    if (!/^[a-f0-9]{40}$/.test(tree)) throw new ReleaseError("unexpected development merge-tree output");
    const name = git(repo, ["config", "user.name"]).trim(), email = git(repo, ["config", "user.email"]).trim();
    developmentSha = git(worktree, ["-c", `user.name=${name}`, "-c", `user.email=${email}`, "commit-tree", tree, "-p", mainSha, "-p", candidateSha, "-m", `chore(release): integrate ${product} v${version}`]).trim();
  }
  const integrated = versions(worktree, developmentSha);
  if (integrated.name !== product || (integrated.version !== version && !greaterVersion(integrated.version, version))) throw new ReleaseError("development integration regresses the released version");
  for (const tag of git(worktree, ["tag", "--list", "v*"]).trim().split("\n")) if (stableVersion(tag.slice(1)) && greaterVersion(tag.slice(1), integrated.version)) throw new ReleaseError("development integration is behind an existing release tag");
  // Persist the immutable object before checkout so interrupted checks can reuse it.
  save({ phase: "checking", mainSha, developmentSha, checkoutFrom: head });
  git(worktree, ["checkout", "--detach", developmentSha]);
  const bootstrap = join(out, `bootstrap-${randomUUID()}.log`), fd = openSync(bootstrap, "wx");
  const bootstrapEnv = productEnvironment();
  Object.assign(bootstrapEnv, { NPM_CONFIG_USERCONFIG: join(out, "empty-user.npmrc"), NPM_CONFIG_GLOBALCONFIG: join(out, "empty-global.npmrc"), NPM_CONFIG_CACHE: join(out, "npm-cache"), NPM_CONFIG_REGISTRY: "https://registry.npmjs.org" });
  let installed;
  try { installed = spawnSync(process.platform === "win32" ? "npm.cmd" : "npm", ["ci", "--ignore-scripts", "--no-audit", "--no-fund"], { cwd: worktree, stdio: ["ignore", fd, fd], env: bootstrapEnv, shell: process.platform === "win32" }); }
  finally { closeSync(fd); }
  if (installed.error || installed.status !== 0) throw new ReleaseError(`development dependency setup failed; inspect ${bootstrap}`);
  const evidence = join(out, `product-${randomUUID()}`), report = verifyProduct({ repo: worktree, sha: developmentSha, out: evidence, execute: true });
  if (report.status !== "pass" || report.product !== product || report.candidateSha !== developmentSha) throw new ReleaseError(`development product gate failed; inspect ${evidence}`);
  // Import only the verified object. No source branch or named release tag moves.
  git(repo, ["fetch", "--no-tags", worktree, developmentSha]);
  const result = { status: "pass", mode: "development-integration", candidateSha, developmentSha, mainSha, evidence, reportSha256: hash(readFileSync(join(evidence, "product-evidence.json"))) };
  save({ phase: "verified", result }); return result;
}

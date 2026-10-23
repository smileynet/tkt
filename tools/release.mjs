#!/usr/bin/env node
// Prepare a local draft candidate; dry-run by default. No publishing or installation.
// Usage: node scripts/release.mjs prepare --repo DIR --version X.Y.Z --out DIR
//        [--execute] [--worktree EXISTING_WORKTREE] [--date YYYY-MM-DD]
//        node scripts/release.mjs verify --repo DIR --sha SHA --artifacts DIR --out DIR [--execute]
import { chmodSync, existsSync, lstatSync, mkdirSync, readFileSync, realpathSync, renameSync, rmSync, writeFileSync } from "node:fs";
import { randomUUID, createHash } from "node:crypto";
import { basename, dirname, isAbsolute, join, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { git, command } from "./release-git.mjs";
import { ReleaseError, versionEdits, stableVersion, greaterVersion } from "./release-version.mjs";
import { verifyCandidate } from "./release-verify.mjs";
import { buildArtifacts, verifyArtifacts } from "./release-artifacts.mjs";

const encode = (value) => JSON.stringify(value, null, 2) + "\n";
const hash = (text) => createHash("sha256").update(text).digest("hex");
const root = (repo) => realpathSync(git(repo, ["rev-parse", "--show-toplevel"]).trim());
function canonical(path) {
  if (existsSync(path)) return realpathSync(path);
  const parent = dirname(path);
  if (parent === path) throw new ReleaseError(`path has no accessible filesystem root: ${path}`);
  return join(canonical(parent), basename(path));
}
const common = (repo) => realpathSync(resolve(repo, git(repo, ["rev-parse", "--git-common-dir"]).trim()));
function worktreeIdentity(repo, worktree) {
  return { root: root(worktree), path: realpathSync(worktree), common: common(worktree), expectedCommon: common(repo) };
}
const head = (repo) => git(repo, ["rev-parse", "HEAD"]).trim();
const status = (repo) => git(repo, ["status", "--porcelain=v1", "-z"]);
function atomicWrite(path, bytes, { pending = `${path}.release-${randomUUID()}.tmp`, written = () => {} } = {}) {
  writeFileSync(pending, bytes, { flag: "wx" });
  if (existsSync(path)) chmodSync(pending, lstatSync(path).mode & 0o777);
  written();
  renameSync(pending, path);
}
function readState(out) {
  const path = join(out, "prepare-state.json");
  if (!existsSync(out)) return null;
  if (!existsSync(path)) throw new ReleaseError("output already exists without preparation ownership; choose a new output directory");
  let state;
  try { state = JSON.parse(readFileSync(path, "utf8")); }
  catch (error) {
    if (error instanceof SyntaxError) throw new ReleaseError("invalid preparation state JSON");
    throw error;
  }
  if (state.schema !== 1 || !/^[a-f0-9]{40}$/.test(state.sourceSha)) throw new ReleaseError("invalid preparation state");
  return state;
}
function checkTags(repo, version) {
  const tags = git(repo, ["tag", "--list", "v*"]).trim().split("\n");
  if (tags.some((tag) => stableVersion(tag.slice(1)) && !greaterVersion(version, tag.slice(1)))) throw new ReleaseError("candidate version must exceed every existing release tag");
}

function plan(options, previous) {
  const repo = root(resolve(options.repo)), out = canonical(resolve(options.out));
  if (status(repo)) throw new ReleaseError("source working tree must be clean");
  if (previous && (previous.repo !== repo || previous.version !== options.version)) throw new ReleaseError("existing preparation belongs to a different repository or version");
  const sourceSha = previous?.sourceSha ?? head(repo);
  const tracked = new Set(git(repo, ["ls-tree", "-r", "--name-only", "-z", sourceSha]).split("\0").filter(Boolean));
  const read = (file) => git(repo, ["show", `${sourceSha}:${file}`]);
  const date = options.date ?? previous?.date ?? new Date().toISOString().slice(0, 10);
  const edits = versionEdits(read, (file) => tracked.has(file), options.version, date);
  checkTags(repo, options.version);
  const branch = `codex/release-${edits.product.name}-${options.version}-${sourceSha.slice(0, 8)}`;
  const worktree = canonical(resolve(options.worktree ?? previous?.worktree ?? join(out, "worktree")));
  if (worktree === repo || (previous && (previous.worktree !== worktree || previous.branch !== branch || previous.date !== date))) throw new ReleaseError("candidate worktree or release identity differs from preparation state");
  if (!previous && existsSync(worktree) && (root(worktree) !== realpathSync(worktree) || common(worktree) !== common(repo) || head(worktree) !== sourceSha || status(worktree))) throw new ReleaseError(`provided worktree must be clean at the source commit in this repository: ${JSON.stringify(worktreeIdentity(repo, worktree))}`);
  const inside = relative(repo, out);
  if (!inside.startsWith(".." + sep) && inside !== ".." && !isAbsolute(inside) && command(repo, ["check-ignore", "-q", inside]).status !== 0) throw new ReleaseError("output inside the source repository must be gitignored");
  const fileHashes = Object.fromEntries([...edits.files].map(([file, text]) => [file, hash(text)]));
  if (previous && encode(previous.fileHashes) !== encode(fileHashes)) throw new ReleaseError("planned version bytes differ from recorded preparation");
  return { repo, out, sourceSha, date, branch, worktree, edits, read, fileHashes };
}

function verifyDraft(context, sha) {
  const { worktree, sourceSha, edits } = context;
  const parents = git(worktree, ["show", "-s", "--format=%P", sha]).trim();
  if (parents !== sourceSha) throw new ReleaseError("candidate history diverged from the recorded source");
  const changed = git(worktree, ["diff", "--name-only", "-z", sourceSha, sha]).split("\0").filter(Boolean);
  if (changed.some((file) => !edits.files.has(file))) throw new ReleaseError("candidate commit contains unrelated changes");
  for (const [file, text] of edits.files) if (git(worktree, ["show", `${sha}:${file}`]) !== text) throw new ReleaseError(`candidate committed bytes differ: ${file}`);
  if (status(worktree)) throw new ReleaseError("candidate worktree has uncommitted changes");
}

export function prepare(options) {
  const previous = readState(canonical(resolve(options.out)));
  const context = plan(options, previous);
  const { repo, out, sourceSha, date, branch, worktree, edits, fileHashes } = context;
  if (!options.execute) return { status: "pass", mode: "dry-run", publishable: false, sourceSha, currentVersion: edits.product.version, version: options.version, date, branch, worktree, files: fileHashes, releaseNotes: edits.notes, fetchOnExecute: command(repo, ["remote", "get-url", "origin"]).status === 0, validation: "not-run" };
  const progress = options.progress ?? (() => {});
  if (command(repo, ["remote", "get-url", "origin"]).status === 0) {
    progress("fetch");
    git(repo, ["fetch", "--tags", "origin"]);
    checkTags(repo, options.version);
    let upstream = command(repo, ["rev-parse", "--verify", "@{upstream}"]);
    if (upstream.status !== 0) {
      const remoteHead = command(repo, ["symbolic-ref", "refs/remotes/origin/HEAD"]);
      const ref = remoteHead.status === 0 ? remoteHead.stdout.trim() : `refs/remotes/origin/${git(repo, ["symbolic-ref", "--short", "HEAD"]).trim()}`;
      upstream = command(repo, ["rev-parse", "--verify", ref]);
    }
    if (upstream.status !== 0 || command(repo, ["merge-base", "--is-ancestor", upstream.stdout.trim(), head(repo)]).status !== 0) throw new ReleaseError("source is behind, diverged or lacks a fetched upstream");
  }
  if (status(repo) || (!previous && head(repo) !== sourceSha)) throw new ReleaseError("source changed during preparation planning");
  let state = previous;
  if (!state) {
    mkdirSync(dirname(out), { recursive: true });
    mkdirSync(out);
    state = { schema: 1, repo, sourceSha, version: options.version, date, branch, worktree, fileHashes, phase: "planned", validation: { product: "not-run", native: "not-run", scope: "not-reviewed", platforms: [] } };
    atomicWrite(join(out, "prepare-state.json"), encode(state));
  }
  const save = (phase, fields = {}) => {
    Object.assign(state, fields, { phase });
    atomicWrite(join(out, "prepare-state.json"), encode(state));
    progress(phase);
  };
  if (!existsSync(worktree)) {
    git(repo, ["worktree", "add", "-b", branch, worktree, sourceSha]);
  } else {
    if (root(worktree) !== realpathSync(worktree) || common(worktree) !== common(repo)) throw new ReleaseError(`candidate is not a worktree of this repository: ${JSON.stringify(worktreeIdentity(repo, worktree))}`);
    const current = command(worktree, ["symbolic-ref", "--short", "HEAD"]);
    if (current.status !== 0 || current.stdout.trim() !== branch) {
      if (previous || head(worktree) !== sourceSha || status(worktree)) throw new ReleaseError("provided worktree is not clean at the planned source");
      git(worktree, ["switch", "-c", branch]);
    }
  }
  if (state.pendingFile) {
    const pending = state.pendingFile, prefix = `${pending.file}.release-`;
    if (head(worktree) !== sourceSha || !edits.files.has(pending.file) || typeof pending.temp !== "string" || !pending.temp.startsWith(prefix) || !/^[a-f0-9-]{36}\.tmp$/.test(pending.temp.slice(prefix.length)) || pending.sha256 !== fileHashes[pending.file]) throw new ReleaseError("invalid pending-file ownership receipt");
    const current = readFileSync(join(worktree, pending.file), "utf8");
    if (![context.read(pending.file), edits.files.get(pending.file)].includes(current)) throw new ReleaseError("pending candidate file was changed outside preparation");
    const temp = join(worktree, pending.temp);
    if (existsSync(temp)) {
      if (!lstatSync(temp).isFile()) throw new ReleaseError("pending temporary path is not an owned regular file");
      rmSync(temp);
    }
    delete state.pendingFile;
    atomicWrite(join(out, "prepare-state.json"), encode(state));
  }
  if (head(worktree) === sourceSha) {
    const entries = status(worktree).split("\0").filter(Boolean);
    if (entries.some((entry) => ![" M", "M ", "MM"].includes(entry.slice(0, 2)) || !edits.files.has(entry.slice(3)))) throw new ReleaseError("candidate has unrelated or untracked changes");
    for (const [file, proposed] of edits.files) {
      const original = context.read(file), actual = readFileSync(join(worktree, file), "utf8"), staged = git(worktree, ["show", `:${file}`]);
      if (![original, proposed].includes(actual) || ![original, proposed].includes(staged)) throw new ReleaseError(`candidate file was modified outside preparation: ${file}`);
    }
    save("worktree");
    for (const [file, text] of edits.files) {
      const temp = `${file}.release-${randomUUID()}.tmp`;
      state.pendingFile = { file, temp, sha256: fileHashes[file] };
      atomicWrite(join(out, "prepare-state.json"), encode(state));
      progress("file-intent", file);
      atomicWrite(join(worktree, file), text, { pending: join(worktree, temp), written: () => progress("file-written", file) });
      progress("file-promoted", file);
      delete state.pendingFile;
      atomicWrite(join(out, "prepare-state.json"), encode(state));
      progress("file", file);
    }
    save("versioned");
    git(worktree, ["add", "--", ...edits.files.keys()]);
    const tree = git(worktree, ["write-tree"]).trim();
    const author = git(repo, ["var", "GIT_AUTHOR_IDENT"]).trim().replace(/ \d+ [+-]\d{4}$/, "");
    const match = /^(.*) <([^<>]+)>$/.exec(author);
    if (!match) throw new ReleaseError("Git author identity could not be resolved");
    const env = { GIT_AUTHOR_NAME: match[1], GIT_AUTHOR_EMAIL: match[2], GIT_COMMITTER_NAME: match[1], GIT_COMMITTER_EMAIL: match[2], GIT_AUTHOR_DATE: `${date}T12:00:00Z`, GIT_COMMITTER_DATE: `${date}T12:00:00Z` };
    // Draft plumbing avoids invoking user hooks or publication helpers; verification is pending.
    const sha = git(worktree, ["commit-tree", tree, "-p", sourceSha, "-m", `chore(release): v${options.version}`], { env }).trim();
    git(worktree, ["update-ref", `refs/heads/${branch}`, sha, sourceSha]);
  }
  const candidateSha = head(worktree);
  verifyDraft(context, candidateSha);
  if (state.candidateSha && state.candidateSha !== candidateSha) throw new ReleaseError("candidate identity changed after preparation");
  save("committed", { candidateSha });
  const notes = join(out, "release-notes.md");
  if (existsSync(notes) && readFileSync(notes, "utf8") !== edits.notes) throw new ReleaseError("existing release notes differ from the candidate changelog");
  atomicWrite(notes, edits.notes);
  buildArtifacts({ repo: worktree, ref: candidateSha, out: join(out, "artifacts") });
  const metadata = verifyArtifacts(join(out, "artifacts"));
  if (metadata.candidateSha !== candidateSha) throw new ReleaseError("artifact identity differs from draft commit");
  save("draft", { notesSha256: hash(edits.notes) });
  return { status: "pass", mode: "draft", publishable: false, version: state.version, candidateSha, worktree, output: out, validation: state.validation };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const args = process.argv.slice(2), mode = args.shift(), options = {};
    const named = mode === "prepare" ? ["--repo", "--version", "--out", "--worktree", "--date"] : mode === "verify" ? ["--repo", "--sha", "--artifacts", "--out", "--codex", "--claude", "--agy"] : [];
    while (args.length) {
      const key = args.shift();
      if (key === "--execute") options.execute = true;
      else if (key === "--dry-run") options.dryRun = true;
      else if (named.includes(key) && args.length) options[key.slice(2)] = args.shift();
      else throw new ReleaseError(`unknown/incomplete option: ${key}`);
    }
    if (!["prepare", "verify"].includes(mode) || !options.repo || !options.out || (mode === "prepare" ? !options.version : !options.sha || !options.artifacts) || (options.execute && options.dryRun)) throw new ReleaseError("usage: prepare --repo DIR --version X.Y.Z --out DIR OR verify --repo DIR --sha SHA --artifacts DIR --out DIR [--execute | --dry-run]");
    options.progress = (phase) => { if (mode === "verify" || ["fetch", "worktree", "versioned", "committed", "draft"].includes(phase)) console.error(`${mode}: ${phase}`); };
    const result = mode === "prepare" ? prepare(options) : await verifyCandidate(options);
    console.log(encode(result)); process.exitCode = result.status === "fail" ? 1 : 0;
  } catch (error) {
    const validation = error instanceof ReleaseError;
    console.error(encode({ status: validation ? "fail" : "error", errors: [error.message] }));
    process.exitCode = validation ? 1 : 2;
  }
}

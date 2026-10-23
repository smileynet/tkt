// Scope every release Git operation to its explicit repository, including in hooks.
import { spawnSync } from "node:child_process";
import { ReleaseError } from "./release-version.mjs";

export function gitEnvironment(extra = {}) {
  const env = { ...process.env, GIT_OPTIONAL_LOCKS: "0", ...extra };
  for (const key of ["GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_COMMON_DIR", "GIT_OBJECT_DIRECTORY", "GIT_ALTERNATE_OBJECT_DIRECTORIES"]) delete env[key];
  return env;
}

export function command(repo, args, options = {}) {
  const env = gitEnvironment(options.env);
  const result = spawnSync("git", ["-C", repo, ...args], { encoding: "utf8", maxBuffer: 32 * 1024 * 1024, ...options, env });
  if (result.error) throw result.error;
  return result;
}

export function git(repo, args, options) {
  const result = command(repo, args, options);
  if (result.status !== 0) throw new ReleaseError(`git ${args[0]} failed: ${result.stderr.toString().trim()}`);
  return result.stdout;
}

// Push only a sealed candidate branch and annotated tag; stable is a later step.
export function pushCandidateRefs({ repo, remote = "origin", branch, tag, candidateSha, tagObject, execute = false }) {
  if (typeof execute !== "boolean" || !/^[a-f0-9]{40}$/.test(candidateSha) || !/^[a-f0-9]{40}$/.test(tagObject)) throw new ReleaseError("invalid candidate ref identity or execution mode");
  for (const ref of [branch, tag]) {
    if (typeof ref !== "string" || !ref || ref.startsWith("-") || command(repo, ["check-ref-format", ref]).status !== 0) throw new ReleaseError("invalid full release ref");
  }
  if (!branch.startsWith("refs/heads/") || branch === "refs/heads/stable" || !tag.startsWith("refs/tags/")) throw new ReleaseError("candidate push requires a branch other than stable and a tag");
  if (typeof remote !== "string" || !/^[A-Za-z0-9][A-Za-z0-9._-]*$/.test(remote)) throw new ReleaseError("invalid remote name");
  const urls = git(repo, ["remote", "get-url", "--push", "--all", remote]).trim().split("\n");
  if (urls.length !== 1 || !urls[0] || urls[0].startsWith("-")) throw new ReleaseError("candidate push requires exactly one push URL");
  const url = urls[0];
  if (git(repo, ["cat-file", "-t", candidateSha]).trim() !== "commit" || git(repo, ["cat-file", "-t", tagObject]).trim() !== "tag" || git(repo, ["rev-parse", `${tagObject}^{commit}`]).trim() !== candidateSha) throw new ReleaseError("annotated tag does not resolve to candidate commit");
  function inspect() {
    const refs = new Map(git(repo, ["ls-remote", "--refs", url, branch, tag]).trim().split("\n").filter(Boolean).map((line) => {
      const fields = line.split(/\s+/);
      if (fields.length !== 2 || !/^[a-f0-9]{40}$/.test(fields[0]) || ![branch, tag].includes(fields[1])) throw new ReleaseError("invalid remote ref response");
      return [fields[1], fields[0]];
    }));
    if (refs.has(tag) && refs.get(tag) !== tagObject) throw new ReleaseError("remote release tag differs; refusing replacement");
    const old = refs.get(branch);
    if (old && old !== candidateSha && command(repo, ["merge-base", "--is-ancestor", old, candidateSha]).status !== 0) throw new ReleaseError("remote candidate branch is divergent or ancestry is unavailable");
    return { branch: old ?? null, tag: refs.get(tag) ?? null };
  }
  let observed = inspect();
  const matches = () => observed.branch === candidateSha && observed.tag === tagObject;
  if (execute && !matches()) {
    const push = command(repo, ["-c", "push.followTags=false", "push", "--atomic", "--no-follow-tags", "--recurse-submodules=no", url, `${candidateSha}:${branch}`, `${tagObject}:${tag}`]);
    observed = inspect();
    if (!matches()) throw new ReleaseError(`atomic candidate push was not observed (exit ${push.status}); inspect and resume the same identity; no sequential fallback`);
  }
  return { status: "pass", mode: execute ? "candidate-push" : "dry-run", complete: matches(), refs: observed };
}

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

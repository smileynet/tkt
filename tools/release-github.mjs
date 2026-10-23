// GitHub.com adapter for the release reconciler; writes require explicit opt-in.
import { spawnSync } from "node:child_process";
import { ReleaseError } from "./release-version.mjs";

export function githubReleaseApi({ execute = false, run = (args, input) => spawnSync("gh", args, { input, encoding: "utf8", maxBuffer: 32 * 1024 * 1024, env: { ...process.env, GH_HOST: "github.com", GH_PROMPT_DISABLED: "1" } }) } = {}) {
  if (typeof execute !== "boolean") throw new ReleaseError("GitHub execution mode must be boolean");
  const repository = (repo) => {
    if (typeof repo !== "string" || !/^[A-Za-z0-9][A-Za-z0-9_.-]*\/[A-Za-z0-9][A-Za-z0-9_.-]*$/.test(repo)) throw new ReleaseError("invalid GitHub repository");
    return `repos/${repo}`;
  };
  const id = (value) => {
    if (!Number.isSafeInteger(value) || value <= 0) throw new ReleaseError("invalid GitHub release ID");
    return value;
  };
  function invoke(args, input) {
    const result = run(args, input);
    if (result.error || result.status !== 0) throw new ReleaseError("GitHub command failed; inspect remote state before resuming");
    return result.stdout;
  }
  function request(endpoint, { method = "GET", body, binary = false, pages = false } = {}) {
    if (method !== "GET" && !execute) throw new ReleaseError("GitHub adapter is read-only");
    const args = ["api", "--hostname", "github.com", "--method", method, "-H", "Accept: application/vnd.github+json", "-H", "X-GitHub-Api-Version: 2026-03-10"];
    if (pages) args.push("--paginate", "--slurp");
    if (body !== undefined) args.push("-H", `Content-Type: ${binary ? "application/octet-stream" : "application/json"}`, "--input", "-");
    args.push(endpoint);
    let result;
    try { result = JSON.parse(invoke(args, body === undefined ? undefined : binary ? body : JSON.stringify(body))); }
    catch (error) {
      if (error instanceof SyntaxError) throw new ReleaseError("invalid JSON from GitHub");
      throw error;
    }
    if (!pages) return result;
    if (!Array.isArray(result) || result.some((page) => !Array.isArray(page))) throw new ReleaseError("invalid paginated GitHub response");
    return result.flat();
  }
  function writable(repo) {
    const result = request(repository(repo));
    if (result?.full_name?.toLowerCase() !== repo.toLowerCase() || result.permissions?.push !== true) throw new ReleaseError("authenticated repository push access is required to discover drafts");
  }
  return {
    tag(repo, tag) {
      const base = repository(repo);
      if (typeof tag !== "string" || !/^v[0-9]+\.[0-9]+\.[0-9]+$/.test(tag)) throw new ReleaseError("invalid release tag");
      const ref = request(`${base}/git/ref/tags/${tag}`);
      if (ref?.ref !== `refs/tags/${tag}` || ref.object?.type !== "tag" || !/^[a-f0-9]{40}$/.test(ref.object.sha)) throw new ReleaseError("remote release tag must be annotated");
      const object = request(`${base}/git/tags/${ref.object.sha}`);
      if (object?.sha !== ref.object.sha || object.tag !== tag || object.object?.type !== "commit" || !/^[a-f0-9]{40}$/.test(object.object.sha)) throw new ReleaseError("remote annotated tag identity differs");
      return { sha: object.object.sha, annotated: true };
    },
    releases(repo) { writable(repo); return request(`${repository(repo)}/releases?per_page=100`, { pages: true }); },
    assets(repo, releaseId) { return request(`${repository(repo)}/releases/${id(releaseId)}/assets?per_page=100`, { pages: true }); },
    immutableEnabled(repo) {
      const result = request(`${repository(repo)}/immutable-releases`);
      if (typeof result?.enabled !== "boolean" || typeof result.enforced_by_owner !== "boolean") throw new ReleaseError("unknown repository immutability policy");
      return result.enabled;
    },
    createDraft(repo, { tag, title, notes }) {
      if (!execute) throw new ReleaseError("GitHub adapter is read-only");
      repository(repo);
      if (typeof tag !== "string" || !/^v[0-9]+\.[0-9]+\.[0-9]+$/.test(tag) || typeof title !== "string" || !title.trim() || typeof notes !== "string" || !notes.trim()) throw new ReleaseError("invalid draft identity");
      invoke(["release", "create", tag, "--repo", repo, "--verify-tag", "--draft", "--title", title, "--notes-file", "-"], notes);
    },
    upload(repo, releaseId, { name, bytes }) {
      const base = repository(repo);
      return request(`https://uploads.github.com/${base}/releases/${id(releaseId)}/assets?name=${encodeURIComponent(name)}`, { method: "POST", body: bytes, binary: true });
    },
    publish(repo, releaseId) { return request(`${repository(repo)}/releases/${id(releaseId)}`, { method: "PATCH", body: { draft: false } }); },
  };
}

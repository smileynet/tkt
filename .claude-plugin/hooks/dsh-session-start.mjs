// Native DeepSeek Harness (dsh) SessionStart hook: injects .tickets/ frontier
// and in-progress tickets via agent/created + agent.inject(...).
import { randomUUID } from "node:crypto";
import { statSync } from "node:fs";
import { isAbsolute, join } from "node:path";
import { spawnSync } from "node:child_process";

function hookPath() {
  const sep = process.platform === "win32" ? ";" : ":";
  const home = process.env.HOME ?? process.env.USERPROFILE;
  const roots = ["/opt/homebrew/bin", "/usr/local/bin"];
  if (home) roots.unshift(join(home, ".cargo", "bin"), join(home, ".local", "bin"));
  return [process.env.PATH, ...roots].filter(Boolean).join(sep);
}

function collectContext(root) {
  if (!root || !isAbsolute(root)) return null;
  try {
    if (!statSync(join(root, ".tickets")).isDirectory()) return null;
  } catch (error) {
    if (error.code === "ENOENT") return null;
    throw error;
  }
  const output = [];
  for (const args of [["ready", "--json"], ["query", "--status", "in_progress"]]) {
    const result = spawnSync("tkt", args, {
      cwd: root,
      encoding: "utf8",
      timeout: 5000,
      maxBuffer: 256 * 1024,
      env: { ...process.env, PATH: hookPath(), NO_COLOR: "1", TKT_ADVICE: "0" },
    });
    if (result.error?.code === "ENOENT") return null;
    if (result.status !== 0) return null;
    output.push(result.stdout.trim());
  }
  return `tkt frontier\n${output[0] || "None."}\n\ntkt in-progress tickets\n${output[1] || "None."}`;
}

export const name = "tkt-session-start";
export const inject = ["agents", "workingDirectory"];

export function apply(ctx) {
  ctx.on("agent/created", async ({ agent, signal }) => {
    if (signal?.aborted) return;
    let cwd;
    try {
      cwd = await ctx.workingDirectory.ensure(agent, signal);
    } catch {
      return;
    }
    if (signal?.aborted) return;
    const text = collectContext(cwd);
    if (!text) return;
    agent.inject(Object.freeze({
      id: randomUUID(),
      role: "user",
      content: Object.freeze([Object.freeze({ type: "text", text })]),
      source: Object.freeze({ kind: "tkt-frontier", form: "instructions" }),
    }));
  });
}

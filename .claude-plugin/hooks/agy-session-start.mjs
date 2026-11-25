#!/usr/bin/env node
// Antigravity SessionStart context. Usage: node ./hooks/agy-session-start.mjs < hook-input.json
import { readFileSync, statSync } from "node:fs";
import { isAbsolute, join } from "node:path";
import { spawnSync } from "node:child_process";

function hookPath() {
  const sep = process.platform === "win32" ? ";" : ":";
  const home = process.env.HOME ?? process.env.USERPROFILE;
  const roots = ["/opt/homebrew/bin", "/usr/local/bin"];
  if (home) roots.unshift(join(home, ".cargo", "bin"), join(home, ".local", "bin"));
  return [process.env.PATH, ...roots].filter(Boolean).join(sep);
}

function findTicketRoot(paths) {
  for (const candidate of paths) {
    if (typeof candidate !== "string" || !isAbsolute(candidate)) continue;
    try {
      if (statSync(join(candidate, ".tickets")).isDirectory()) return candidate;
    } catch (error) {
      if (error.code !== "ENOENT") throw error;
    }
  }
  return null;
}

try {
  const raw = readFileSync(0, "utf8").trim();
  const input = raw ? JSON.parse(raw) : {};
  const candidates = Array.isArray(input.workspacePaths) ? input.workspacePaths : [];
  const root = findTicketRoot(candidates);
  if (!root) process.exit(0);
  const output = [];
  for (const args of [["ready", "--json"], ["query", "--status", "in_progress"]]) {
    const result = spawnSync("tkt", args, {
      cwd: root,
      encoding: "utf8",
      timeout: 5000,
      maxBuffer: 256 * 1024,
      env: { ...process.env, PATH: hookPath(), NO_COLOR: "1", TKT_ADVICE: "0" },
    });
    if (result.error?.code === "ENOENT") process.exit(0);
    if (result.status !== 0) throw new Error(`tkt ${args.join(" ")} failed: ${result.error?.message ?? result.stderr}`);
    output.push(result.stdout.trim());
  }
  const context = `tkt frontier\n${output[0] || "None."}\n\ntkt in-progress tickets\n${output[1] || "None."}`;
  console.log(JSON.stringify({ injectSteps: [{ systemMessage: { systemMessage: context } }] }));
} catch (error) {
  console.error(`tkt Antigravity SessionStart: ${error.message}`);
  process.exit(1);
}

#!/usr/bin/env node
// Claude SessionStart context. Usage: node session-start.mjs < hook-input.json
import { readFileSync, statSync } from "node:fs";
import { isAbsolute, join } from "node:path";
import { spawnSync } from "node:child_process";

try {
  const input = JSON.parse(readFileSync(0, "utf8"));
  const root = process.env.CLAUDE_PROJECT_DIR;
  if (input.hook_event_name !== "SessionStart" || !root || !isAbsolute(root)) process.exit(0);
  try {
    if (!statSync(join(root, ".tickets")).isDirectory()) process.exit(0);
  } catch (error) {
    if (error.code === "ENOENT") process.exit(0);
    throw error;
  }
  const output = [];
  for (const args of [["ready", "--json"], ["query", "--status", "in_progress"]]) {
    const result = spawnSync("tkt", args, { cwd: root, encoding: "utf8", timeout: 5000, maxBuffer: 256 * 1024, env: { ...process.env, NO_COLOR: "1", TKT_ADVICE: "0" } });
    if (result.error?.code === "ENOENT") process.exit(0);
    if (result.status !== 0) throw new Error(`tkt ${args.join(" ")} failed: ${result.error?.message ?? result.stderr}`);
    output.push(result.stdout.trim());
  }
  const context = `tkt frontier\n${output[0] || "None."}\n\ntkt in-progress tickets\n${output[1] || "None."}`;
  // ZCode parses hook stdout as a strict hookEventName JSON union and discards
  // plain text; Claude Code injects plain stdout directly. ZCode also sets the
  // CLAUDE_* tokens, so a ZCODE_* token is the host discriminator.
  if (process.env.ZCODE_SESSION_ID || process.env.ZCODE_PLUGIN_ROOT) {
    console.log(JSON.stringify({ hookEventName: "SessionStart", additionalContext: context }));
  } else {
    console.log(context);
  }
} catch (error) {
  console.error(`tkt SessionStart: ${error.message}`);
  process.exit(1);
}

// Native DeepSeek Harness (dsh) plugin hook for tkt:
// - SessionStart frontier injection via agent/created (skipped on resume for V4.1 KV-cache hygiene)
// - Monotonic ctx.tools.guard() blocking direct status: done edits in .tickets/*.md
// - Read-only, parallel-safe tkt_ready and tkt_query tools compatible with native and PTC modes
import { randomUUID } from "node:crypto";
import { statSync } from "node:fs";
import { isAbsolute, join } from "node:path";
import { spawnSync } from "node:child_process";

const TICKET_FILE = /(?:^|\/)\.tickets\/[^/]+\.md$/i;
const STATUS_DONE = /^[ \t]*(?:\+[ \t]*)?status[ \t]*:[ \t]*(?:["']done["']|done)(?:[ \t]*(?:#.*)?)$/m;
const GUARD_REASON = 'Refusing direct edit setting status: done in .tickets/*.md. Use `tkt close <id> --check-all --evidence "..." --resolution "..."` instead.';

function hookPath() {
  const sep = process.platform === "win32" ? ";" : ":";
  const home = process.env.HOME ?? process.env.USERPROFILE;
  const roots = ["/opt/homebrew/bin", "/usr/local/bin"];
  if (home) roots.unshift(join(home, ".cargo", "bin"), join(home, ".local", "bin"));
  return [process.env.PATH, ...roots].filter(Boolean).join(sep);
}

function hasTicketsDir(root) {
  if (!root || !isAbsolute(root)) return false;
  try {
    return statSync(join(root, ".tickets")).isDirectory();
  } catch {
    return false;
  }
}

function runTkt(root, args) {
  const result = spawnSync("tkt", args, {
    cwd: root,
    encoding: "utf8",
    timeout: 5000,
    maxBuffer: 256 * 1024,
    env: { ...process.env, PATH: hookPath(), NO_COLOR: "1", TKT_ADVICE: "0" },
  });
  if (result.error?.code === "ENOENT") return null;
  if (result.status !== 0) return null;
  return result.stdout.trim();
}

function collectContext(root) {
  if (!hasTicketsDir(root)) return null;
  const ready = runTkt(root, ["ready", "--json"]);
  if (ready === null) return null;
  const wip = runTkt(root, ["query", "--status", "in_progress"]);
  if (wip === null) return null;
  return `tkt frontier\n${ready || "None."}\n\ntkt in-progress tickets\n${wip || "None."}`;
}

export function guardTicketStatusDone(exec) {
  const args = exec?.arguments;
  if (!args || typeof args !== "object") return undefined;
  const paths = [args.file_path, args.path, args.filePath, args.target_file, args.filename]
    .filter((v) => typeof v === "string" && v.length > 0)
    .map((p) => p.replaceAll("\\", "/"));
  if (!paths.some((p) => TICKET_FILE.test(p))) return undefined;
  const texts = [];
  for (const key of ["content", "new_string", "new_str", "newText", "replacement", "patch", "diff"]) {
    if (typeof args[key] === "string") texts.push(args[key]);
  }
  if (Array.isArray(args.edits)) {
    for (const edit of args.edits) {
      if (!edit || typeof edit !== "object") continue;
      for (const key of ["content", "new_string", "new_str", "newText", "replacement"]) {
        if (typeof edit[key] === "string") texts.push(edit[key]);
      }
    }
  }
  return texts.some((t) => STATUS_DONE.test(t)) ? GUARD_REASON : undefined;
}

async function resolveCwd(ctx, agent, signal) {
  if (!ctx.workingDirectory || typeof ctx.workingDirectory.ensure !== "function") return null;
  try {
    return await ctx.workingDirectory.ensure(agent, signal);
  } catch {
    return null;
  }
}

function registerTools(ctx) {
  if (!ctx.tools) return;
  if (typeof ctx.tools.guard === "function") {
    ctx.tools.guard(guardTicketStatusDone);
  }
  if (typeof ctx.tools.register !== "function") return;
  const outputSpec = {
    schema: {
      type: "object",
      properties: { stdout: { type: "string" } },
      required: ["stdout"],
      additionalProperties: false,
    },
    render: (_args, value) => [{ type: "text", text: value.stdout }],
  };
  const when = async ({ agent, signal }) => hasTicketsDir(await resolveCwd(ctx, agent, signal));

  ctx.tools.register({
    name: "tkt_ready",
    description: "List ready (frontier) tickets in the current project (.tickets/) as JSON.",
    parameters: {
      type: "object",
      properties: {
        tag: { type: "string", description: "Optional tag filter" },
      },
      additionalProperties: false,
    },
    output: outputSpec,
    isConcurrencySafe: () => true,
    timeoutMs: 10000,
    execute: async (args, exec) => {
      const cwd = await resolveCwd(ctx, exec?.agent, exec?.signal);
      if (!hasTicketsDir(cwd)) return { stdout: "[]" };
      const cliArgs = ["ready", "--json"];
      if (typeof args?.tag === "string" && args.tag.trim()) cliArgs.push("--tag", args.tag.trim());
      const out = runTkt(cwd, cliArgs);
      if (out === null) throw new Error("tkt ready failed or tkt CLI not found on PATH");
      return { stdout: out || "[]" };
    },
  }, { when });

  ctx.tools.register({
    name: "tkt_query",
    description: "Query project tickets in .tickets/ by optional status or tag as JSON.",
    parameters: {
      type: "object",
      properties: {
        status: { type: "string", enum: ["open", "in_progress", "done", "backlog"], description: "Filter by ticket status" },
        tag: { type: "string", description: "Filter by tag" },
        allContext: { type: "boolean", description: "Ignore active tag context" },
      },
      additionalProperties: false,
    },
    output: outputSpec,
    isConcurrencySafe: () => true,
    timeoutMs: 10000,
    execute: async (args, exec) => {
      const cwd = await resolveCwd(ctx, exec?.agent, exec?.signal);
      if (!hasTicketsDir(cwd)) return { stdout: "[]" };
      const cliArgs = ["query", "--json"];
      if (typeof args?.status === "string" && args.status.trim()) cliArgs.push("--status", args.status.trim());
      if (typeof args?.tag === "string" && args.tag.trim()) cliArgs.push("--tag", args.tag.trim());
      if (args?.allContext === true) cliArgs.push("--all-context");
      const out = runTkt(cwd, cliArgs);
      if (out === null) throw new Error("tkt query failed or tkt CLI not found on PATH");
      return { stdout: out || "[]" };
    },
  }, { when });
}

export const name = "tkt-session-start";
export const inject = ["agents", "workingDirectory"];

export function apply(ctx) {
  ctx.on("agent/created", async ({ agent, source, signal }) => {
    if (signal?.aborted || source === "resume") return;
    const cwd = await resolveCwd(ctx, agent, signal);
    if (!cwd || signal?.aborted) return;
    const text = collectContext(cwd);
    if (!text) return;
    agent.inject(Object.freeze({
      id: randomUUID(),
      role: "user",
      content: Object.freeze([Object.freeze({ type: "text", text })]),
      source: Object.freeze({ kind: "tkt-frontier", form: "instructions" }),
    }));
  });

  if (typeof ctx.inject === "function") {
    ctx.inject(["tools"], (toolCtx) => registerTools(toolCtx));
  } else if (ctx.tools) {
    registerTools(ctx);
  }
}

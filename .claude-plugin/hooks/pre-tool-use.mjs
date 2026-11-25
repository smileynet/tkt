#!/usr/bin/env node
// Shared PreToolUse guard for Claude Code, Qwen Code, and ZCode:
// blocks direct file edits that write `status: done` into `.tickets/*.md`.
import { readFileSync, realpathSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const TICKET_FILE = /(?:^|\/)\.tickets\/[^/]+\.md$/i;
const STATUS_DONE = /^[ \t]*(?:\+[ \t]*)?status[ \t]*:[ \t]*(?:["']done["']|done)(?:[ \t]*(?:#.*)?)$/m;
const REASON = 'Refusing direct edit setting status: done in .tickets/*.md. Use `tkt close <id> --check-all --evidence "..." --resolution "..."` instead.';

function extractPaths(input) {
  if (!input || typeof input !== "object") return [];
  const raw = [input.file_path, input.path, input.filePath, input.target_file, input.filename];
  return raw.filter((v) => typeof v === "string" && v.length > 0).map((p) => p.replaceAll("\\", "/"));
}

function extractProposedTexts(input) {
  if (!input || typeof input !== "object") return [];
  const texts = [];
  for (const key of ["content", "new_string", "new_str", "newText", "replacement", "patch", "diff"]) {
    if (typeof input[key] === "string") texts.push(input[key]);
  }
  if (Array.isArray(input.edits)) {
    for (const edit of input.edits) {
      if (!edit || typeof edit !== "object") continue;
      for (const key of ["content", "new_string", "new_str", "newText", "replacement"]) {
        if (typeof edit[key] === "string") texts.push(edit[key]);
      }
    }
  }
  return texts;
}

export function checkToolUse(payload) {
  const input = payload?.tool_input ?? payload?.toolInput ?? payload?.arguments;
  const paths = extractPaths(input);
  if (!paths.some((p) => TICKET_FILE.test(p))) return null;
  const texts = extractProposedTexts(input);
  if (!texts.some((t) => STATUS_DONE.test(t))) return null;
  return REASON;
}

export function formatDenial(reason, env = process.env) {
  const isZCode = Boolean(env.ZCODE_SESSION_ID || env.ZCODE_PLUGIN_ROOT);
  if (isZCode) {
    return JSON.stringify({
      hookEventName: "PreToolUse",
      permissionDecision: "deny",
      permissionDecisionReason: reason,
    });
  }
  return JSON.stringify({
    hookSpecificOutput: {
      hookEventName: "PreToolUse",
      permissionDecision: "deny",
      permissionDecisionReason: reason,
    },
  });
}

function main() {
  let raw = "";
  try { raw = readFileSync(0, "utf8"); } catch { return; }
  if (!raw.trim()) return;
  let payload;
  try { payload = JSON.parse(raw); } catch { return; }
  const reason = checkToolUse(payload);
  if (!reason) return;
  process.stdout.write(formatDenial(reason) + "\n");
}

if (process.argv[1] && realpathSync(fileURLToPath(import.meta.url)) === realpathSync(resolve(process.argv[1]))) {
  main();
}

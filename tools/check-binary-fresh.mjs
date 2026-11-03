#!/usr/bin/env node
import { execFileSync } from "node:child_process";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

function git(...args) {
  const root = resolve(fileURLToPath(new URL("..", import.meta.url)));
  return execFileSync("git", ["-C", root, ...args], { encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] }).trim();
}

function report(status, details = {}) {
  process.stdout.write(`${JSON.stringify({ status, ...details })}\n`);
}

try {
  const head = git("rev-parse", "--short=7", "HEAD");
  const version = (process.platform === "win32"
    ? execFileSync("tkt --version", { encoding: "utf8", stdio: ["ignore", "pipe", "pipe"], shell: true })
    : execFileSync("tkt", ["--version"], { encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] })).trim();
  const embedded = /\(([0-9a-f]{7,40})(?:-dirty)?\)/i.exec(version)?.[1];
  if (!embedded) {
    report("error", { error: "installed tkt --version does not contain an embedded git hash", head, version });
    process.exitCode = 2;
  } else if (embedded.toLowerCase() === head.toLowerCase()) {
    report("fresh", { head, installed: embedded, version });
    process.exitCode = 0;
  } else {
    report("stale", { head, installed: embedded, version });
    process.exitCode = 1;
  }
} catch (error) {
  report("error", { error: error.message });
  process.exitCode = 2;
}

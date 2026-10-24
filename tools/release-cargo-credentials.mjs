#!/usr/bin/env node
// Authorize Cargo's freshly packaged checksum. Usage: node release-cargo-credentials.mjs --cargo-plugin
// RELEASE_CARGO_POLICY names a private immutable seal-policy JSON; RELEASE_CARGO_TOKEN supplies the token.
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const refuse = (message) => ({ Err: { kind: "other", message } });
function registryPolicy(index) {
  if (typeof index !== "string") return false;
  try {
    const url = new URL(index.replace(/^sparse\+/, ""));
    return !url.username && !url.password && !url.search && !url.hash &&
      (url.protocol === "https:" || (url.protocol === "http:" && url.hostname === "127.0.0.1"));
  } catch { return false; }
}

export function credentialReply(request, policy, token) {
  if (typeof policy?.name !== "string" || !/^[a-zA-Z][a-zA-Z0-9_-]*$/.test(policy.name) ||
      typeof policy.version !== "string" || !/^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/.test(policy.version) ||
      typeof policy.sha256 !== "string" || !/^[a-f0-9]{64}$/.test(policy.sha256) || !registryPolicy(policy.index)) return refuse("invalid sealed crate policy");
  if (typeof token !== "string" || !token || /\s/.test(token)) return refuse("release credential is unavailable");
  if (request?.v !== 1 || request.kind !== "get" || !["read", "publish"].includes(request.operation) || (request.args !== undefined && (!Array.isArray(request.args) || request.args.length))) return refuse("unsupported credential request or override");
  if (request.registry?.["index-url"] !== policy.index) return refuse("registry differs from the sealed policy");
  if (request.operation === "publish" && (request.name !== policy.name || request.vers !== policy.version || request.cksum !== policy.sha256)) return refuse("packaged crate identity or checksum differs from the seal");
  // A cached read token must never bypass the later publish-checksum request.
  return { Ok: { kind: "get", token, cache: "never", operation_independent: false } };
}

async function requestLine() {
  let input = "";
  process.stdin.setEncoding("utf8");
  for await (const chunk of process.stdin) {
    input += chunk;
    if (Buffer.byteLength(input) > 65536) throw new Error("request limit");
    const newline = input.indexOf("\n");
    if (newline !== -1) return JSON.parse(input.slice(0, newline));
  }
  throw new Error("missing request line");
}

async function main() {
  if (process.argv.length !== 3 || process.argv[2] !== "--cargo-plugin") {
    console.error("usage: node release-cargo-credentials.mjs --cargo-plugin");
    process.exitCode = 2;
    return;
  }
  console.log('{"v":[1]}');
  let response;
  try {
    const request = await requestLine();
    const policy = JSON.parse(readFileSync(process.env.RELEASE_CARGO_POLICY, "utf8"));
    response = credentialReply(request, policy, process.env.RELEASE_CARGO_TOKEN);
  } catch {
    // Parse and filesystem errors can contain input snippets; do not echo credentials.
    response = refuse("credential request or sealed policy could not be read");
  }
  console.log(JSON.stringify(response));
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) await main();

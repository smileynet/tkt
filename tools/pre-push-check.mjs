#!/usr/bin/env node
import { spawnSync } from 'node:child_process';
import { closeSync, constants, existsSync, fchmodSync, lstatSync, mkdirSync, openSync, readSync, realpathSync, linkSync, unlinkSync, writeFileSync } from 'node:fs';
import { randomUUID } from 'node:crypto';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

export const SHIM = '#!/bin/sh\n# tkt optional full pre-push verification v1\nexec node "$(git rev-parse --show-toplevel)/tools/pre-push-check.mjs" run "$@"\n';
const TOOL_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const INPUT_BYTES = 256 * 1024;
const GATE_MS = 5 * 60 * 1000;

function git(root, args, env = process.env) {
  const result = spawnSync('git', ['--no-optional-locks', '--no-replace-objects', '--no-lazy-fetch', '-C', root, ...args], { env, encoding: 'utf8', maxBuffer: 1024 * 1024, windowsHide: true });
  if (result.error || result.status !== 0) throw new Error(`Git inspection failed: ${result.error?.message ?? result.stderr.trim()}`);
  return result.stdout.trim();
}
function clean(root, env) {
  const entries = git(root, ['ls-files', '-v'], env).split('\n');
  if (entries.some(entry => entry[0] === 'S' || (entry && entry[0] !== entry[0].toUpperCase()))) throw new Error('Index skip-worktree/assume-unchanged flags can hide source changes; clear/review them before this opt-in check');
  if (git(root, ['-c', 'core.fsmonitor=false', 'status', '--porcelain=v1', '--untracked-files=all'], env)) throw new Error('Working tree/index has changes or non-ignored untracked files; preserve/commit work before retrying. No automatic stash/reset is performed.');
}
export function parseUpdates(input) {
  if (Buffer.byteLength(input) > INPUT_BYTES) throw new Error('pre-push input exceeds 256 KiB');
  const lines = input.trim() ? input.trim().split(/\r?\n/) : [];
  if (lines.length > 128) throw new Error('pre-push input exceeds 128 ref updates; split this push');
  return lines.map(line => {
    const fields = line.split(/\s+/);
    if (fields.length !== 4 || !/^(?:[a-f\d]{40}|[a-f\d]{64})$/i.test(fields[1]) || !/^(?:[a-f\d]{40}|[a-f\d]{64})$/i.test(fields[3])) throw new Error('Malformed Git pre-push ref input');
    return { local: fields[1], deleted: /^0+$/.test(fields[1]) };
  });
}
export function verifyPush(root, input, { env = process.env, timeoutMs = GATE_MS } = {}) {
  const updates = parseUpdates(input).filter(update => !update.deleted);
  if (!updates.length) return { outcome: 'skip', reason: 'No new commit refs to verify (empty or deletion-only push)' };
  const head = git(root, ['rev-parse', '--verify', 'HEAD'], env);
  for (const update of updates) {
    if (git(root, ['rev-parse', '--verify', `${update.local}^{commit}`], env) !== head) throw new Error('Only refs resolving to current HEAD are supported; switch to and verify the intended tip, or split the push');
  }
  clean(root, env);
  console.error(`[pre-push] Running the full mise check for ${head.slice(0, 12)}; timeout is best-effort for the direct mise process only.`);
  const gateEnv = { ...env };
  for (const name of git(root, ['rev-parse', '--local-env-vars'], env).split('\n')) delete gateEnv[name];
  const result = spawnSync('mise', ['run', 'check'], { cwd: root, env: gateEnv, stdio: ['ignore', 'inherit', 'inherit'], timeout: timeoutMs, windowsHide: true });
  if (result.error?.code === 'ETIMEDOUT') throw new Error('TIMEOUT / unverified: direct mise deadline expired. Descendants may still be running; inspect and stop them before retrying. Nothing is certified as passing.');
  if (result.error) throw new Error(`INFRASTRUCTURE ERROR / unverified: cannot execute mise: ${result.error.message}`);
  if (result.status !== 0) throw new Error(`Full check failed or did not finish (status ${result.status}, signal ${result.signal ?? 'none'}); inspect its output. No passing verification is recorded.`);
  if (git(root, ['rev-parse', '--verify', 'HEAD'], env) !== head) throw new Error('HEAD changed during verification; check the intended tip again');
  clean(root, env);
  return { outcome: 'pass', head, reason: 'Full check command succeeded; HEAD and checkout bindings remained unchanged. Read streamed output for skips and limits.' };
}
function noLink(path, directory) {
  let info;
  try { info = lstatSync(path); } catch (error) { if (error.code === 'ENOENT') return null; throw error; }
  if (info.isSymbolicLink() || (directory ? !info.isDirectory() : !info.isFile())) throw new Error(`Preserve linked/non-regular hook path: ${path}`);
  return info;
}
function ownedHook(path) {
  const info = noLink(path, false);
  if (!info) return null;
  const expected = Buffer.from(SHIM), bytes = Buffer.alloc(expected.length + 1);
  const fd = openSync(path, constants.O_RDONLY | (constants.O_NOFOLLOW ?? 0));
  try {
    const count = readSync(fd, bytes, 0, bytes.length, 0);
    if (count !== expected.length || !bytes.subarray(0, count).equals(expected)) throw new Error(`Preserve foreign or modified pre-push hook: ${path}`);
  } finally { closeSync(fd); }
  return info;
}
function readInput() {
  const chunks = [], buffer = Buffer.alloc(4096); let total = 0;
  for (;;) {
    const count = readSync(0, buffer, 0, buffer.length, null);
    if (!count) break;
    total += count;
    if (total > INPUT_BYTES) throw new Error('pre-push input exceeds 256 KiB');
    chunks.push(Buffer.from(buffer.subarray(0, count)));
  }
  return new TextDecoder('utf-8', { fatal: true }).decode(Buffer.concat(chunks));
}
export function main(args = process.argv.slice(2)) {
  const [action, ...rest] = args;
  if (!['install', 'check', 'remove', 'run'].includes(action)) throw new Error('Usage: node tools/pre-push-check.mjs install --full-check | check | remove (run is Git hook transport only)');
  if (action !== 'run' && rest.some(arg => arg !== '--full-check')) throw new Error('Unknown pre-push checker argument');
  const root = git(process.cwd(), ['rev-parse', '--show-toplevel']);
  if (realpathSync(root) !== realpathSync(TOOL_ROOT)) throw new Error('Run this repository-specific checker from its own checkout');
  if (action === 'run') {
    const result = verifyPush(root, readInput());
    console.error(`[pre-push] ${result.outcome.toUpperCase()}: ${result.reason}`);
    return 0;
  }
  const configured = spawnSync('git', ['-C', root, 'config', '--get', 'core.hooksPath'], { encoding: 'utf8', windowsHide: true });
  if (configured.error || ![0, 1].includes(configured.status)) throw new Error('Cannot inspect existing Git hook configuration');
  if (configured.status === 0) throw new Error('core.hooksPath is configured; preserve that hook system and integrate manually');
  const common = git(root, ['rev-parse', '--path-format=absolute', '--git-common-dir']);
  const hooks = join(common, 'hooks'), path = join(hooks, 'pre-push');
  noLink(hooks, true);
  const existing = ownedHook(path);
  if (action === 'check') {
    const executable = existing && (process.platform === 'win32' || (existing.mode & 0o111) !== 0);
    console.log(executable ? 'Owned full pre-push check is installed (native execution not implied).' : 'No executable owned full pre-push check is installed.');
    return executable ? 0 : 1;
  }
  if (action === 'remove') {
    if (existing) unlinkSync(path);
    console.log(existing ? 'Removed owned pre-push hook; other hooks/configuration preserved.' : 'No owned pre-push hook to remove.');
    return 0;
  }
  if (!rest.includes('--full-check')) throw new Error('Installation is opt-in: pass --full-check to acknowledge the full check can take minutes and blocks on failed/unverified execution.');
  if (existing) {
    const fd = openSync(path, constants.O_RDONLY | (constants.O_NOFOLLOW ?? 0));
    try { fchmodSync(fd, 0o755); } finally { closeSync(fd); }
  } else {
    mkdirSync(hooks, { recursive: true });
    const temporary = join(hooks, `.tkt-pre-push-${process.pid}-${randomUUID()}`);
    try { writeFileSync(temporary, SHIM, { flag: 'wx', mode: 0o755 }); linkSync(temporary, path); }
    finally { if (existsSync(temporary)) unlinkSync(temporary); }
  }
  console.log('Installed explicit full pre-push check; no core.hooksPath/global settings changed. Linked worktrees share it.');
  return 0;
}
if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  try { process.exitCode = main(); }
  catch (error) { console.error(`[pre-push] ${error.message}`); process.exitCode = 1; }
}

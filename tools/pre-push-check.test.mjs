import test, { after } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, existsSync, copyFileSync, lstatSync, symlinkSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve, relative, isAbsolute, delimiter, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
import { SHIM, parseUpdates, verifyPush } from './pre-push-check.mjs';
import { removeTree } from './test-fixtures.mjs';


const suite = mkdtempSync(join(tmpdir(), 'tkt-pre-push-tests-'));
const source = fileURLToPath(new URL('./pre-push-check.mjs', import.meta.url));
const bin = join(suite, 'bin'); mkdirSync(bin);
const mise = join(bin, process.platform === 'win32' ? 'mise.exe' : 'mise');
const base = { ...process.env, DO_NOT_TRACK: '1', GIT_CONFIG_GLOBAL: process.platform === 'win32' ? 'NUL' : '/dev/null', GIT_CONFIG_NOSYSTEM: '1', GIT_CONFIG_COUNT: '0' };
for (const key of ['GIT_DIR', 'GIT_WORK_TREE', 'GIT_INDEX_FILE', 'GIT_COMMON_DIR', 'GIT_PREFIX', 'GIT_CONFIG_PARAMETERS']) delete base[key];
for (const key of Object.keys(base)) {
  if (key.toUpperCase() === 'PATH') { const value = base[key]; delete base[key]; base.PATH = value; }
}
const compile = spawnSync('rustc', [fileURLToPath(new URL('../tests/support/mise_gate_probe.rs', import.meta.url)), '-o', mise], { env: base, encoding: 'utf8' });
assert.equal(compile.status, 0, compile.stderr);
after(() => {
  const inside = relative(resolve(tmpdir()), resolve(suite));
  assert.ok(inside && !inside.startsWith('..') && !isAbsolute(inside));
  assert.ok(suite.includes('tkt-pre-push-tests-'));
  removeTree(suite);
});
function git(root, args, env = base, expected = 0) {
  const result = spawnSync('git', ['-C', root, ...args], { env, encoding: 'utf8' });
  assert.equal(result.status, expected, `${args}: ${result.stdout}\n${result.stderr}`);
  return result.stdout.trim();
}
function fixture(remote = false) {
  const area = mkdtempSync(join(suite, 'case-')), root = join(area, 'repo Ω spaces'); mkdirSync(root);
  git(root, ['init', '-q', '-b', 'main']);
  for (const [key, value] of [['user.name', 'Gate fixture'], ['user.email', 'fixture@example.invalid'], ['maintenance.auto', 'false'], ['gc.auto', '0'], ['core.autocrlf', 'false']]) git(root, ['config', key, value]);
  mkdirSync(join(root, 'tools')); copyFileSync(source, join(root, 'tools/pre-push-check.mjs'));
  writeFileSync(join(root, 'README.md'), 'Fixture source\n');
  writeFileSync(join(root, 'mise.toml'), '[tasks.check]\nrun="echo fixture"\n');
  git(root, ['add', '.']); git(root, ['commit', '-qm', 'fixture']);
  const log = join(root, '.git/gate-invocations');
  const env = { ...base, PATH: [bin, dirname(process.execPath), base.PATH].join(delimiter), TKT_GATE_FIXTURE_LOG: log, TKT_GATE_FIXTURE_MODE: 'ok' };
  const script = join(root, 'tools/pre-push-check.mjs');
  function cli(args, expected = 0) {
    const out = spawnSync(process.execPath, [script, ...args], { cwd: root, env, encoding: 'utf8' });
    assert.equal(out.status, expected, `${args}: ${out.stdout}\n${out.stderr}`); return out;
  }
  let remotePath;
  if (remote) {
    remotePath = join(area, 'remote.git'); git(root, ['init', '--bare', '-q', remotePath]);
    git(root, ['remote', 'add', 'origin', remotePath]); git(root, ['push', '-qu', 'origin', 'main']);
  }
  return { root, area, log, env, cli, script, remote: remotePath };
}
const input = head => `refs/heads/main ${head} refs/heads/main ${'0'.repeat(head.length)}\n`;
const calls = f => existsSync(f.log) ? readFileSync(f.log, 'utf8').trim().split('\n') : [];

test('installation is explicit, owned, idempotent and shared with linked worktrees', () => {
  const f = fixture();
  f.cli(['install'], 1); assert.equal(existsSync(join(f.root, '.git/hooks/pre-push')), false);
  f.cli(['check'], 1); f.cli(['install', '--full-check']); f.cli(['check']);
  assert.equal(readFileSync(join(f.root, '.git/hooks/pre-push'), 'utf8'), SHIM);
  f.cli(['install', '--full-check']);
  const linked = join(f.area, 'linked worktree'); git(f.root, ['worktree', 'add', '-qb', 'linked', linked]);
  const out = spawnSync(process.execPath, [join(linked, 'tools/pre-push-check.mjs'), 'check'], { cwd: linked, env: f.env, encoding: 'utf8' });
  assert.equal(out.status, 0, out.stderr);
  f.cli(['remove']); f.cli(['remove']); assert.equal(existsSync(join(f.root, '.git/hooks/pre-push')), false);
  assert.equal(git(f.root, ['config', '--get', 'core.hooksPath'], base, 1), '');
});

test('foreign, modified and linked hooks plus custom hooksPath are preserved', () => {
  for (const mode of ['foreign', 'modified', 'link', 'directory', 'configured']) {
    const f = fixture(); const path = join(f.root, '.git/hooks/pre-push');
    if (mode === 'configured') git(f.root, ['config', 'core.hooksPath', '.foreign-hooks']);
    else if (mode === 'link') { const other = join(f.area, 'foreign-hook'); writeFileSync(other, 'foreign'); symlinkSync(other, path, 'file'); }
    else if (mode === 'directory') mkdirSync(path);
    else writeFileSync(path, mode === 'modified' ? SHIM + '# user change\n' : 'foreign');
    for (const action of [['install', '--full-check'], ['check'], ['remove']]) f.cli(action, 1);
    if (mode === 'configured') assert.equal(git(f.root, ['config', '--get', 'core.hooksPath']), '.foreign-hooks');
    else if (mode === 'directory') assert.ok(lstatSync(path).isDirectory());
    else assert.equal(readFileSync(path, 'utf8'), mode === 'modified' ? SHIM + '# user change\n' : 'foreign');
  }
});

test('actual Git push invokes the gate, binds HEAD, strips local Git environment and blocks failure', () => {
  const f = fixture(true); f.cli(['install', '--full-check']);
  writeFileSync(join(f.root, 'README.md'), 'First checked change\n'); git(f.root, ['add', '.']); git(f.root, ['commit', '-qm', 'first']);
  const first = git(f.root, ['rev-parse', 'HEAD']); git(f.root, ['push', '-q', 'origin', 'main'], f.env);
  assert.deepEqual(calls(f), [first]); assert.equal(git(f.remote, ['rev-parse', 'refs/heads/main']), first);
  writeFileSync(join(f.root, 'README.md'), 'Second checked change\n'); git(f.root, ['add', '.']); git(f.root, ['commit', '-qm', 'second']);
  const second = git(f.root, ['rev-parse', 'HEAD']);
  const failed = spawnSync('git', ['-C', f.root, 'push', '-q', 'origin', 'main'], { env: { ...f.env, TKT_GATE_FIXTURE_MODE: 'fail' }, encoding: 'utf8' });
  assert.notEqual(failed.status, 0); assert.match(failed.stderr, /Full check failed/);
  assert.deepEqual(calls(f), [first, second]); assert.equal(git(f.remote, ['rev-parse', 'refs/heads/main']), first);
  git(f.root, ['push', '-q', 'origin', 'main'], f.env); assert.equal(git(f.remote, ['rev-parse', 'refs/heads/main']), second);
});

test('unsupported revisions and dirty or masked source cannot pass a different checkout', () => {
  const f = fixture(); const head = git(f.root, ['rev-parse', 'HEAD']);
  writeFileSync(join(f.root, 'README.md'), 'Changed\n'); assert.throws(() => verifyPush(f.root, input(head), { env: f.env }), /Working tree/); assert.deepEqual(calls(f), []);
  git(f.root, ['add', '.']); git(f.root, ['commit', '-qm', 'next']);
  assert.throws(() => verifyPush(f.root, input(head), { env: f.env }), /current HEAD/); assert.deepEqual(calls(f), []);
  const current = git(f.root, ['rev-parse', 'HEAD']);
  git(f.root, ['update-index', '--assume-unchanged', 'README.md']); assert.throws(() => verifyPush(f.root, input(current), { env: f.env }), /assume-unchanged/); git(f.root, ['update-index', '--no-assume-unchanged', 'README.md']);
  writeFileSync(join(f.root, 'untracked.txt'), 'Untracked source\n'); assert.throws(() => verifyPush(f.root, input(current), { env: f.env }), /untracked/); assert.deepEqual(calls(f), []);
});

test('post-gate HEAD and worktree changes are refused without rolling them back', () => {
  for (const mode of ['head', 'dirty']) {
    const f = fixture(); const head = git(f.root, ['rev-parse', 'HEAD']);
    assert.throws(() => verifyPush(f.root, input(head), { env: { ...f.env, TKT_GATE_FIXTURE_MODE: mode } }), mode === 'head' ? /HEAD changed/ : /Working tree/);
    if (mode === 'head') assert.notEqual(git(f.root, ['rev-parse', 'HEAD']), head);
    else assert.equal(readFileSync(join(f.root, 'README.md'), 'utf8'), 'Changed during gate\n');
  }
});

test('deletions skip, same-HEAD annotated tags share one run, malformed input is refused', () => {
  const f = fixture(); const head = git(f.root, ['rev-parse', 'HEAD']);
  assert.equal(verifyPush(f.root, '', { env: f.env }).outcome, 'skip');
  assert.equal(verifyPush(f.root, `(delete) ${'0'.repeat(40)} refs/heads/old ${head}\n`, { env: f.env }).outcome, 'skip'); assert.deepEqual(calls(f), []);
  git(f.root, ['tag', '-a', 'test-tag', '-m', 'fixture tag']); const tag = git(f.root, ['rev-parse', 'test-tag']);
  assert.equal(verifyPush(f.root, input(head) + `refs/tags/test-tag ${tag} refs/tags/test-tag ${'0'.repeat(40)}\n`, { env: f.env }).outcome, 'pass'); assert.equal(calls(f).length, 1);
  for (const text of ['bad input', input(head).repeat(129), 'x'.repeat(256 * 1024 + 1)]) assert.throws(() => parseUpdates(text));
});

test('timeout and missing runtime are explicitly unverified failures', () => {
  const f = fixture(); const head = git(f.root, ['rev-parse', 'HEAD']);
  assert.throws(() => verifyPush(f.root, input(head), { env: { ...f.env, TKT_GATE_FIXTURE_MODE: 'sleep' }, timeoutMs: 100 }), /TIMEOUT \/ unverified/);
  const gitName = process.platform === 'win32' ? 'git.exe' : 'git';
  const gitDir = base.PATH.split(delimiter).find(directory => existsSync(join(directory, gitName)));
  assert.ok(gitDir); assert.equal(existsSync(join(gitDir, process.platform === 'win32' ? 'mise.exe' : 'mise')), false);
  assert.throws(() => verifyPush(f.root, input(head), { env: { ...f.env, PATH: gitDir } }), /INFRASTRUCTURE ERROR \/ unverified/);
});

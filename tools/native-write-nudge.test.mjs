import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync, spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const script = fileURLToPath(new URL('../assets/native-write-nudge.mjs', import.meta.url));
const env = { ...process.env, DO_NOT_TRACK: '1' };
delete env.NODE_OPTIONS;
function run(profile, input) {
  const result = spawnSync(process.execPath, [script, profile], { input, env, timeout: 4000 });
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stderr.toString());
  assert.equal(result.stderr.toString(), '');
  assert.ok(result.stdout.length < 1024);
  const value = JSON.parse(result.stdout);
  assert.equal(Object.hasOwn(value, 'permissionDecision'), false);
  assert.equal(Object.hasOwn(value, 'decision'), false);
  return value;
}
const event = (tool, input, name = 'PreToolUse') => JSON.stringify({ hook_event_name: name, tool_name: tool, tool_input: input });
const hinted = value => typeof value.hookSpecificOutput?.additionalContext === 'string';

test('Claude advisory path detection normalizes separators and traversal without exposing input', () => {
  for (const path of ['.tickets/01-work.md', '/work/.tickets/cr-axcq.md', 'C:\\work\\.tickets\\03-work.md', './.tickets/../.tickets/04.md']) {
    for (const tool of ['Edit', 'Write']) {
      const value = run('claude', event(tool, { file_path: path, old_string: 'private-secret' }));
      assert.ok(hinted(value), path);
      assert.equal(value.hookSpecificOutput.hookEventName, 'PreToolUse');
      assert.equal(Object.hasOwn(value.hookSpecificOutput, 'permissionDecision'), false);
      assert.equal(JSON.stringify(value).includes('private-secret'), false);
      assert.match(value.hookSpecificOutput.additionalContext, /Body\/spec prose edits remain allowed/);
    }
  }
  for (const path of ['.tickets/sub/01.md', '.tickets2/01.md', '.tickets/../notes.md', '.tickets/.md', '.tickets/01.txt', '.tickets/01.md\nother', '.tickets/' + 'x'.repeat(4096) + '.md']) {
    assert.deepEqual(run('claude', event('Write', { file_path: path })), {}, path);
  }
  assert.equal(hinted(run('claude', event('Write', { file_path: '.TICKETS/01.MD' }))), process.platform === 'win32');
});

test('Codex recognizes only bounded canonical apply_patch headers including moves', () => {
  for (const header of ['Add File', 'Update File', 'Delete File', 'Move to']) {
    const patch = `*** Begin Patch\n*** ${header}: .tickets/01-work.md\n*** End Patch\n`;
    assert.ok(hinted(run('codex', event('apply_patch', { command: patch }))), header);
  }
  const move = '*** Begin Patch\n*** Update File: docs/notes.md\n*** Move to: .tickets/01-work.md\n*** End Patch';
  assert.ok(hinted(run('codex', event('apply_patch', { command: move }))));
  const fake = '*** Begin Patch\n*** Update File: docs/notes.md\n+*** Update File: .tickets/01-work.md\n *** Delete File: .tickets/02.md\n*** End Patch';
  assert.deepEqual(run('codex', event('apply_patch', { command: fake })), {});
  assert.deepEqual(run('codex', event('Bash', { command: 'echo .tickets/01-work.md' })), {});
  assert.deepEqual(run('codex', event('apply_patch', { command: 'echo "*** Begin Patch"' })), {});
  const many = '*** Begin Patch\n' + '*** Update File: .tickets/01.md\n'.repeat(65) + '*** End Patch';
  assert.deepEqual(run('codex', event('apply_patch', { command: many })), {});
  const lines = '*** Begin Patch\n*** Update File: .tickets/01.md\n' + '+x\n'.repeat(8192) + '*** End Patch';
  assert.deepEqual(run('codex', event('apply_patch', { command: lines })), {});
});

test('unknown malformed oversized and non-event payloads remain nonblocking empty JSON', () => {
  for (const input of ['', '{', 'null', '[]', Buffer.from([0xff]), JSON.stringify({ content: 'x'.repeat(256 * 1024) })]) {
    assert.deepEqual(run('claude', input), {});
  }
  assert.deepEqual(run('claude', event('Edit', { file_path: '.tickets/01.md' }, 'PostToolUse')), {});
  assert.deepEqual(run('claude', event('Bash', { file_path: '.tickets/01.md' })), {});
  assert.deepEqual(run('claude', event('Edit', { file_path: 1 })), {});
  assert.deepEqual(run('unknown', event('Edit', { file_path: '.tickets/01.md' })), {});
});

test('incomplete stdin reaches the bounded empty-output deadline', async () => {
  const start = Date.now();
  const child = spawn(process.execPath, [script, 'claude'], { env, stdio: ['pipe', 'pipe', 'pipe'] });
  child.stdin.on('error', () => {});
  child.stdin.write('{');
  let output = '', errors = '';
  child.stdout.on('data', chunk => { output += chunk; });
  child.stderr.on('data', chunk => { errors += chunk; });
  const status = await new Promise((resolve, reject) => { child.on('error', reject); child.on('close', resolve); });
  assert.equal(status, 0);
  assert.equal(output, '{}\n');
  assert.equal(errors, '');
  assert.ok(Date.now() - start < 4000);
});

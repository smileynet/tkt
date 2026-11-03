import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, mkdir, writeFile, rm, symlink } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { once } from 'node:events';
import { createDashboardServer } from './serve-dashboard.mjs';

test('serve current dashboard and explicit links without exposing repository files', async t => {
  const root = await mkdtemp(join(tmpdir(), 'dashboard-lan-'));
  await mkdir(join(root, 'docs/evidence'), { recursive: true });
  await mkdir(join(root, '.git'));
  const html = '<h1>Decisions</h1><script type="application/json" id="gm-entries">[{"links":[{"href":"evidence/result.md"},{"href":"../.git/config"},{"href":"../.GIT/config"},{"href":"../../outside.md"}]}]</script>';
  await writeFile(join(root, 'docs/status.html'), html);
  await writeFile(join(root, 'docs/evidence/result.md'), 'Evidence');
  await writeFile(join(root, '.git/config'), 'Private');
  await mkdir(join(root, '.GIT'), { recursive: true });
  await writeFile(join(root, '.GIT/config'), 'Private');
  await writeFile(join(root, 'private.txt'), 'Private');
  const server = createDashboardServer(root);
  t.after(async () => { server.closeAllConnections(); await new Promise(resolve => server.close(resolve)); await rm(root, { recursive: true }); });
  server.listen(0, '127.0.0.1'); await once(server, 'listening');
  const base = `http://127.0.0.1:${server.address().port}`;
  const page = await fetch(base + '/');
  assert.equal(page.status, 200); assert.equal(new URL(page.url).pathname, '/docs/status.html');
  assert.equal(await page.text(), html); assert.equal(page.headers.get('cache-control'), 'no-store');
  const linked = await fetch(base + '/docs/evidence/result.md');
  assert.equal(linked.status, 200); assert.equal(await linked.text(), 'Evidence');
  assert.match(linked.headers.get('content-type'), /text\/plain/);
  for (const path of ['/private.txt', '/.git/config', '/.GIT/config', '/docs/', '/outside.md', '/docs/%2e%2e/.git/config', '/docs/evidence/result.md%2f..%2f..%2fprivate.txt']) assert.equal((await fetch(base + path)).status, 404, path);
  assert.equal((await fetch(base + '/docs/status.html', { method: 'POST' })).status, 405);
  const head = await fetch(base + '/docs/status.html', { method: 'HEAD' });
  assert.equal(head.status, 200); assert.equal(await head.text(), '');
  await writeFile(join(root, 'docs/status.html'), '<h1>Updated</h1>');
  assert.equal(await (await fetch(base + '/docs/status.html')).text(), '<h1>Updated</h1>');
});

test('linked symlinks cannot expose Git metadata or files outside the checkout', async t => {
  const temp = await mkdtemp(join(tmpdir(), 'dashboard-links-'));
  const root = join(temp, 'repo');
  await mkdir(join(root, 'docs'), { recursive: true });
  await mkdir(join(root, '.git'));
  await writeFile(join(root, '.git/config'), 'Private Git data');
  await writeFile(join(temp, 'outside.md'), 'Outside');
  t.after(() => rm(temp, { recursive: true }));
  try {
    await symlink(join(root, '.git/config'), join(root, 'docs/git.md'), 'file');
    await symlink(join(temp, 'outside.md'), join(root, 'docs/outside.md'), 'file');
  } catch (error) { if (error.code === 'EPERM') { t.skip('OS does not permit test symlinks'); return; } throw error; }
  await writeFile(join(root, 'docs/status.html'), '<script type="application/json" id="gm-entries">[{"links":[{"href":"git.md"},{"href":"outside.md"}]}]</script>');
  const server = createDashboardServer(root);
  t.after(async () => { server.closeAllConnections(); await new Promise(resolve => server.close(resolve)); });
  server.listen(0, '127.0.0.1'); await once(server, 'listening');
  const base = `http://127.0.0.1:${server.address().port}`;
  assert.equal((await fetch(base + '/docs/git.md')).status, 404);
  assert.equal((await fetch(base + '/docs/outside.md')).status, 404);
});

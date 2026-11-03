import { createServer } from 'node:http';
import { readFile, realpath } from 'node:fs/promises';
import { dirname, extname, isAbsolute, relative, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { parseArgs } from 'node:util';

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const types = { '.html': 'text/html', '.svg': 'image/svg+xml', '.png': 'image/png', '.jpg': 'image/jpeg', '.jpeg': 'image/jpeg', '.webp': 'image/webp' };
const allowed = (root, file) => {
  const path = relative(root, file);
  return !isAbsolute(path) && path !== '..' && !path.startsWith('../') && !path.startsWith('..\\') && !path.split(/[\\/]/).some(part => ['.git', '.scratch', '.references'].includes(part));
};

function linkedFiles(html, root) {
  const dashboard = resolve(root, 'docs/status.html');
  const files = new Map([['/docs/status.html', dashboard]]);
  function visit(value) {
    if (!value || typeof value !== 'object') return;
    for (const [key, child] of Object.entries(value)) {
      if (['href', 'src'].includes(key) && typeof child === 'string' && child && !child.startsWith('#') && !child.startsWith('/') && !/^[a-z][a-z0-9+.-]*:/i.test(child)) {
        const url = new URL(child, 'http://dashboard.invalid/docs/status.html');
        const file = fileURLToPath(new URL(child, pathToFileURL(dashboard)));
        if (allowed(root, file)) files.set(url.pathname, file);
      }
      visit(child);
    }
  }
  for (const match of html.matchAll(/<script type="application\/json" id="gm-(?:config|entries|tickets)">([\s\S]*?)<\/script>/g)) visit(JSON.parse(match[1]));
  return files;
}

export function createDashboardServer(root = projectRoot) {
  root = resolve(root);
  return createServer(async (req, res) => {
    res.setHeader('Cache-Control', 'no-store');
    res.setHeader('X-Content-Type-Options', 'nosniff');
    if (!['GET', 'HEAD'].includes(req.method)) { res.writeHead(405, { Allow: 'GET, HEAD' }); res.end(); return; }
    let pathname;
    try { pathname = new URL(req.url, 'http://dashboard.invalid').pathname; }
    catch { res.writeHead(400); res.end('Invalid URL'); return; }
    if (pathname === '/' || pathname === '/status.html') { res.writeHead(302, { Location: '/docs/status.html' }); res.end(); return; }
    try {
      const html = await readFile(resolve(root, 'docs/status.html'), 'utf8');
      const file = linkedFiles(html, root).get(pathname);
      if (!file || !allowed(await realpath(root), await realpath(file))) { res.writeHead(404); res.end('Not found'); return; }
      const body = await readFile(file);
      const type = types[extname(file).toLowerCase()] || 'text/plain';
      res.writeHead(200, { 'Content-Type': type + (type.startsWith('text/') ? '; charset=utf-8' : ''), 'Content-Length': body.length });
      res.end(req.method === 'HEAD' ? undefined : body);
    } catch (error) {
      const missing = ['ENOENT', 'ENOTDIR', 'EISDIR'].includes(error.code);
      res.writeHead(missing ? 404 : 500); res.end(missing ? 'Not found' : 'Cannot read dashboard');
    }
  });
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const { values } = parseArgs({ options: { host: { type: 'string', default: '127.0.0.1' }, port: { type: 'string', default: '8765' } } });
  const port = Number(values.port);
  if (!Number.isInteger(port) || port < 1 || port > 65535) throw new Error('--port must be an integer from 1 to 65535');
  const server = createDashboardServer();
  server.on('error', error => { console.error(error.message); process.exitCode = 1; });
  server.listen(port, values.host, () => console.log(`Dashboard: http://${values.host}:${port}/docs/status.html`));
}

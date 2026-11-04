const LIMIT = 256 * 1024;
const MAX_LINES = 8192;
const MAX_PATHS = 64;
const profile = process.argv[2];
const context = 'This call targets a top-level Markdown path under .tickets/. Prefer tkt edit/claim/close for frontmatter and lifecycle changes so atomic writes and gates apply. Body/spec prose edits remain allowed. Use tkt note ID --append for an attributable note and tkt capabilities -o json for exact commands. Run tkt from the appropriate repository; this advisory does not prove file identity or authorize any mutation.';

function ticketPath(value) {
  if (typeof value !== 'string' || Buffer.byteLength(value) > 4096 || /[\0\r\n]/.test(value)) return false;
  const parts = value.replaceAll('\\', '/').split('/');
  if (parts.length > 256) return false;
  const normalized = [];
  for (const part of parts) {
    if (!part || part === '.') continue;
    if (part === '..' && normalized.length && normalized.at(-1) !== '..') normalized.pop();
    else normalized.push(part);
  }
  if (normalized.length < 2) return false;
  let directory = normalized.at(-2), name = normalized.at(-1);
  if (process.platform === 'win32') { directory = directory.toLowerCase(); name = name.toLowerCase(); }
  return directory === '.tickets' && name.length > 3 && name.endsWith('.md');
}

function patchTouchesTicket(command) {
  if (typeof command !== 'string') return false;
  const lines = command.split(/\r?\n/);
  while (lines.at(-1) === '') lines.pop();
  if (lines.length > MAX_LINES || lines[0] !== '*** Begin Patch' || lines.at(-1) !== '*** End Patch') return false;
  let paths = 0, touched = false;
  for (const line of lines.slice(1, -1)) {
    const match = /^\*\*\* (?:Add File|Update File|Delete File|Move to): (.+)$/.exec(line);
    if (!match) continue;
    if (++paths > MAX_PATHS) return false;
    touched ||= ticketPath(match[1]);
  }
  return touched;
}

function inspect(input) {
  if (!input || input.hook_event_name !== 'PreToolUse' || !input.tool_input || typeof input.tool_input !== 'object') return {};
  const touched = profile === 'claude'
    ? ['Edit', 'Write'].includes(input.tool_name) && ticketPath(input.tool_input.file_path)
    : profile === 'codex' && input.tool_name === 'apply_patch' && patchTouchesTicket(input.tool_input.command);
  return touched ? { hookSpecificOutput: { hookEventName: 'PreToolUse', additionalContext: context } } : {};
}

let bytes = 0, chunks = [], oversized = false, finished = false;
const timer = setTimeout(() => finish({}), 1500);
function finish(output) {
  if (finished) return;
  finished = true;
  clearTimeout(timer);
  process.stdin.destroy();
  process.stdout.write(`${JSON.stringify(output)}\n`, () => process.exit(0));
}
process.stdin.on('data', chunk => {
  bytes += chunk.length;
  if (bytes > LIMIT) { oversized = true; chunks = []; }
  else if (!oversized) chunks.push(chunk);
});
process.stdin.on('error', () => finish({}));
process.stdin.on('end', () => {
  if (oversized) return finish({});
  try {
    const text = new TextDecoder('utf-8', { fatal: true }).decode(Buffer.concat(chunks));
    finish(inspect(JSON.parse(text)));
  } catch { finish({}); }
});

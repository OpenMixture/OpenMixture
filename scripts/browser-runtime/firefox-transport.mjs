// Minimal WebDriver BiDi transport for the same public-package qualification.
// No browser launch, GPU preference, shader or renderer implementation lives here.
import assert from 'node:assert/strict';

export async function connectFirefox(endpoint) {
  const socket = new WebSocket(endpoint);
  await new Promise((resolve, reject) => {
    const timer = setTimeout(() => { socket.close(); reject(new Error('BiDi connection timeout')); }, 20_000);
    socket.addEventListener('open', () => { clearTimeout(timer); resolve(); }, { once: true });
    socket.addEventListener('error', error => { clearTimeout(timer); reject(error); }, { once: true });
  });
  let nextId = 0;
  const pending = new Map(), pages = new Map();
  socket.addEventListener('close', () => {
    for (const task of pending.values()) { clearTimeout(task.timer); task.reject(new Error('BiDi connection closed')); }
    pending.clear();
  });
  socket.addEventListener('message', event => {
    const message = JSON.parse(event.data);
    if (message.id) {
      const task = pending.get(message.id);
      if (!task) return;
      pending.delete(message.id); clearTimeout(task.timer);
      if (message.type === 'error') task.reject(new Error(JSON.stringify(message)));
      else task.resolve(message.result);
    } else if (message.method === 'log.entryAdded' && message.params.type === 'javascript' && message.params.level === 'error') {
      pages.get(message.params.source?.context)?.error?.(new Error(message.params.text));
    }
  });
  const send = (method, params = {}) => new Promise((resolve, reject) => {
    const id = ++nextId;
    const timer = setTimeout(() => { pending.delete(id); reject(new Error(`BiDi timeout: ${method}`)); }, 60_000);
    pending.set(id, { resolve, reject, timer });
    socket.send(JSON.stringify({ id, method, params }));
  });
  const session = await send('session.new', { capabilities: { alwaysMatch: { acceptInsecureCerts: false } } });
  await send('session.subscribe', { events: ['log.entryAdded'] });
  return {
    capabilities: session.capabilities,
    version: () => session.capabilities.browserVersion,
    contexts: () => [{ newPage: async () => {
      const { context } = await send('browsingContext.create', { type: 'tab' });
      let closed = false;
      const page = {
        on(event, callback) { assert.equal(event, 'pageerror'); page.error = callback; },
        async addInitScript(fn, argument) {
          await send('script.addPreloadScript', { contexts: [context],
            functionDeclaration: `() => { (${fn.toString()})(${JSON.stringify(argument) ?? 'undefined'}); }` });
        },
        async goto(url) { await send('browsingContext.navigate', { context, url, wait: 'complete' }); },
        async evaluate(fn, argument) {
          const result = await send('script.evaluate', { target: { context }, awaitPromise: true,
            expression: `(async () => JSON.stringify(await (${fn.toString()})(${JSON.stringify(argument) ?? 'undefined'})))()` });
          assert.equal(result.type, 'success', JSON.stringify(result));
          return result.result.type === 'undefined' ? undefined : JSON.parse(result.result.value);
        },
        async screenshot({ path }) {
          const { writeFile } = await import('node:fs/promises');
          const result = await send('browsingContext.captureScreenshot', { context, origin: 'document' });
          await writeFile(path, Buffer.from(result.data, 'base64'));
        },
        isClosed: () => closed,
        async close() { if (!closed) { await send('browsingContext.close', { context }); closed = true; pages.delete(context); } },
      };
      pages.set(context, page);
      return page;
    } }],
    async close() { try { await send('session.end'); } finally { socket.close(); } },
  };
}

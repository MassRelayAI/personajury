#!/usr/bin/env node
// Page-walk exporter: captures an ARIA walk (<name>.walk.json) and a screenshot (<name>.png) per page, over CDP.
// Helper run before `subject add`; never part of the engine. No npm dependency. Needs Node >= 22 (global WebSocket)
// and a local Chrome/Chromium (or --cdp ws://...). Format and id rule: docs/walks.md.
import { spawn } from 'node:child_process';
import { existsSync } from 'node:fs';
import { mkdir, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const DESKTOP = { width: 1280, height: 800, mobile: false };
const PHONE = { width: 390, height: 844, mobile: true };
// AX roles that carry no meaning of their own: dropped, their children are promoted.
const DROP = new Set(['none', 'generic', 'presentation', 'StaticText', 'InlineTextBox', 'LineBreak', 'ListMarker',
  'RootWebArea', 'WebArea', 'Section', 'Ignored', 'Unknown', 'Div', 'text']);
// Roles whose AX name is empty but whose meaning is their text (the fixture names them by their text).
const TEXT_NAMED = new Set(['paragraph', 'listitem', 'cell', 'gridcell', 'term', 'definition', 'caption', 'blockquote']);

export function slug(s) {
  return String(s).normalize('NFKD').replace(/[̀-ͯ]/g, '').toLowerCase()
    .replace(/[^a-z0-9]+/g, '-').replace(/^-+|-+$/g, '').slice(0, 40).replace(/-+$/, '');
}

const squash = (s) => String(s ?? '').replace(/\s+/g, ' ').trim();
const val = (v) => (v && v.value !== undefined && v.value !== null ? v.value : '');

// Pure: Accessibility.getFullAXTree nodes (+ optional Map backendDOMNodeId -> href) -> the walk `nodes` array.
export function buildNodes(axNodes, hrefs = new Map()) {
  const byId = new Map(axNodes.map((n) => [n.nodeId, n]));
  const textOf = (n) => {
    const role = val(n.role);
    if (n.ignored && role !== 'StaticText') { /* ignored wrappers can still hold text */ }
    if (role === 'StaticText') return val(n.name);
    if (role === 'ListMarker') return '';
    return (n.childIds || []).map((c) => (byId.get(c) ? textOf(byId.get(c)) : '')).join('');
  };
  const seen = new Map();
  const uniqueId = (base) => {
    let n = (seen.get(base) || 0) + 1;
    let id = n === 1 ? base : `${base}-${n}`;
    while (seen.has(id) && id !== base) { n += 1; id = `${base}-${n}`; }
    seen.set(base, n);
    if (id !== base) seen.set(id, 1);
    return id;
  };
  const walk = (n) => {
    const kids = (n.childIds || []).flatMap((c) => (byId.get(c) ? walk(byId.get(c)) : []));
    const role = val(n.role);
    if (n.ignored || DROP.has(role) || !role) return kids;
    let name = squash(val(n.name));
    if (!name && TEXT_NAMED.has(role)) name = squash(textOf(n));
    const base = slug(role) + (slug(name) ? `-${slug(name)}` : '');
    if (role === 'link') {
      const href = hrefs.get(n.backendDOMNodeId);
      if (href !== undefined) name = name ? `${name} (${href})` : `(${href})`;
    }
    const node = { id: null, role, name, _kids: role === 'link' ? [] : kids, _base: base };
    return [node];
  };
  const roots = axNodes.length ? walk(axNodes[0]) : [];
  // ids are assigned in document order (pre-order) after the tree is built, so promotion never reorders them.
  const finish = (n) => {
    n.id = uniqueId(n._base);
    const out = { id: n.id, role: n.role, name: n.name };
    const kids = n._kids.map(finish);
    if (kids.length) out.children = kids;
    return out;
  };
  return roots.map(finish);
}

export function parseArgs(argv) {
  const o = { out: '.', phone: false, cdp: null, chrome: null, pages: [] };
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === '--out') o.out = argv[++i];
    else if (a === '--phone') o.phone = true;
    else if (a === '--cdp') o.cdp = argv[++i];
    else if (a === '--chrome') o.chrome = argv[++i];
    else if (a === '-h' || a === '--help') o.help = true;
    else if (a.startsWith('--')) throw new Error(`unknown option ${a}`);
    else {
      const eq = a.indexOf('=');
      if (eq < 1) throw new Error(`expected name=url-or-file, got ${a}`);
      const name = a.slice(0, eq);
      if (!/^[A-Za-z0-9._-]+$/.test(name)) throw new Error(`bad page name ${name}`);
      let url = a.slice(eq + 1);
      if (!/^[a-z][a-z0-9+.-]*:\/\//i.test(url)) url = pathToFileURL(resolve(url)).href;
      o.pages.push({ name, url });
    }
  }
  return o;
}

const USAGE = 'usage: node tools/walk.mjs [--out DIR] [--phone] [--cdp ws://...] [--chrome PATH] name=url|file ...';

export function findChrome(explicit) {
  if (explicit) return existsSync(explicit) ? explicit : null; // an explicit path is never silently replaced
  const c = [process.env.CHROME_BIN, '/usr/bin/google-chrome', '/usr/bin/google-chrome-stable',
    '/usr/bin/chromium', '/usr/bin/chromium-browser', '/snap/bin/chromium',
    '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',
    '/Applications/Chromium.app/Contents/MacOS/Chromium'].filter(Boolean);
  return c.find((p) => existsSync(p)) || null;
}

class Cdp {
  constructor(ws) {
    this.ws = ws; this.id = 0; this.pending = new Map(); this.listeners = [];
    ws.addEventListener('message', (e) => {
      const m = JSON.parse(e.data);
      if (m.id && this.pending.has(m.id)) {
        const { res, rej } = this.pending.get(m.id); this.pending.delete(m.id);
        m.error ? rej(new Error(`${m.error.message}`)) : res(m.result);
      } else if (m.method) this.listeners.forEach((l) => l(m));
    });
  }
  static open(url) {
    return new Promise((res, rej) => {
      const ws = new WebSocket(url);
      ws.addEventListener('open', () => res(new Cdp(ws)));
      ws.addEventListener('error', () => rej(new Error(`cannot connect to ${url}`)));
    });
  }
  send(method, params = {}, sessionId) {
    const id = ++this.id;
    return new Promise((res, rej) => {
      this.pending.set(id, { res, rej });
      this.ws.send(JSON.stringify({ id, method, params, ...(sessionId ? { sessionId } : {}) }));
    });
  }
  waitFor(method, sessionId, ms = 30000) {
    return new Promise((res, rej) => {
      const t = setTimeout(() => rej(new Error(`timeout waiting for ${method}`)), ms);
      const l = (m) => {
        if (m.method === method && (!sessionId || m.sessionId === sessionId)) {
          clearTimeout(t); this.listeners = this.listeners.filter((x) => x !== l); res(m.params);
        }
      };
      this.listeners.push(l);
    });
  }
  close() { this.ws.close(); }
}

async function launchChrome(path) {
  const dir = await mkdtemp(join(tmpdir(), 'walk-chrome-'));
  const proc = spawn(path, ['--headless=new', '--remote-debugging-port=0', `--user-data-dir=${dir}`, '--no-first-run',
    '--no-default-browser-check', '--disable-gpu', '--hide-scrollbars', '--force-device-scale-factor=1',
    ...(process.getuid && process.getuid() === 0 ? ['--no-sandbox'] : []), 'about:blank'], { stdio: 'ignore' });
  const portFile = join(dir, 'DevToolsActivePort');
  for (let i = 0; i < 200; i++) {
    if (existsSync(portFile)) {
      const [port, path2] = (await readFile(portFile, 'utf8')).trim().split('\n');
      if (port && path2) return { ws: `ws://127.0.0.1:${port}${path2}`, proc, dir };
    }
    await new Promise((r) => setTimeout(r, 50));
  }
  proc.kill(); throw new Error('Chrome did not open a debugging port');
}

async function capture(cdp, url, vp) {
  const { targetId } = await cdp.send('Target.createTarget', { url: 'about:blank' });
  const { sessionId: s } = await cdp.send('Target.attachToTarget', { targetId, flatten: true });
  try {
    await cdp.send('Page.enable', {}, s);
    await cdp.send('DOM.enable', {}, s);
    await cdp.send('Accessibility.enable', {}, s);
    await cdp.send('Emulation.setDeviceMetricsOverride', { width: vp.width, height: vp.height, deviceScaleFactor: 1, mobile: vp.mobile }, s);
    const loaded = cdp.waitFor('Page.loadEventFired', s);
    const nav = await cdp.send('Page.navigate', { url }, s);
    if (nav.errorText) throw new Error(`${url}: ${nav.errorText}`);
    await loaded;
    await cdp.send('Runtime.evaluate', { expression: 'document.fonts ? document.fonts.ready.then(() => 1) : 1', awaitPromise: true }, s);
    const { nodes } = await cdp.send('Accessibility.getFullAXTree', {}, s);
    const hrefs = new Map();
    for (const n of nodes) {
      if (!n.ignored && val(n.role) === 'link' && n.backendDOMNodeId) {
        const { node } = await cdp.send('DOM.describeNode', { backendNodeId: n.backendDOMNodeId }, s);
        const a = node.attributes || [];
        const i = a.indexOf('href');
        if (i >= 0) hrefs.set(n.backendDOMNodeId, a[i + 1]);
      }
    }
    const { cssContentSize: c } = await cdp.send('Page.getLayoutMetrics', {}, s);
    const shot = await cdp.send('Page.captureScreenshot', { format: 'png', captureBeyondViewport: true,
      clip: { x: 0, y: 0, width: vp.width, height: Math.max(vp.height, Math.ceil(c.height)), scale: 1 } }, s);
    return { walk: { url, viewport: [vp.width, vp.height], nodes: buildNodes(nodes, hrefs) }, png: Buffer.from(shot.data, 'base64') };
  } finally {
    await cdp.send('Target.closeTarget', { targetId }).catch(() => {});
  }
}

export async function main(argv) {
  let o;
  try { o = parseArgs(argv); } catch (e) { console.error(`walk: ${e.message}\n${USAGE}`); return 2; }
  if (o.help || !o.pages.length) { console.error(USAGE); return o.help ? 0 : 2; }
  if (typeof WebSocket === 'undefined') { console.error('walk: this Node has no global WebSocket; use Node >= 22'); return 2; }
  let chrome = null; let cdp;
  try {
    let wsUrl = o.cdp;
    if (!wsUrl) {
      const path = findChrome(o.chrome);
      if (!path) { console.error('walk: no Chrome or Chromium found. Install one, pass --chrome PATH, set CHROME_BIN, or attach with --cdp ws://...'); return 2; }
      chrome = await launchChrome(path); wsUrl = chrome.ws;
    }
    cdp = await Cdp.open(wsUrl);
    await mkdir(o.out, { recursive: true });
    for (const { name, url } of o.pages) {
      const passes = [['', DESKTOP], ...(o.phone ? [['.phone', PHONE]] : [])];
      for (const [suffix, vp] of passes) {
        const { walk, png } = await capture(cdp, url, vp);
        await writeFile(join(o.out, `${name}${suffix}.walk.json`), `${JSON.stringify(walk, null, 2)}\n`);
        await writeFile(join(o.out, `${name}${suffix}.png`), png);
        console.log(`${name}${suffix}: ${url}`);
      }
    }
    return 0;
  } catch (e) {
    console.error(`walk: ${e.message}`); return 1;
  } finally {
    try { cdp?.close(); } catch { /* closing */ }
    if (chrome) { chrome.proc.kill(); await new Promise((r) => setTimeout(r, 200)); await rm(chrome.dir, { recursive: true, force: true }).catch(() => {}); }
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  process.exit(await main(process.argv.slice(2)));
}

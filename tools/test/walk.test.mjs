// node --test tools/test/walk.test.mjs
// Unit tests need nothing. The capture tests need a local Chrome and Node >= 22 (global WebSocket); without them they skip.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { cpSync, mkdtempSync, readFileSync, readdirSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { test } from 'node:test';
import { fileURLToPath } from 'node:url';
import { buildNodes, findChrome, parseArgs, slug } from '../walk.mjs';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const WALK = join(ROOT, 'tools/walk.mjs');
const chrome = findChrome(null);
const skip = !chrome ? 'no Chrome/Chromium on this machine' : typeof WebSocket === 'undefined' ? 'Node has no global WebSocket (need >= 22)' : false;
if (skip) console.log(`# capture tests skipped: ${skip}`);

const flat = (ns, o = []) => { for (const n of ns) { o.push(n); flat(n.children || [], o); } return o; };
const run = (args) => spawnSync(process.execPath, [WALK, ...args], { encoding: 'utf8' });
const ax = (nodeId, role, name, childIds = [], extra = {}) => ({ nodeId, role: { value: role }, name: { value: name }, childIds, ...extra });

test('slug', () => {
  assert.equal(slug('Sign up free!'), 'sign-up-free');
  assert.equal(slug('  Café  Crème '), 'cafe-creme');
  assert.equal(slug('$0 forever'), '0-forever');
  assert.equal(slug('x'.repeat(80)).length, 40);
});

test('buildNodes: id rule, drops, de-duplication in document order, link href', () => {
  const nodes = [
    ax('1', 'RootWebArea', 'T', ['2']),
    ax('2', 'main', '', ['3', '4', '5', '6', '8']),
    ax('3', 'link', 'More', ['9'], { backendDOMNodeId: 11 }),
    ax('4', 'link', 'More', [], { backendDOMNodeId: 12 }),
    ax('5', 'generic', '', ['7']),
    ax('6', 'link', 'More', [], { backendDOMNodeId: 13 }),
    ax('7', 'paragraph', '', ['10']),
    ax('8', 'link', 'Gone', [], { ignored: true, backendDOMNodeId: 14 }),
    ax('9', 'StaticText', 'More'),
    ax('10', 'StaticText', 'Hello  world'),
  ];
  const hrefs = new Map([[11, 'a.html'], [12, '#x'], [13, 'mailto:a@b.c']]);
  const out = buildNodes(nodes, hrefs);
  assert.deepEqual(out, [{ id: 'main', role: 'main', name: '', children: [
    { id: 'link-more', role: 'link', name: 'More (a.html)' },
    { id: 'link-more-2', role: 'link', name: 'More (#x)' },
    { id: 'paragraph-hello-world', role: 'paragraph', name: 'Hello world' },
    { id: 'link-more-3', role: 'link', name: 'More (mailto:a@b.c)' },
  ] }]);
  assert.deepEqual(buildNodes(nodes, hrefs), out, 'same input, same output');
});

test('parseArgs', () => {
  const o = parseArgs(['--out', 's/', '--phone', 'p=https://x.test/a', 'h=fixtures/site/index.html']);
  assert.equal(o.phone, true);
  assert.equal(o.pages[0].url, 'https://x.test/a');
  assert.match(o.pages[1].url, /^file:\/\/\/.*fixtures\/site\/index\.html$/);
  assert.throws(() => parseArgs(['nonsense']));
});

test('a missing Chrome prints a clear message and exits 2', () => {
  const r = run(['--chrome', '/nonexistent/chrome', 'p=fixtures/site/pricing.html']);
  assert.equal(r.status, 2);
  assert.match(r.stderr, /no Chrome or Chromium found/);
});

const capture = (dir, extra = []) => {
  const r = run(['--out', dir, ...extra, `pricing=${join(ROOT, 'fixtures/site/pricing.html')}`, `home=${join(ROOT, 'fixtures/site/index.html')}`]);
  assert.equal(r.status, 0, r.stderr);
};
const load = (dir, f) => JSON.parse(readFileSync(join(dir, f), 'utf8'));

// A recorded node (hand-written id in fixtures/site-walk) is found in the capture by role + name + the multiset of its
// named descendants, so unnamed containers (article, list) are matched by their content, not by position.
const sig = (n) => JSON.stringify(flat(n.children || []).filter((d) => d.name).map((d) => `${d.role}|${d.name}`).sort());
const find = (cap, rec) => flat(cap).find((n) => n.role === rec.role && n.name === rec.name && sig(n) === sig(rec));

test('every node id cited by the fixture verdicts exists in a capture of fixtures/site', { skip }, () => {
  const dir = mkdtempSync(join(tmpdir(), 'walk-t-'));
  try {
    capture(dir);
    const vdir = join(ROOT, 'fixtures/run/verdicts');
    const cited = new Set();
    for (const f of readdirSync(vdir)) {
      for (const m of readFileSync(join(vdir, f), 'utf8').matchAll(/\b(home|pricing)\.walk\.json#([\w-]+)/g)) cited.add(`${m[1]}#${m[2]}`);
    }
    assert.ok(cited.size > 15, 'the verdicts cite walk nodes');
    const missing = []; const unrecorded = [];
    for (const c of cited) {
      const [page, id] = c.split('#');
      const rec = flat(load(join(ROOT, 'fixtures/site-walk'), `${page}.walk.json`).nodes).find((n) => n.id === id);
      if (!rec) { unrecorded.push(c); continue; }
      if (!find(load(dir, `${page}.walk.json`).nodes, rec)) missing.push(c);
    }
    assert.deepEqual(missing, [], 'cited nodes absent from the capture');
    // These belong to the comparator (fixtures/comparators/acme-pricing), not to fixtures/site.
    assert.deepEqual(unrecorded.sort(), ['pricing#btn-free', 'pricing#plans-table', 'pricing#price-business', 'pricing#price-free']);
  } finally { rmSync(dir, { recursive: true, force: true }); }
});

test('determinism: two captures give byte-identical walks; a copy of the site gives the same ids', { skip }, () => {
  const a = mkdtempSync(join(tmpdir(), 'walk-a-')); const b = mkdtempSync(join(tmpdir(), 'walk-b-')); const site = mkdtempSync(join(tmpdir(), 'walk-s-'));
  try {
    capture(a, ['--phone']); capture(b, ['--phone']);
    for (const f of readdirSync(a).filter((x) => x.endsWith('.walk.json'))) {
      assert.equal(readFileSync(join(a, f), 'utf8'), readFileSync(join(b, f), 'utf8'), f);
    }
    assert.ok(readdirSync(a).includes('pricing.phone.walk.json') && readdirSync(a).includes('pricing.phone.png'));
    assert.deepEqual(load(a, 'pricing.phone.walk.json').viewport, [390, 844]);
    cpSync(join(ROOT, 'fixtures/site'), site, { recursive: true });
    const c = mkdtempSync(join(tmpdir(), 'walk-c-'));
    const r = run(['--out', c, `pricing=${join(site, 'pricing.html')}`]);
    assert.equal(r.status, 0, r.stderr);
    const ids = (d) => flat(load(d, 'pricing.walk.json').nodes).map((n) => n.id);
    assert.deepEqual(ids(c), ids(a));
    rmSync(c, { recursive: true, force: true });
    const w = load(a, 'pricing.walk.json');
    assert.deepEqual(w.viewport, [1280, 800]);
    assert.equal(new Set(flat(w.nodes).map((n) => n.id)).size, flat(w.nodes).length, 'ids are unique');
    assert.ok(flat(w.nodes).some((n) => n.role === 'link' && /\(mailto:hello@lantern\.example\)$/.test(n.name)));
  } finally { for (const d of [a, b, site]) rmSync(d, { recursive: true, force: true }); }
});

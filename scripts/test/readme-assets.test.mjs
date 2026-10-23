// adapted from comparegraph scripts/test/readme-assets.test.mjs
// scripts/readme-assets.mjs: deterministic render of three data-driven illustrations; --check goes red when the data or the SVG drifts.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtempSync, readFileSync, writeFileSync, existsSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const script = path.join(root, 'scripts/readme-assets.mjs');
const run = (...a) => spawnSync('node', [script, ...a], { encoding: 'utf8' });
const json = (n) => path.join(root, `docs/assets/${n}.json`);
const svg = (n) => path.join(root, `docs/assets/${n}.svg`);

// One number per asset, changed in a temp copy.
const edits = {
  strata: (d) => { d.strata[0].subject.median += 1; },
  spend: (d) => { d.rows[0].bytes += 1000; },
  matrix: (d) => { d.strata[0].rows[0].scores[0] = d.strata[0].rows[0].scores[0] === 5 ? 1 : 5; },
};

for (const name of Object.keys(edits)) {
  test(`${name}: rendering twice gives byte-identical output`, () => {
    const dir = mkdtempSync(path.join(tmpdir(), 'jg-assets-'));
    const a = path.join(dir, 'a.svg');
    const b = path.join(dir, 'b.svg');
    assert.equal(run('--asset', name, '--in', json(name), '--out', a).status, 0);
    assert.equal(run('--asset', name, '--in', json(name), '--out', b).status, 0);
    assert.equal(readFileSync(a, 'utf8'), readFileSync(b, 'utf8'));
    assert.equal(readFileSync(a, 'utf8'), readFileSync(svg(name), 'utf8'), 'the committed SVG is the render');
  });

  test(`${name}: a changed number changes the output and --check against the committed SVG fails`, () => {
    const dir = mkdtempSync(path.join(tmpdir(), 'jg-assets-'));
    const data = JSON.parse(readFileSync(json(name), 'utf8'));
    edits[name](data);
    const edited = path.join(dir, `${name}.json`);
    writeFileSync(edited, JSON.stringify(data));
    const out = path.join(dir, 'out.svg');
    assert.equal(run('--asset', name, '--in', edited, '--out', out).status, 0);
    assert.notEqual(readFileSync(out, 'utf8'), readFileSync(svg(name), 'utf8'));
    assert.equal(run('--asset', name, '--in', edited, '--out', svg(name), '--check').status, 1);
  });
}

test('--check passes on the committed JSON and SVGs, and the two hand-drawn files exist', () => {
  const r = run('--check');
  assert.equal(r.status, 0, r.stderr);
  for (const n of Object.keys(edits)) assert.match(r.stdout, new RegExp(`${n}.svg is current`));
  for (const n of ['loop', 'refusal']) assert.ok(existsSync(svg(n)), `${n}.svg is committed`);
});

test('strata.json and matrix.json derive from fixtures/run/matrix.json: one moved score turns --check red', () => {
  const m = JSON.parse(readFileSync(json('matrix'), 'utf8'));
  assert.equal(m.cols.length, 21);
  assert.deepEqual(m.strata.map((s) => s.rows.length), [4, 2, 1]);
  const fixture = JSON.parse(readFileSync(path.join(root, 'fixtures/run/matrix.json'), 'utf8'));
  const cell = fixture[0].cells.find((c) => c.lens === 'compare' && c.dimension === 'clarity' && c.about === null);
  cell.scores['builder-ana'] = cell.scores['builder-ana'] === 5 ? 1 : 5;
  const dir = mkdtempSync(path.join(tmpdir(), 'jg-assets-'));
  const copy = path.join(dir, 'matrix.json');
  writeFileSync(copy, JSON.stringify(fixture));
  const r = run('--check', '--source', copy);
  assert.equal(r.status, 1);
  assert.match(r.stderr, /differs from what/);
});

test('spend.json carries the command and the measured bytes of the fixture run report (34,934 at spend 7 compact)', () => {
  const d = JSON.parse(readFileSync(json('spend'), 'utf8'));
  assert.match(d.command, /jurygraph brief/);
  assert.equal(d.rows.find((r) => r.spend === 7 && r.detail === 'compact').bytes, 34934);
});

test('every SVG is a card with a title and a desc', () => {
  for (const n of ['loop', 'refusal', ...Object.keys(edits)]) {
    const s = readFileSync(svg(n), 'utf8');
    assert.match(s, /role="img"/);
    assert.match(s, /<title id="t">.+<\/title><desc id="d">.+<\/desc>/);
  }
});

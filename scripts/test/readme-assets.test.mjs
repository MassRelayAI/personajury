// adapted from comparegraph scripts/test/readme-assets.test.mjs
// scripts/readme-assets.mjs: deterministic render of four data-driven illustrations; --check goes red when the data or the SVG drifts.
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
const HAND = ['loop', 'refusal', 'playbook-site', 'use-cases'];
const json = (n) => path.join(root, `docs/assets/${n}.json`);
const svg = (n) => path.join(root, `docs/assets/${n}.svg`);

// One number per asset, changed in a temp copy.
const edits = {
  strata: (d) => { d.strata[0].subject.median += 1; },
  spend: (d) => { d.rows[0].bytes += 1000; },
  matrix: (d) => { d.strata[0].rows[0].scores[0] = d.strata[0].rows[0].scores[0] === 5 ? 1 : 5; },
  rounds: (d) => { d.lenses[0].rows[0].round2[0] = d.lenses[0].rows[0].round2[0] === 5 ? 1 : 5; },
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

test('--check passes on the committed JSON and SVGs, and the four hand-drawn files exist', () => {
  const r = run('--check');
  assert.equal(r.status, 0, r.stderr);
  for (const n of Object.keys(edits)) assert.match(r.stdout, new RegExp(`${n}.svg is current`));
  for (const n of HAND) assert.ok(existsSync(svg(n)), `${n}.svg is committed`);
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
  assert.match(d.command, /jurypersona brief/);
  assert.equal(d.rows.find((r) => r.spend === 7 && r.detail === 'compact').bytes, 34934);
});

test('every SVG is a card with a title and a desc', () => {
  for (const n of [...HAND, ...Object.keys(edits)]) {
    const s = readFileSync(svg(n), 'utf8');
    assert.match(s, /role="img"/);
    assert.match(s, /<title id="t">.+<\/title><desc id="d">.+<\/desc>/);
  }
});

test('rounds.json is the public CompareGraph report: blockers 15 to 6 and 6 to 0, three strata, a link to the report', () => {
  const d = JSON.parse(readFileSync(json('rounds'), 'utf8'));
  assert.match(d.source, /^https:\/\/github\.com\/massrelay-ai\/comparegraph\/blob\/main\/docs\/reports\/2026-10-24-readme-jury\.md$/);
  assert.deepEqual(d.strata.map((s) => s.stake), ['primary', 'secondary', 'bystander']);
  assert.deepEqual([d.blockers.critical.total1, d.blockers.critical.total2], [15, 6]);
  assert.deepEqual([d.blockers.compare.total1, d.blockers.compare.total2], [6, 0]);
  const clarity = d.lenses.find((l) => l.lens === 'compare').rows.find((r) => r.dimension === 'clarity');
  assert.deepEqual([clarity.round1, clarity.round2], [[2, 3, 1], [4, 3, 4]]);
});

test('rounds: per-stratum blockers that do not sum to the total, or a median off the scale, are refused', () => {
  const dir = mkdtempSync(path.join(tmpdir(), 'jg-assets-'));
  for (const [why, edit, msg] of [
    ['sum', (d) => { d.blockers.critical.round2[0] += 1; }, /strata sum/],
    ['scale', (d) => { d.lenses[1].rows[0].round1[2] = 6; }, /outside 1 to 5/],
  ]) {
    const data = JSON.parse(readFileSync(json('rounds'), 'utf8'));
    edit(data);
    const p = path.join(dir, `${why}.json`);
    writeFileSync(p, JSON.stringify(data));
    const r = run('--asset', 'rounds', '--in', p, '--out', path.join(dir, `${why}.svg`));
    assert.equal(r.status, 1, why);
    assert.match(r.stderr, msg);
  }
});

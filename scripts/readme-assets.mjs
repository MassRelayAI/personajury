// adapted from comparegraph scripts/readme-assets.mjs
// Renders docs/assets/{strata,spend,matrix}.svg from the JSON of the same name. Deterministic, no dependencies.
// loop.svg and refusal.svg are hand-drawn: committed, edited by hand, never generated (--check only requires them to exist).
// strata.json and matrix.json are derived from fixtures/run/matrix.json and fixtures/run/verdicts/ (--derive, or a plain run).
// spend.json is measured by scripts/readme-assets-data.sh (it runs the binary).
// node scripts/readme-assets.mjs            derive the two JSON files, then write the three SVGs
// node scripts/readme-assets.mjs --check    exit 1 if a committed JSON or SVG differs from what would be written
// --asset <name> limits the run to one of them; with --in/--out (the test uses them) it defaults to spend.
// --source <matrix.json> reads the fixture matrix from another file (the test and the red/green proof use it).
import { readFileSync, writeFileSync, existsSync, readdirSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const args = process.argv.slice(2);
const opt = (name, dflt) => (args.includes(name) ? args[args.indexOf(name) + 1] : dflt);

const esc = (s) => String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
const num = (n) => n.toLocaleString('en-US');
const FONT = 'font-family="Helvetica, Arial, sans-serif" fill="#1f2328"';
const MONO = 'font-family="monospace"';
const INK = '#1f2328', MUTED = '#57606a', LINE = '#d0d7de', BLUE = '#0550ae';
const FILL = { blue: '#ddf4ff', yellow: '#fff8c5', green: '#dafbe1' };

// One white card with a border, so contrast never depends on the page behind it (README, crates.io, a PNG export).
const open = (w, h, title, desc) => `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${w} ${h}" width="${w}" role="img" aria-labelledby="t d">
<title id="t">${esc(title)}</title><desc id="d">${esc(desc)}</desc>
<rect width="100%" height="100%" fill="#ffffff"/>
<rect x="1" y="1" width="${w - 2}" height="${h - 2}" rx="10" fill="#ffffff" stroke="${LINE}" stroke-width="1.2"/>
<g ${FONT}>
`;
const close = '</g>\n</svg>\n';

// ---------------------------------------------------------------- derive

const median = (xs) => { const s = [...xs].sort((a, b) => a - b); return s[Math.floor((s.length - 1) / 2)]; };

export function deriveMatrix(fixture, verdicts) {
  const cols = fixture[0].cells.map((c) => ({
    lens: c.lens, dimension: c.dimension, about: c.about,
    label: c.lens === 'critical' ? c.dimension : (c.about ?? 'subject'),
  }));
  const strata = fixture.map((s) => ({
    stake: s.stake,
    rows: s.personas.map((p) => ({
      persona: p,
      scores: s.cells.map((c) => c.scores[p]),
      blockers: verdicts[p] ?? 0,
    })),
    median: s.cells.map((c) => c.median),
  }));
  return { title: 'The fixture jury, every score', subject: 'Lantern Notes', cols, strata };
}

export function deriveStrata(fixture) {
  const dim = { lens: 'compare', dimension: 'clarity' };
  const comparator = 'cmp:acme-pricing';
  const pick = (s, about) => s.cells.find((c) => c.lens === dim.lens && c.dimension === dim.dimension && c.about === about);
  const strata = fixture.map((s) => {
    const a = pick(s, null), b = pick(s, comparator);
    const part = (c) => ({ scores: c.scores, median: c.median, min: c.min, max: c.max });
    return { stake: s.stake, personas: s.personas, subject: part(a), comparator: part(b) };
  });
  const all = strata.flatMap((s) => Object.values(s.subject.scores));
  return {
    title: 'The same question, asked of three strata',
    dimension: `${dim.lens}:${dim.dimension}`, subject: 'Lantern Notes', comparator,
    pooled_mean: Math.round((all.reduce((x, y) => x + y, 0) / all.length) * 10) / 10,
    strata,
  };
}

function readVerdictBlockers(dir) {
  const out = {};
  if (!existsSync(dir)) return out;
  for (const f of readdirSync(dir).sort()) {
    const v = JSON.parse(readFileSync(path.join(dir, f), 'utf8'));
    out[v.persona] = (out[v.persona] ?? 0) + v.lines.reduce((a, l) => a + l.blockers.length, 0);
  }
  return out;
}

function derived(sourcePath) {
  const fixture = JSON.parse(readFileSync(sourcePath, 'utf8'));
  const verdicts = readVerdictBlockers(path.join(path.dirname(sourcePath), 'verdicts'));
  return { strata: deriveStrata(fixture), matrix: deriveMatrix(fixture, verdicts) };
}
const asJson = (o) => JSON.stringify(o, null, 1) + '\n';

// ---------------------------------------------------------------- strata

const STAKE = {
  primary: { fill: FILL.blue, who: 'would use it' },
  secondary: { fill: FILL.yellow, who: 'buys, approves or supports it' },
  bystander: { fill: FILL.green, who: 'passes by' },
};

export function renderStrata(d) {
  const W = 1050, H = 556;
  const pw = 316, gap = 31, left = 30, top = 92, ph = 418;
  const lane = [{ key: 'subject', name: d.subject, color: '#0969da', y0: 0 }, { key: 'comparator', name: d.comparator, color: '#bf8700', y0: 1 }];
  const desc = `The ${d.dimension} dimension, one dot per persona on a 1 to 5 scale. ` + d.strata.map((s) => {
    const n = s.personas.length;
    return `${s.stake} (${n} persona${n > 1 ? 's' : ''}): ${d.subject} median ${s.subject.median}, range ${s.subject.min} to ${s.subject.max}; ${d.comparator} median ${s.comparator.median}, range ${s.comparator.min} to ${s.comparator.max}`;
  }).join('. ') + `. Pooling every persona into one mean gives ${d.pooled_mean.toFixed(1)}, a number the engine refuses to print.`;
  let o = open(W, H, `${d.title}: ${d.dimension}`, desc);
  o += `<text x="30" y="40" font-size="18" font-weight="700">${esc(d.title)}</text>
<text x="30" y="62" font-size="13" fill="${MUTED}">One dot per persona, <tspan ${MONO} fill="${BLUE}">${esc(d.dimension)}</tspan>, 1 to 5. Bar: median. Line: lowest to highest.</text>
<text x="${W - 30}" y="40" font-size="16" text-anchor="end" fill="${MUTED}" text-decoration="line-through">all strata: ${d.pooled_mean.toFixed(1)}</text>
<text x="${W - 30}" y="62" font-size="13" text-anchor="end" fill="#cf222e">the engine refuses this row</text>
`;
  d.strata.forEach((s, i) => {
    const px = left + i * (pw + gap);
    const ax0 = px + 34, ax1 = px + pw - 34;
    const xs = (v) => ax0 + ((v - 1) / 4) * (ax1 - ax0);
    const n = s.personas.length;
    o += `<rect x="${px}" y="${top}" width="${pw}" height="${ph}" rx="8" fill="#ffffff" stroke="${LINE}" stroke-width="1.2"/>
<rect x="${px + 16}" y="${top + 16}" width="${s.stake.length * 9 + 24}" height="26" rx="13" fill="${STAKE[s.stake].fill}" stroke="${MUTED}" stroke-width="1"/>
<text x="${px + 28}" y="${top + 34}" font-size="14" font-weight="700">${esc(s.stake)}</text>
<text x="${px + 16}" y="${top + 62}" font-size="12" fill="${MUTED}">${esc(STAKE[s.stake].who)} · ${n} persona${n > 1 ? 's' : ''}</text>
`;
    for (const l of lane) {
      const part = s[l.key];
      const y0 = top + 100 + l.y0 * 120; // lane label baseline
      const cy = y0 + 42;
      o += `<text x="${px + 16}" y="${y0}" font-size="12" font-weight="700">${esc(l.name)}</text>
<line x1="${ax0}" y1="${cy + 4}" x2="${ax1}" y2="${cy + 4}" stroke="${LINE}" stroke-width="1"/>
`;
      if (part.max > part.min) {
        o += `<line x1="${xs(part.min)}" y1="${cy}" x2="${xs(part.max)}" y2="${cy}" stroke="${MUTED}" stroke-width="1.5"/>
<line x1="${xs(part.min)}" y1="${cy - 6}" x2="${xs(part.min)}" y2="${cy + 6}" stroke="${MUTED}" stroke-width="1.5"/>
<line x1="${xs(part.max)}" y1="${cy - 6}" x2="${xs(part.max)}" y2="${cy + 6}" stroke="${MUTED}" stroke-width="1.5"/>
`;
      }
      o += `<line x1="${xs(part.median)}" y1="${cy - 30}" x2="${xs(part.median)}" y2="${cy + 30}" stroke="${INK}" stroke-width="3" stroke-linecap="round"/>
`;
      const stack = new Map();
      for (const p of s.personas) {
        const v = part.scores[p];
        const k = stack.get(v) ?? 0;
        stack.set(v, k + 1);
      }
      const seen = new Map();
      for (const p of s.personas) {
        const v = part.scores[p];
        const k = seen.get(v) ?? 0;
        seen.set(v, k + 1);
        const total = stack.get(v);
        const dy = (k - (total - 1) / 2) * 12;
        o += `<circle cx="${xs(v)}" cy="${cy + dy}" r="5.5" fill="${l.color}" stroke="#ffffff" stroke-width="1.5"/>
`;
      }
      o += `<text x="${px + 16}" y="${cy + 48}" font-size="12" fill="${MUTED}">median <tspan font-weight="700" fill="${INK}">${part.median}</tspan> · range ${part.min} to ${part.max}</text>
`;
    }
    const ay = top + 338;
    o += `<line x1="${ax0}" y1="${ay}" x2="${ax1}" y2="${ay}" stroke="${MUTED}" stroke-width="1.2"/>
`;
    for (let v = 1; v <= 5; v++) {
      o += `<line x1="${xs(v)}" y1="${ay}" x2="${xs(v)}" y2="${ay + 5}" stroke="${MUTED}" stroke-width="1.2"/><text x="${xs(v)}" y="${ay + 20}" font-size="12" text-anchor="middle" fill="${MUTED}">${v}</text>
`;
    }
    o += `<line x1="${px + 16}" y1="${top + ph - 38}" x2="${px + pw - 16}" y2="${top + ph - 38}" stroke="${LINE}" stroke-width="1"/>
<text x="${px + 16}" y="${top + ph - 14}" font-size="13">The comparator leads by <tspan font-weight="700">${s.comparator.median - s.subject.median}</tspan>: median ${s.comparator.median} to ${s.subject.median}</text>
`;
  });
  o += `<g font-size="12" fill="${MUTED}"><circle cx="${W - 330}" cy="${H - 22}" r="5.5" fill="#0969da"/><text x="${W - 318}" y="${H - 18}">${esc(d.subject)}</text><circle cx="${W - 190}" cy="${H - 22}" r="5.5" fill="#bf8700"/><text x="${W - 178}" y="${H - 18}">${esc(d.comparator)}</text></g>
<text x="30" y="${H - 18}" font-size="12" fill="${MUTED}">Fixture run, 7 personas, report 2026-10-23-v0-1-measured. A fixture we wrote, not a benchmark.</text>
`;
  return o + close;
}

// ---------------------------------------------------------------- spend

export function renderSpend(d) {
  const W = 1050;
  const rows = d.rows;
  const spends = [...new Set(rows.map((r) => r.spend))];
  const max = Math.max(...rows.map((r) => r.bytes));
  const x0 = 180, barMax = 460, barH = 28, groupH = 92, top = 112;
  const H = top + spends.length * groupH + 86;
  const fills = { compact: FILL.blue, full: FILL.yellow };
  const desc = `Brief bytes for the critical lens on the fixture workspace: ` + spends.map((sp) => {
    const r = rows.filter((x) => x.spend === sp);
    return `spend ${sp}, ${r.map((x) => `${x.detail} ${num(x.bytes)} bytes`).join(' and ')}, the project's estimate ${num(sp * d.tokens_per_brief)} tokens`;
  }).join('; ') + `. ${d.caption}`;
  let o = open(W, H, d.title, desc);
  o += `<text x="30" y="40" font-size="18" font-weight="700">${esc(d.title)}</text>
<rect x="30" y="58" width="14" height="14" rx="3" fill="${fills.compact}" stroke="${MUTED}" stroke-width="1"/><text x="52" y="70" font-size="13">detail <tspan ${MONO} fill="${BLUE}">compact</tspan>: attachments by path and hash</text>
<rect x="390" y="58" width="14" height="14" rx="3" fill="${fills.full}" stroke="${MUTED}" stroke-width="1"/><text x="412" y="70" font-size="13">detail <tspan ${MONO} fill="${BLUE}">full</tspan>: text attachments inline</text>
<text x="${W - 30}" y="${top - 18}" font-size="12" text-anchor="end" fill="${MUTED}">the toml's own estimate</text>
`;
  spends.forEach((sp, g) => {
    const y = top + g * groupH;
    const r = rows.filter((x) => x.spend === sp);
    o += `<text x="30" y="${y + 24}" font-size="16" font-weight="700">spend ${sp}</text>
<text x="30" y="${y + 44}" font-size="12" fill="${MUTED}">${r[0].briefs} brief${r[0].briefs > 1 ? 's' : ''}</text>
`;
    r.forEach((x, k) => {
      const by = y + k * (barH + 8);
      const w = Math.max(2, Math.round((x.bytes / max) * barMax));
      o += `<text x="${x0 - 10}" y="${by + 19}" font-size="12" text-anchor="end" ${MONO} fill="${BLUE}">${esc(x.detail)}</text>
<rect x="${x0}" y="${by}" width="${w}" height="${barH}" rx="4" fill="${fills[x.detail]}" stroke="${MUTED}" stroke-width="1.2"/>
<text x="${x0 + w + 10}" y="${by + 19}" font-size="14" font-weight="700">${num(x.bytes)} <tspan font-size="12" font-weight="400" fill="${MUTED}">bytes</tspan></text>
`;
    });
    o += `<text x="${W - 30}" y="${y + 40}" font-size="18" font-weight="700" text-anchor="end">≈ ${num(sp * d.tokens_per_brief)} tokens</text>
${g < spends.length - 1 ? `<line x1="30" y1="${y + groupH - 14}" x2="${W - 30}" y2="${y + groupH - 14}" stroke="#eaeef2" stroke-width="1"/>` : ''}
`;
  });
  const by = top + spends.length * groupH;
  o += `<line x1="${x0}" y1="${top - 8}" x2="${x0}" y2="${by - 18}" stroke="${MUTED}" stroke-width="1.2"/>
<text x="${W / 2}" y="${by + 14}" font-size="14" font-weight="700" text-anchor="middle">${esc(d.caption.split('. ')[0])}.</text>
<text x="${W / 2}" y="${by + 36}" font-size="12" text-anchor="middle" fill="${MUTED}">${tokenNote(d)}</text>
`;
  return o + close;
}
const tokenNote = (d) => `Bytes are measured on the fixture (critical lens, linear scale). The token figure is ${num(d.tokens_per_brief)} per brief from fixtures/jurygraph.toml, an estimate.`;

// ---------------------------------------------------------------- matrix

const STEP = ['#eaf5ff', '#b6e3ff', '#80ccff', '#54aeff', '#0969da'];

export function renderMatrix(d) {
  const W = 1050;
  const cw = 35, rh = 26, gapG = 8, gx0 = 196;
  const groupOf = (c) => (c.lens === 'critical' ? 'critical' : `${c.lens}:${c.dimension}`);
  const colX = [];
  let x = gx0, prev = null;
  d.cols.forEach((c) => {
    const g = groupOf(c);
    if (prev !== null && g !== prev) x += gapG;
    colX.push(x);
    x += cw;
    prev = g;
  });
  const gridEnd = x;
  const bx = gridEnd + 16; // blockers column
  const spans = (keyOf) => {
    const out = [];
    d.cols.forEach((c, i) => {
      const k = keyOf(c);
      const last = out[out.length - 1];
      if (last && last.k === k) last.to = i; else out.push({ k, from: i, to: i });
    });
    return out;
  };
  const gridTop = 258;
  const nRows = d.strata.reduce((a, s) => a + s.rows.length + 1, 0);
  const H = gridTop + nRows * rh + d.strata.length * 40 + 96;
  const total = d.strata.reduce((a, s) => a + s.rows.length, 0);
  const desc = `${d.title}: ${total} personas in ${d.strata.length} strata (${d.strata.map((s) => `${s.stake} ${s.rows.length}`).join(', ')}), ${d.cols.length} columns of lens, dimension and option. Each cell is a score from 1 to 5. ` +
    d.strata.map((s) => `${s.stake} medians ${s.median.join(' ')}`).join('. ') + `. Blockers per persona: ` +
    d.strata.flatMap((s) => s.rows.map((r) => `${r.persona} ${r.blockers}`)).join(', ') + '. No row spans strata.';
  let o = open(W, H, `${d.title}, ${d.subject}`, desc);
  o += `<text x="30" y="40" font-size="18" font-weight="700">${esc(d.title)}: ${esc(d.subject)}, ${total} personas, ${d.cols.length} columns</text>
<text x="30" y="62" font-size="13" fill="${MUTED}">One row per persona, grouped by stake. The score is printed in every cell; the fill only helps the eye. No row spans strata.</text>
`;
  // lens and dimension headers
  for (const sp of spans((c) => c.lens)) {
    const a = colX[sp.from], b = colX[sp.to] + cw;
    o += `<text x="${(a + b) / 2}" y="94" font-size="13" font-weight="700" text-anchor="middle" ${MONO} fill="${BLUE}">${esc(sp.k)}</text>
<line x1="${a + 2}" y1="102" x2="${b - 2}" y2="102" stroke="${MUTED}" stroke-width="1.2"/>
`;
  }
  for (const sp of spans((c) => (c.lens === 'critical' ? '' : c.dimension))) {
    if (!sp.k) continue;
    const a = colX[sp.from], b = colX[sp.to] + cw;
    o += `<text x="${(a + b) / 2}" y="122" font-size="12" font-weight="700" text-anchor="middle">${esc(sp.k)}</text>
<line x1="${a + 2}" y1="129" x2="${b - 2}" y2="129" stroke="${LINE}" stroke-width="1"/>
`;
  }
  d.cols.forEach((c, i) => {
    o += `<text transform="translate(${colX[i] + cw / 2 + 4} ${gridTop - 8}) rotate(-90)" font-size="12" fill="${INK}">${esc(c.label)}</text>
`;
  });
  o += `<text transform="translate(${bx + cw / 2 + 4} ${gridTop - 8}) rotate(-90)" font-size="12" fill="${INK}">blockers</text>
`;
  let y = gridTop;
  for (const s of d.strata) {
    o += `<rect x="30" y="${y + 6}" width="${bx + cw - 30}" height="26" rx="6" fill="${STAKE[s.stake].fill}" stroke="${LINE}" stroke-width="1"/>
<text x="44" y="${y + 24}" font-size="13" font-weight="700">${esc(s.stake)}</text>
<text x="124" y="${y + 24}" font-size="12" fill="${MUTED}">${esc(STAKE[s.stake].who)}</text>
`;
    y += 40;
    const cell = (cx, cy, v, bold) => `<rect x="${cx + 1}" y="${cy + 1}" width="${cw - 2}" height="${rh - 2}" rx="3" fill="${STEP[v - 1]}"/><text x="${cx + cw / 2}" y="${cy + 18}" font-size="13" ${bold ? 'font-weight="700"' : ''} text-anchor="middle" fill="${v === 5 ? '#ffffff' : INK}">${v}</text>
`;
    for (const r of s.rows) {
      o += `<text x="${gx0 - 10}" y="${y + 18}" font-size="12" text-anchor="end">${esc(r.persona)}</text>
`;
      r.scores.forEach((v, i) => { o += cell(colX[i], y, v, false); });
      o += `<text x="${bx + cw / 2}" y="${y + 18}" font-size="13" text-anchor="middle" ${r.blockers ? 'font-weight="700" fill="#cf222e"' : `fill="${MUTED}"`}>${r.blockers}</text>
`;
      y += rh;
    }
    o += `<line x1="44" y1="${y + 1}" x2="${gridEnd}" y2="${y + 1}" stroke="${MUTED}" stroke-width="1.2"/>
<text x="${gx0 - 10}" y="${y + 19}" font-size="12" font-weight="700" text-anchor="end">median</text>
`;
    s.median.forEach((v, i) => { o += cell(colX[i], y + 2, v, true); });
    y += rh + 2;
  }
  const ly = y + 30;
  o += `<text x="30" y="${ly + 4}" font-size="12" fill="${MUTED}">score</text>
`;
  STEP.forEach((f, i) => {
    o += `<rect x="${72 + i * 38}" y="${ly - 12}" width="34" height="22" rx="3" fill="${f}" stroke="${LINE}" stroke-width="1"/><text x="${72 + i * 38 + 17}" y="${ly + 4}" font-size="13" text-anchor="middle" fill="${i === 4 ? '#ffffff' : INK}">${i + 1}</text>
`;
  });
  o += `<text x="290" y="${ly + 4}" font-size="12" fill="${MUTED}">1 is the bottom of the lens's anchors, 5 the top. Blockers: the points a persona marked as stopping it.</text>
<text x="30" y="${ly + 34}" font-size="12" fill="${MUTED}">Fixture run, report 2026-10-23-v0-1-measured: a fixture we wrote, not a benchmark. Medians are per stratum and per column.</text>
`;
  return o + close;
}

// ---------------------------------------------------------------- run

const HAND = ['loop', 'refusal'];
const assets = { strata: renderStrata, spend: renderSpend, matrix: renderMatrix };
const check = args.includes('--check');
const custom = args.includes('--in') || args.includes('--out');
const names = args.includes('--asset') ? [opt('--asset')] : custom ? ['spend'] : Object.keys(assets);
const source = opt('--source', path.join(root, 'fixtures/run/matrix.json'));
let bad = false;

if (!custom) {
  for (const h of HAND) {
    if (!existsSync(path.join(root, `docs/assets/${h}.svg`))) { console.error(`docs/assets/${h}.svg (hand-drawn) is missing`); bad = true; }
    else if (check) console.log(`${h}.svg is present (hand-drawn)`);
  }
  const dv = derived(source);
  for (const n of ['strata', 'matrix']) {
    if (names.includes(n) === false) continue;
    const p = path.join(root, `docs/assets/${n}.json`);
    const want = asJson(dv[n]);
    if (check) {
      if ((existsSync(p) ? readFileSync(p, 'utf8') : '') !== want) { console.error(`${p} differs from what ${source} derives; run node scripts/readme-assets.mjs`); bad = true; }
    } else writeFileSync(p, want);
  }
}
for (const name of names) {
  if (!assets[name]) { console.error(`unknown asset ${name}`); process.exit(2); }
  const inPath = opt('--in', path.join(root, `docs/assets/${name}.json`));
  const outPath = opt('--out', path.join(root, `docs/assets/${name}.svg`));
  const svg = assets[name](JSON.parse(readFileSync(inPath, 'utf8')));
  if (check) {
    const have = existsSync(outPath) ? readFileSync(outPath, 'utf8') : '';
    if (have !== svg) { console.error(`${outPath} differs from the render of ${inPath}; run node scripts/readme-assets.mjs`); bad = true; }
    else console.log(`${name}.svg is current`);
  } else {
    writeFileSync(outPath, svg);
  }
}
if (bad) process.exit(1);

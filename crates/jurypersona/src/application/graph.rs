//! graph: the jury as one static SVG (D11). Personas in three stratum columns, the dimensions they scored in a column
//! per lens, the findings on the right; edges coloured by score or by decision, a median badge per stratum on every
//! dimension node (never one combined number), and, over several runs, each finding's life as a strip of dots.
//! Deterministic: integer layout from `domain::graphlayout`, ordered containers, no clock, no random.
use super::check::{lives, Life, State};
use super::verdict::{load_lens, load_run, open, score_rows};
use super::{AppError, Result};
use crate::domain::graphlayout::{self as lay, Item};
use crate::domain::*;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;

// the README asset palette (docs/assets/*.svg, brand.md)
const INK: &str = "#1f2328";
const MUTED: &str = "#57606a";
const LINE: &str = "#d0d7de";
const BLUE: &str = "#0550ae";
const RED: &str = "#cf222e";
const GREEN: &str = "#1a7f37";
const GREY: &str = "#8c959f";
const LINK: &str = "#0969da";
/// the score scale, one step per score 1..=5 (the README matrix ramp, darkened so a low step still shows on white)
const STEP: [&str; 5] = ["#b4d3ee", "#86bbe9", "#4f9fe3", "#1f78d1", "#0550ae"];
const FONT: &str = "system-ui, -apple-system, 'Segoe UI', Helvetica, Arial, sans-serif";
const MONO: &str = "ui-monospace, SFMono-Regular, Menlo, Consolas, monospace";

/// y of the column headers and of the first row of nodes
/// the most dots a finding's strip draws: the last rounds, a mark for the earlier ones
const MAX_DOTS: usize = 8;
const HEAD_Y: i32 = 92;
const TOP: i32 = 150;

fn esc(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => o.push_str("&amp;"),
            '<' => o.push_str("&lt;"),
            '>' => o.push_str("&gt;"),
            '"' => o.push_str("&quot;"),
            // not XML 1.0 characters: C0 controls but tab, LF and CR, and the non-characters U+FFFE and U+FFFF
            c if ((c as u32) < 0x20 && !matches!(c, '\n' | '\t' | '\r')) || c == '\u{FFFE}' || c == '\u{FFFF}' => {}
            c => o.push(c),
        }
    }
    o
}

fn tint(stake: Stake) -> &'static str {
    match stake {
        Stake::Primary => "#ddf4ff",
        Stake::Secondary => "#fff8c5",
        Stake::Bystander => "#dafbe1",
    }
}
fn meaning(stake: Stake) -> &'static str {
    match stake {
        Stake::Primary => "would use it",
        Stake::Secondary => "approves it",
        Stake::Bystander => "passes by",
    }
}
/// (stroke, fill) of a finding by its status
fn status_colours(status: &str) -> (&'static str, &'static str) {
    match status {
        "fixed" => (GREEN, "#dafbe1"),
        "dismissed" => (GREY, "#f6f8fa"),
        "row" => (LINK, "#ddf4ff"),
        _ => (RED, "#ffebe9"),
    }
}
fn step(score: i64) -> &'static str {
    STEP[(score.clamp(1, 5) - 1) as usize]
}
fn stake_idx(s: Stake) -> usize {
    match s {
        Stake::Primary => 0,
        Stake::Secondary => 1,
        Stake::Bystander => 2,
    }
}
const STAKES: [Stake; 3] = [Stake::Primary, Stake::Secondary, Stake::Bystander];

/// A node of a lens lane: a dimension, or an `@about` option under it.
struct Node {
    lens: String,
    dim: String,
    about: Option<String>,
    /// the text on the node
    label: String,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    /// per stratum: (median, personas)
    medians: [Option<(i64, usize)>; 3],
}
impl Node {
    fn key(&self) -> String {
        match &self.about {
            Some(a) => format!("{}:{}@{a}", self.lens, self.dim),
            None => format!("{}:{}", self.lens, self.dim),
        }
    }
    /// where an edge arrives: the node's left side
    fn left(&self) -> (i32, i32) {
        let ind = if self.about.is_some() { lay::SUB_INDENT } else { 0 };
        (self.x + ind, self.y + self.h / 2)
    }
}

struct PersonaBox {
    id: String,
    name: String,
    stake: Stake,
    x: i32,
    y: i32,
}

pub fn render(root: &Path, run_ids: &[i64]) -> Result<String> {
    let store = open(root)?;
    let mut ids: Vec<i64> = run_ids.to_vec();
    ids.sort_unstable();
    ids.dedup();
    if ids.is_empty() {
        return Err(AppError::InvalidArguments("graph names at least one run".into()));
    }
    let mut runs: Vec<RunRow> = Vec::new();
    for id in &ids {
        let run = load_run(&store, *id)?;
        if let Some(first) = runs.first() {
            if first.subject_name != run.subject_name {
                return Err(AppError::InvalidArguments(format!(
                    "a graph is over one subject: run {} is on {}, run {} is on {}",
                    first.id, first.subject_name, run.id, run.subject_name
                )));
            }
        }
        runs.push(run);
    }
    let subject = runs[0].subject_name.clone();

    // one stake per persona, as the matrix demands
    let mut stakes: BTreeMap<String, Stake> = BTreeMap::new();
    for r in &runs {
        for p in &r.personas {
            if let Some(s) = r.stakes.get(p) {
                if stakes.insert(p.clone(), *s).is_some_and(|old| old != *s) {
                    return Err(AppError::InvalidArguments(format!(
                        "one stake per persona in a graph: {p} has two stakes in runs {ids:?}"
                    )));
                }
            }
        }
    }

    // the lenses in order of their first run; scores come from the latest run of each
    let mut lenses: Vec<Lens> = Vec::new();
    let mut latest: BTreeMap<String, RunRow> = BTreeMap::new();
    for r in &runs {
        let lens = load_lens(&store, r)?;
        if !lenses.iter().any(|l| l.name == lens.name) {
            lenses.push(lens);
        }
        latest.insert(r.lens_name.clone(), r.clone());
    }
    let mut rows = Vec::new();
    let mut line_rows: Vec<(String, VerdictRow)> = Vec::new();
    for l in &lenses {
        let run = &latest[&l.name];
        let vr = store.verdict_rows(run.id, false)?;
        rows.extend(score_rows(run, &vr));
        line_rows.extend(vr.into_iter().map(|v| (l.name.clone(), v)));
    }
    let matrix = matrix::build(&rows, &stakes, &lenses, Aggregate::PerStratum)?;

    // personas by stake, then id
    let mut persona_ids: BTreeSet<String> = BTreeSet::new();
    for r in &runs {
        persona_ids.extend(r.personas.iter().cloned());
    }
    let mut by_stake: [Vec<PersonaBox>; 3] = [vec![], vec![], vec![]];
    for id in &persona_ids {
        let Some(stake) = stakes.get(id).copied() else { continue };
        let name = store.persona(id)?.map_or_else(|| id.clone(), |p| p.name);
        by_stake[stake_idx(stake)].push(PersonaBox { id: id.clone(), name, stake, x: 0, y: 0 });
    }

    // dimension nodes, lens by lens: each dimension, then its options (sorted; the whole subject is the dimension)
    let mut nodes_by_lens: Vec<Vec<Node>> = Vec::new();
    for l in &lenses {
        let mut nodes = Vec::new();
        for d in &l.dimensions {
            let abouts: BTreeSet<&str> = matrix
                .strata
                .iter()
                .flat_map(|s| s.cells.iter())
                .filter(|c| c.lens == l.name && c.dimension == d.id)
                .filter_map(|c| c.about.as_deref())
                .collect();
            let medians_of = |about: Option<&str>| {
                let mut m: [Option<(i64, usize)>; 3] = [None; 3];
                for s in &matrix.strata {
                    if let Some(c) = s.cells.iter().find(|c| c.lens == l.name && c.dimension == d.id && c.about.as_deref() == about) {
                        m[stake_idx(s.stake)] = Some((c.median, c.n));
                    }
                }
                m
            };
            let whole = medians_of(None);
            nodes.push(Node {
                lens: l.name.clone(),
                dim: d.id.clone(),
                about: None,
                label: d.id.clone(),
                x: 0,
                y: 0,
                w: 0,
                h: lay::DIM_H,
                medians: whole,
            });
            let list: Vec<String> = abouts.iter().map(|a| a.to_string()).collect();
            for a in list {
                let m = medians_of(Some(&a));
                nodes.push(Node {
                    lens: l.name.clone(),
                    dim: d.id.clone(),
                    about: Some(a.clone()),
                    label: a,
                    x: 0,
                    y: 0,
                    w: 0,
                    h: lay::SUB_H,
                    medians: m,
                });
            }
        }
        nodes_by_lens.push(nodes);
    }

    // findings, in the order of their persona's box, then of appearance
    let all_lives = lives(&store, &runs)?;
    let mut order: Vec<(usize, usize)> = Vec::new();
    {
        let mut pos: BTreeMap<&str, usize> = BTreeMap::new();
        let mut n = 0;
        for col in &by_stake {
            for p in col {
                pos.insert(&p.id, n);
                n += 1;
            }
        }
        for (i, l) in all_lives.iter().enumerate() {
            order.push((pos.get(l.finding.persona.as_str()).copied().unwrap_or(usize::MAX), i));
        }
        order.sort();
    }
    let findings: Vec<&Life> = order.iter().map(|(_, i)| &all_lives[*i]).collect();

    // ---- geometry ----
    let lane_count = lenses.len().clamp(1, 2);
    let mut want = vec![0i32; lane_count];
    for (i, nodes) in nodes_by_lens.iter().enumerate() {
        for n in nodes {
            let ind = if n.about.is_some() { lay::SUB_INDENT } else { 0 };
            want[i % lane_count] = want[i % lane_count].max(ind + lay::text_px(&n.label, if n.about.is_some() { 10 } else { 11 }));
        }
        want[i % lane_count] = want[i % lane_count].max(lay::text_px(&lenses[i].name, 13));
    }
    let lane_xw = lay::lanes(&want);
    // each lane is a flow of its lenses' nodes; a lens after the first in a lane gets a header row
    let mut lane_items: Vec<Vec<Item>> = vec![vec![]; lane_count];
    let mut lane_slots: Vec<Vec<(usize, Option<usize>)>> = vec![vec![]; lane_count]; // (lens, node) ; node None = header row
    for (i, nodes) in nodes_by_lens.iter().enumerate() {
        let lane = i % lane_count;
        if i >= lane_count {
            lane_items[lane].push(Item { h: 22, gap_before: 26 });
            lane_slots[lane].push((i, None));
        }
        for (j, n) in nodes.iter().enumerate() {
            let gap = if n.about.is_none() { 12 } else { 4 };
            let gap = if lane_items[lane].is_empty() { 0 } else if matches!(lane_slots[lane].last(), Some((_, None))) { 8 } else { gap };
            lane_items[lane].push(Item { h: n.h, gap_before: gap });
            lane_slots[lane].push((i, Some(j)));
        }
    }
    let lane_flows: Vec<(Vec<i32>, i32)> = lane_items.iter().map(|it| lay::flow(it)).collect();
    let (find_ys, find_h) = lay::stack(findings.len(), lay::FINDING_H, lay::FINDING_GAP);
    // the stratum columns descend like stairs: each starts below the one before it, under its own title
    // above 30 personas the stairs would make the card grow with the sum of the strata: the columns sit side by side
    let side_by_side = by_stake.iter().map(Vec::len).sum::<usize>() > 30;
    let mut col_top = [TOP; 3];
    let mut stair_y = TOP;
    let mut tallest = 0;
    for (s, col) in by_stake.iter_mut().enumerate() {
        let (ys, h) = lay::stack(col.len(), lay::PERSONA_H, lay::PERSONA_GAP);
        col_top[s] = stair_y;
        for (p, y) in col.iter_mut().zip(ys) {
            p.x = lay::MARGIN + s as i32 * (lay::PERSONA_W + lay::STRATUM_GAP);
            p.y = stair_y + y;
        }
        tallest = tallest.max(h);
        if !side_by_side {
            stair_y += h + lay::STAIR_GAP + lay::TITLE_H;
        }
    }
    let persona_bottom = if side_by_side { TOP + tallest } else { stair_y - lay::STAIR_GAP - lay::TITLE_H };
    let multi_lens_rounds = lenses.iter().any(|l| runs.iter().filter(|r| r.lens_name == l.name).count() > 1);
    let legend_h = if multi_lens_rounds { 190 } else { 144 };
    let legend_y = persona_bottom + 40;
    let content_h = lane_flows
        .iter()
        .map(|f| f.1)
        .chain([find_h, legend_y + legend_h - TOP])
        .max()
        .unwrap_or(0)
        .max(60);

    for (lane, (ys, _)) in lane_flows.iter().enumerate() {
        for (slot, y) in lane_slots[lane].iter().zip(ys) {
            if let (i, Some(j)) = slot {
                let n = &mut nodes_by_lens[*i][*j];
                n.x = lane_xw[lane].0;
                n.w = lane_xw[lane].1;
                n.y = TOP + y;
            }
        }
    }
    let find_x = lay::WIDTH - lay::MARGIN - lay::FINDING_W;
    let find_y = |i: usize| TOP + find_ys[i];

    // rounds: some lens was run more than once, so a finding has a life to draw (runs of different lenses do not)
    let multi = multi_lens_rounds;
    let height = TOP + content_h + 28;

    // ---- the words ----
    let counts: Vec<usize> = by_stake.iter().map(Vec::len).collect();
    let n_personas: usize = counts.iter().sum();
    let n_dims: usize = lenses.iter().map(|l| l.dimensions.len()).sum();
    let status_n = |s: &str| findings.iter().filter(|f| f.status == s).count();
    let desc = format!(
        "JuryPersona · {subject}: {n_personas} personas ({}), scoring {n_dims} dimensions across {} lens{}, and {} findings: {} open, {} fixed, {} dismissed, {} row. Edges from a persona to a dimension are coloured by the score from 1 to 5; edges to a finding are coloured by its decision. Each dimension shows a median badge per stratum, never one number across strata.",
        STAKES.iter().zip(&counts).map(|(s, n)| format!("{} {n}", s.name())).collect::<Vec<_>>().join(", "),
        lenses.len(),
        if lenses.len() == 1 { "" } else { "es" },
        findings.len(),
        status_n("open"),
        status_n("fixed"),
        status_n("dismissed"),
        status_n("row"),
    );

    let mut o = String::new();
    let w = lay::WIDTH;
    let _ = writeln!(o, "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {w} {height}\" width=\"{w}\" height=\"{height}\" role=\"img\" aria-labelledby=\"t d\">");
    let _ = writeln!(o, "<title id=\"t\">JuryPersona · {}</title><desc id=\"d\">{}</desc>", esc(&subject), esc(&desc));
    let _ = writeln!(o, "<rect width=\"100%\" height=\"100%\" fill=\"#ffffff\"/>");
    let _ = writeln!(o, "<rect x=\"1\" y=\"1\" width=\"{}\" height=\"{}\" rx=\"10\" fill=\"#ffffff\" stroke=\"{LINE}\" stroke-width=\"1.2\"/>", w - 2, height - 2);
    let _ = writeln!(o, "<g font-family=\"{FONT}\" fill=\"{INK}\">");
    let _ = writeln!(o, "<text x=\"30\" y=\"40\" font-size=\"18\" font-weight=\"700\">JuryPersona · {}</text>", esc(&lay::fit(&subject, 760, 18)));
    let _ = writeln!(o, "<text x=\"30\" y=\"62\" font-size=\"13\" fill=\"{MUTED}\">Personas by stake, the dimensions they scored, the blockers they raised. Edge colour is the score; badges are medians per stratum, never one number.</text>");
    let rounds = format!(
        "run{} {}",
        if runs.len() == 1 { "" } else { "s" },
        runs.iter().map(|r| r.id.to_string()).collect::<Vec<_>>().join(", ")
    );
    let _ = writeln!(o, "<text x=\"{}\" y=\"40\" font-size=\"13\" text-anchor=\"end\" fill=\"{MUTED}\">{}</text>", w - 30, esc(&rounds));

    // column headers
    for (s, stake) in STAKES.iter().enumerate() {
        let x = lay::MARGIN + s as i32 * (lay::PERSONA_W + lay::STRATUM_GAP);
        let hy = col_top[s] - lay::TITLE_H;
        let pw = lay::text_px(stake.name(), 14) + 24;
        let _ = writeln!(o, "<rect x=\"{x}\" y=\"{hy}\" width=\"{pw}\" height=\"26\" rx=\"13\" fill=\"{}\" stroke=\"{MUTED}\" stroke-width=\"1\"/>", tint(*stake));
        let _ = writeln!(o, "<text x=\"{}\" y=\"{}\" font-size=\"14\" font-weight=\"700\">{}</text>", x + 12, hy + 18, stake.name());
        let _ = writeln!(o, "<text x=\"{x}\" y=\"{}\" font-size=\"12\" fill=\"{MUTED}\">{} \u{b7} {}</text>", hy + 44, meaning(*stake), counts[s]);
    }
    for (lane, (lx, lw)) in lane_xw.iter().enumerate() {
        // the first lens of each lane heads the lane; later lenses are rows in the flow
        let mut heads: Vec<(usize, i32)> = vec![(lane, HEAD_Y + 18)];
        for (k, slot) in lane_slots[lane].iter().enumerate() {
            if let (i, None) = slot {
                heads.push((*i, TOP + lane_flows[lane].0[k] + 16));
            }
        }
        for (i, y) in heads {
            let _ = writeln!(o, "<text x=\"{lx}\" y=\"{y}\" font-size=\"13\" font-weight=\"700\" font-family=\"{MONO}\" fill=\"{BLUE}\">{}</text>", esc(&lenses[i].name));
            let _ = writeln!(o, "<line x1=\"{lx}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{MUTED}\" stroke-width=\"1.2\"/>", y + 8, lx + lw, y + 8);
            for (s, stake) in STAKES.iter().enumerate() {
                let bx = lx + lw - 6 - lay::BADGES_W + s as i32 * (lay::BADGE_W + lay::BADGE_GAP);
                let _ = writeln!(o, "<text x=\"{}\" y=\"{}\" font-size=\"9\" text-anchor=\"middle\" fill=\"{MUTED}\"><title>{}</title>{}</text>", bx + lay::BADGE_W / 2, y - 2, stake.name(), stake.name()[..1].to_uppercase());
            }
        }
    }
    let _ = writeln!(o, "<text x=\"{find_x}\" y=\"{}\" font-size=\"14\" font-weight=\"700\">findings</text>", HEAD_Y + 18);
    let _ = writeln!(o, "<line x1=\"{find_x}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{MUTED}\" stroke-width=\"1.2\"/>", HEAD_Y + 26, find_x + lay::FINDING_W, HEAD_Y + 26);
    let _ = writeln!(o, "<text x=\"{find_x}\" y=\"{}\" font-size=\"12\" fill=\"{MUTED}\">{} blocker{} raised</text>", HEAD_Y + 46, findings.len(), if findings.len() == 1 { "" } else { "s" });

    // ---- edges (behind the nodes) ----
    let all_nodes: Vec<&Node> = nodes_by_lens.iter().flatten().collect();
    let persona_at: BTreeMap<&str, &PersonaBox> = by_stake.iter().flatten().map(|p| (p.id.as_str(), p)).collect();
    for (lens, v) in &line_rows {
        let Some(p) = persona_at.get(v.persona.as_str()) else { continue };
        let Some(n) = all_nodes.iter().find(|n| n.lens == *lens && n.dim == v.dimension && n.about == v.about) else { continue };
        let (x1, y1) = (p.x + lay::PERSONA_W, p.y + lay::PERSONA_H / 2);
        let (x2, y2) = n.left();
        let key = n.key();
        let _ = writeln!(
            o,
            "<path class=\"edge-score\" data-persona=\"{}\" data-node=\"{}\" data-score=\"{}\" d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\" stroke-opacity=\"0.8\"><title>{} scored {} {}</title></path>",
            esc(&p.id), esc(&key), v.score, lay::edge_path(x1, y1, x2, y2), step(v.score), v.score.clamp(1, 5) * 4 / 10 + 1, esc(&p.id), esc(&key), v.score
        );
    }
    for (i, f) in findings.iter().enumerate() {
        let Some(p) = persona_at.get(f.finding.persona.as_str()) else { continue };
        let (x1, y1) = (p.x + lay::PERSONA_W, p.y + lay::PERSONA_H / 2);
        let (x2, y2) = (find_x, find_y(i) + lay::FINDING_H / 2);
        let (col, _) = status_colours(f.status);
        let dash = if f.status == "dismissed" { " stroke-dasharray=\"4 3\"" } else { "" };
        let _ = writeln!(
            o,
            "<path class=\"edge-finding\" data-finding=\"{}\" data-status=\"{}\" d=\"{}\" fill=\"none\" stroke=\"{col}\" stroke-width=\"1.6\"{dash} stroke-opacity=\"0.85\"><title>{} raised {} ({})</title></path>",
            f.finding.id, f.status, lay::edge_path(x1, y1, x2, y2), esc(&p.id), &f.finding.id[..6], f.status
        );
    }

    // ---- nodes ----
    for p in by_stake.iter().flatten() {
        let _ = writeln!(o, "<g>");
        let _ = writeln!(o, "<title>{} ({}, {})</title>", esc(&p.name), esc(&p.id), p.stake.name());
        let _ = writeln!(o, "<rect class=\"persona\" data-persona=\"{}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"7\" fill=\"#ffffff\" stroke=\"{LINE}\" stroke-width=\"1.2\"/>", esc(&p.id), p.x, p.y, lay::PERSONA_W, lay::PERSONA_H);
        let _ = writeln!(o, "<text x=\"{}\" y=\"{}\" font-size=\"12\" font-weight=\"700\">{}</text>", p.x + 10, p.y + 17, esc(&lay::fit(&p.name, lay::PERSONA_W - 20, 12)));
        let pill_w = lay::text_px(p.stake.name(), 9) + 12;
        let _ = writeln!(o, "<text x=\"{}\" y=\"{}\" font-size=\"10\" font-family=\"{MONO}\" fill=\"{MUTED}\">{}</text>", p.x + 10, p.y + 31, esc(&lay::fit(&p.id, lay::PERSONA_W - 22 - pill_w, 10)));
        let _ = writeln!(o, "<rect x=\"{}\" y=\"{}\" width=\"{pill_w}\" height=\"13\" rx=\"6.5\" fill=\"{}\" stroke=\"{MUTED}\" stroke-width=\"0.8\"/>", p.x + lay::PERSONA_W - 8 - pill_w, p.y + 21, tint(p.stake));
        let _ = writeln!(o, "<text x=\"{}\" y=\"{}\" font-size=\"9\" text-anchor=\"middle\">{}</text>", p.x + lay::PERSONA_W - 8 - pill_w / 2, p.y + 31, p.stake.name());
        let _ = writeln!(o, "</g>");
    }
    for n in &all_nodes {
        let ind = if n.about.is_some() { lay::SUB_INDENT } else { 0 };
        let (x, w) = (n.x + ind, n.w - ind);
        let sub = n.about.is_some();
        let key = n.key();
        let _ = writeln!(o, "<g>");
        let _ = writeln!(o, "<title>{}</title>", esc(&key));
        let _ = writeln!(o, "<rect class=\"node\" data-node=\"{}\" x=\"{x}\" y=\"{}\" width=\"{w}\" height=\"{}\" rx=\"5\" fill=\"{}\" stroke=\"{LINE}\" stroke-width=\"1\"/>", esc(&key), n.y, n.h, if sub { "#f6f8fa" } else { "#ffffff" });
        let size = if sub { 10 } else { 11 };
        let label = lay::fit(&n.label, w - 8 - lay::BADGES_W - 10, size);
        let _ = writeln!(o, "<text x=\"{}\" y=\"{}\" font-size=\"{size}\"{}{}>{}</text>", x + 8, n.y + n.h / 2 + 4, if sub { "" } else { " font-weight=\"700\"" }, if sub { format!(" fill=\"{MUTED}\"") } else { String::new() }, esc(&label));
        for (s, stake) in STAKES.iter().enumerate() {
            let bx = n.x + n.w - 6 - lay::BADGES_W + s as i32 * (lay::BADGE_W + lay::BADGE_GAP);
            let by = n.y + (n.h - lay::BADGE_H) / 2;
            match n.medians[s] {
                Some((m, count)) => {
                    let _ = writeln!(o, "<rect class=\"badge\" data-node=\"{}\" data-stratum=\"{}\" data-median=\"{m}\" x=\"{bx}\" y=\"{by}\" width=\"{}\" height=\"{}\" rx=\"4\" fill=\"{}\"><title>{} median {m} of {count}</title></rect>", esc(&key), stake.name(), lay::BADGE_W, lay::BADGE_H, step(m), stake.name());
                    let _ = writeln!(o, "<text x=\"{}\" y=\"{}\" font-size=\"10\" font-weight=\"700\" text-anchor=\"middle\" fill=\"{}\">{m}</text>", bx + lay::BADGE_W / 2, by + 11, if m >= 4 { "#ffffff" } else { INK });
                }
                None => {
                    let _ = writeln!(o, "<rect class=\"badge\" data-node=\"{}\" data-stratum=\"{}\" data-median=\"none\" x=\"{bx}\" y=\"{by}\" width=\"{}\" height=\"{}\" rx=\"4\" fill=\"none\" stroke=\"{LINE}\" stroke-width=\"1\"><title>{}: no score</title></rect>", esc(&key), stake.name(), lay::BADGE_W, lay::BADGE_H, stake.name());
                }
            }
        }
        let _ = writeln!(o, "</g>");
    }
    let strip_w = if multi { findings.iter().map(|f| f.states.len().min(MAX_DOTS) as i32).max().unwrap_or(0) * 12 + 4 + if findings.iter().any(|f| f.states.len() > MAX_DOTS) { 22 } else { 0 } } else { 0 };
    for (i, f) in findings.iter().enumerate() {
        let y = find_y(i);
        let (col, fill) = status_colours(f.status);
        let dash = if f.status == "dismissed" { " stroke-dasharray=\"4 3\"" } else { "" };
        let short = &f.finding.id[..6];
        let text = lay::fit(&f.finding.text, lay::FINDING_W - 12 - 38 - 6 - strip_w, 10);
        let _ = writeln!(o, "<g>");
        let _ = writeln!(o, "<title>{} ({}) {} on {}{}: {}</title>", short, f.status, esc(&f.finding.persona), esc(&f.finding.dimension), f.finding.about.as_ref().map_or(String::new(), |a| format!("@{}", esc(a))), esc(&f.finding.text));
        let _ = writeln!(o, "<rect class=\"finding\" data-finding=\"{}\" data-status=\"{}\" x=\"{find_x}\" y=\"{y}\" width=\"{}\" height=\"{}\" rx=\"5\" fill=\"{fill}\" stroke=\"{col}\" stroke-width=\"1.2\"{dash}/>", f.finding.id, f.status, lay::FINDING_W, lay::FINDING_H);
        let _ = writeln!(o, "<text x=\"{}\" y=\"{}\" font-size=\"9\" font-family=\"{MONO}\" fill=\"{MUTED}\">{short}</text>", find_x + 8, y + 16);
        let _ = writeln!(o, "<text x=\"{}\" y=\"{}\" font-size=\"10\">{}</text>", find_x + 8 + 38, y + 16, esc(&text));
        if multi {
            // the finding's own rounds, the latest at the right edge
            let hidden = f.states.len().saturating_sub(MAX_DOTS);
            let shown = &f.states[hidden..];
            let n = shown.len() as i32;
            if hidden > 0 {
                let mx = find_x + lay::FINDING_W - 8 - 6 - (n - 1) * 12 - 10;
                let _ = writeln!(o, "<text x=\"{mx}\" y=\"{}\" font-size=\"8\" text-anchor=\"end\" fill=\"{MUTED}\"><title>{hidden} earlier runs not drawn</title>+{hidden}</text>", y + 15);
            }
            for (k, (run, st)) in shown.iter().enumerate() {
                let cx = find_x + lay::FINDING_W - 8 - 6 - (n - 1 - k as i32) * 12;
                let cy = y + lay::FINDING_H / 2;
                let (attrs, what) = match st {
                    State::Raised => (format!("fill=\"{RED}\" stroke=\"{RED}\""), "raised"),
                    State::RaisedAgain => (format!("fill=\"{RED}\" stroke=\"#82071e\" stroke-width=\"2\""), "raised again after a fix"),
                    State::Fixed => (format!("fill=\"{GREEN}\" stroke=\"{GREEN}\""), "absent after a fix"),
                    State::Unjudged => (format!("fill=\"none\" stroke=\"{GREEN}\" stroke-dasharray=\"2 2\""), "absent after a fix, but the persona did not judge"),
                    State::Absent => (format!("fill=\"none\" stroke=\"{GREY}\""), "absent"),
                };
                let _ = writeln!(o, "<circle class=\"life\" data-finding=\"{}\" data-run=\"{run}\" data-state=\"{}\" cx=\"{cx}\" cy=\"{cy}\" r=\"4\" {attrs}><title>run {run}: {what}</title></circle>", f.finding.id, st.name());
            }
        }
        let _ = writeln!(o, "</g>");
    }

    // ---- legend, under the stairs ----
    let lx = lay::MARGIN;
    let ly = legend_y;
    let _ = writeln!(o, "<g font-size=\"12\" fill=\"{MUTED}\">");
    let _ = writeln!(o, "<text x=\"{lx}\" y=\"{ly}\" font-size=\"13\" font-weight=\"700\" fill=\"{INK}\">Reading the graph</text>");
    let cy = ly + 30;
    for (i, c) in STEP.iter().enumerate() {
        let _ = writeln!(o, "<rect x=\"{}\" y=\"{}\" width=\"30\" height=\"18\" rx=\"3\" fill=\"{c}\"/>", lx + i as i32 * 32, cy - 12);
        let _ = writeln!(o, "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" fill=\"{}\">{}</text>", lx + i as i32 * 32 + 15, cy + 2, if i >= 3 { "#ffffff" } else { INK }, i + 1);
    }
    let _ = writeln!(o, "<text x=\"{}\" y=\"{}\">score: the colour and weight of an edge</text>", lx + 5 * 32 + 8, cy + 2);
    let _ = writeln!(o, "<text x=\"{lx}\" y=\"{}\">Badges P S B: the median of the primary, secondary</text>", cy + 28);
    let _ = writeln!(o, "<text x=\"{lx}\" y=\"{}\">and bystander personas. Never one number across them.</text>", cy + 46);
    let fy = cy + 76;
    let mut x = lx;
    for status in ["open", "fixed", "dismissed", "row"] {
        let (col, fill) = status_colours(status);
        let dash = if status == "dismissed" { " stroke-dasharray=\"4 3\"" } else { "" };
        let _ = writeln!(o, "<rect x=\"{x}\" y=\"{}\" width=\"22\" height=\"14\" rx=\"3\" fill=\"{fill}\" stroke=\"{col}\" stroke-width=\"1.2\"{dash}/>", fy - 10);
        let _ = writeln!(o, "<text x=\"{}\" y=\"{}\">{status}</text>", x + 28, fy + 1);
        x += 28 + lay::text_px(status, 12) + 18;
    }
    if multi {
        let ty = fy + 54;
        let _ = writeln!(o, "<text x=\"{lx}\" y=\"{}\">One dot per run of the lens, oldest first:</text>", ty - 16);
        let mut x = lx;
        for (label, attrs) in [
            ("raised", format!("fill=\"{RED}\" stroke=\"{RED}\"")),
            ("raised again", format!("fill=\"{RED}\" stroke=\"#82071e\" stroke-width=\"2\"")),
            ("fixed", format!("fill=\"{GREEN}\" stroke=\"{GREEN}\"")),
            ("not judged", format!("fill=\"none\" stroke=\"{GREEN}\" stroke-dasharray=\"2 2\"")),
            ("absent", format!("fill=\"none\" stroke=\"{GREY}\"")),
        ] {
            let _ = writeln!(o, "<circle cx=\"{}\" cy=\"{}\" r=\"4\" {attrs}/>", x + 4, ty - 3);
            let _ = writeln!(o, "<text x=\"{}\" y=\"{ty}\">{label}</text>", x + 14);
            x += 14 + lay::text_px(label, 12) + 18;
        }
    }
    let _ = writeln!(o, "</g>");
    let _ = writeln!(o, "</g>");
    let _ = writeln!(o, "</svg>");
    Ok(o)
}

/// The verb: the SVG text, or a file when `out` is given.
pub fn show(root: &Path, runs: &[i64], svg: bool, out: Option<&Path>) -> Result<Value> {
    if !svg {
        return Err(AppError::InvalidArguments("graph draws an SVG: pass --svg".into()));
    }
    let text = render(root, runs)?;
    let mut ids: Vec<i64> = runs.to_vec();
    ids.sort_unstable();
    ids.dedup();
    match out {
        Some(p) => {
            std::fs::write(p, text.as_bytes()).map_err(|e| AppError::Io(p.to_path_buf(), e))?;
            Ok(json!({"runs": ids, "bytes": text.len(), "path": p}))
        }
        None => Ok(json!({"runs": ids, "bytes": text.len(), "svg": text})),
    }
}

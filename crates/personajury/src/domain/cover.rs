//! Panel cover (D30): the cover-aware draw and the coverage counts. Pure. Every figure is a count of seats; nothing is a
//! score, nothing is averaged, and the strata are kept apart (one-way cells per stratum, pairs panel-wide).
use super::model::Persona;
use super::traits::{are_correlated, TraitDef};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

/// One trait of a panel's cover, with its definition.
pub struct CoverTrait<'a> {
    pub id: &'a str,
    pub def: &'a TraitDef,
}

fn held(p: &Persona, c: &CoverTrait) -> Vec<String> {
    p.traits
        .as_ref()
        .and_then(|m| m.get(c.id))
        .map(|v| c.def.bands_held(v))
        .unwrap_or_default()
}

/// The cover cells already seated: one-way (stratum, trait, band) and panel-wide pairs (trait i, band, trait j, band).
#[derive(Default)]
pub struct State {
    oneway: BTreeSet<(String, usize, String)>,
    pairs: BTreeSet<(usize, String, usize, String)>,
}
impl State {
    fn gain(&self, cover: &[CoverTrait], correlated: &BTreeSet<(usize, usize)>, p: &Persona, stratum: &str) -> (usize, usize) {
        let bands: Vec<Vec<String>> = cover.iter().map(|c| held(p, c)).collect();
        let mut one = 0;
        for (i, bs) in bands.iter().enumerate() {
            one += bs.iter().filter(|b| !self.oneway.contains(&(stratum.to_string(), i, (*b).clone()))).count();
        }
        let mut pairs = 0;
        for i in 0..cover.len() {
            for j in i + 1..cover.len() {
                if correlated.contains(&(i, j)) {
                    continue;
                }
                for a in &bands[i] {
                    for b in &bands[j] {
                        if !self.pairs.contains(&(i, a.clone(), j, b.clone())) {
                            pairs += 1;
                        }
                    }
                }
            }
        }
        (one, pairs)
    }
    fn add(&mut self, cover: &[CoverTrait], p: &Persona, stratum: &str) {
        let bands: Vec<Vec<String>> = cover.iter().map(|c| held(p, c)).collect();
        for (i, bs) in bands.iter().enumerate() {
            for b in bs {
                self.oneway.insert((stratum.to_string(), i, b.clone()));
            }
        }
        for i in 0..cover.len() {
            for j in i + 1..cover.len() {
                for a in &bands[i] {
                    for b in &bands[j] {
                        self.pairs.insert((i, a.clone(), j, b.clone()));
                    }
                }
            }
        }
    }
}

/// The index pairs of cover traits that either lists as correlated.
pub fn correlated_indexes(cover: &[CoverTrait]) -> BTreeSet<(usize, usize)> {
    let defs: BTreeMap<String, TraitDef> = cover.iter().map(|c| (c.id.to_string(), c.def.clone())).collect();
    let mut out = BTreeSet::new();
    for i in 0..cover.len() {
        for j in i + 1..cover.len() {
            if are_correlated(&defs, cover[i].id, cover[j].id) {
                out.insert((i, j));
            }
        }
    }
    out
}

/// Choose `count` seats of one term from `order` (the term's pool in its seeded order, the tie-breaker): each seat is the
/// unchosen persona that adds the most new one-way cells for the term's stratum, then the most new panel-wide pairs;
/// ties go to the earliest in `order`. `state` carries across terms.
pub fn choose<'a>(
    cover: &[CoverTrait],
    state: &mut State,
    order: &[&'a Persona],
    count: usize,
    stratum: &str,
) -> Vec<&'a Persona> {
    let correlated = correlated_indexes(cover);
    let mut chosen: Vec<&Persona> = Vec::new();
    let mut used = vec![false; order.len()];
    while chosen.len() < count && chosen.len() < order.len() {
        let mut best: Option<(usize, (usize, usize))> = None;
        for (i, p) in order.iter().enumerate() {
            if used[i] {
                continue;
            }
            let g = state.gain(cover, &correlated, p, stratum);
            if best.is_none_or(|(_, bg)| g > bg) {
                best = Some((i, g));
            }
        }
        let Some((i, _)) = best else { break };
        used[i] = true;
        state.add(cover, order[i], stratum);
        chosen.push(order[i]);
    }
    chosen
}

fn partition_key(p: &Persona, c: &CoverTrait) -> String {
    let h = held(p, c);
    if h.is_empty() {
        "unset".into()
    } else {
        h.join("+")
    }
}

/// The coverage of a seated panel: `seated` is each seat with the stratum word it sits in (`primary`, `secondary`,
/// `bystander`, or `unstaked`).
pub fn coverage(cover: &[CoverTrait], seated: &[(&Persona, String)]) -> Value {
    let correlated = correlated_indexes(cover);
    let mut strata: Vec<String> = Vec::new();
    for w in ["primary", "secondary", "bystander", "unstaked"] {
        if seated.iter().any(|(_, s)| s == w) {
            strata.push(w.to_string());
        }
    }
    let mut one_way = Vec::new();
    let mut balance = Vec::new();
    for c in cover {
        for st in &strata {
            let here: Vec<&Persona> = seated.iter().filter(|(_, s)| s == st).map(|(p, _)| *p).collect();
            let mut counts = serde_json::Map::new();
            let mut missing = Vec::new();
            for b in c.def.bands() {
                let n = here.iter().filter(|p| held(p, c).contains(&b)).count();
                if n == 0 {
                    missing.push(b.clone());
                }
                counts.insert(b, json!(n));
            }
            counts.insert("unset".into(), json!(here.iter().filter(|p| held(p, c).is_empty()).count()));
            one_way.push(json!({"trait": c.id, "shape": c.def.shape, "stratum": st, "seated": Value::Object(counts)}));
            if !missing.is_empty() {
                balance.push(json!({"trait": c.id, "stratum": st, "missing": missing}));
            }
        }
    }
    let mut total = 0usize;
    let mut covered = 0usize;
    let mut uncovered = Vec::new();
    let mut excluded = Vec::new();
    for i in 0..cover.len() {
        for j in i + 1..cover.len() {
            if correlated.contains(&(i, j)) {
                excluded.push(json!([cover[i].id, cover[j].id]));
                continue;
            }
            for a in cover[i].def.bands() {
                for b in cover[j].def.bands() {
                    total += 1;
                    let hit = seated.iter().any(|(p, _)| held(p, &cover[i]).contains(&a) && held(p, &cover[j]).contains(&b));
                    if hit {
                        covered += 1;
                    } else {
                        uncovered.push(json!([cover[i].id, a, cover[j].id, b]));
                    }
                }
            }
        }
    }
    // two cover traits that split the seated personas into the same groups say the same thing twice
    let mut collinear = Vec::new();
    for i in 0..cover.len() {
        for j in i + 1..cover.len() {
            let ki: Vec<String> = seated.iter().map(|(p, _)| partition_key(p, &cover[i])).collect();
            let kj: Vec<String> = seated.iter().map(|(p, _)| partition_key(p, &cover[j])).collect();
            let blocks = ki.iter().collect::<BTreeSet<_>>().len();
            let same = (0..ki.len()).all(|a| (0..ki.len()).all(|b| (ki[a] == ki[b]) == (kj[a] == kj[b])));
            if blocks >= 2 && same {
                collinear.push(json!([cover[i].id, cover[j].id]));
            }
        }
    }
    // a band every seated persona of a tag shares (two or more) while a seated persona without the tag differs
    let tags: BTreeSet<&String> = seated.iter().flat_map(|(p, _)| p.tags.iter()).collect();
    let mut confounded = Vec::new();
    for c in cover {
        for tag in &tags {
            let with: Vec<&Persona> = seated.iter().filter(|(p, _)| p.tags.contains(tag)).map(|(p, _)| *p).collect();
            if with.len() < 2 {
                continue;
            }
            let keys: BTreeSet<String> = with.iter().map(|p| partition_key(p, c)).collect();
            if keys.len() != 1 || keys.contains("unset") {
                continue;
            }
            let band = keys.into_iter().next().unwrap_or_default();
            let differs = seated
                .iter()
                .filter(|(p, _)| !p.tags.contains(tag))
                .any(|(p, _)| {
                    let k = partition_key(p, c);
                    k != "unset" && k != band
                });
            if differs {
                confounded.push(json!({"trait": c.id, "tag": tag, "band": band}));
            }
        }
    }
    json!({
        "cover": cover.iter().map(|c| c.id).collect::<Vec<_>>(),
        "one_way": one_way,
        "pairs": {"scope": "panel", "covered": covered, "total": total, "uncovered": uncovered, "excluded_correlated": excluded},
        "balance": balance,
        "collinear": collinear,
        "confounded": confounded,
    })
}

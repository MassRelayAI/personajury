//! Calibration (D20): does the jury rank things the way a person does? Per stratum, the cells (lens, dimension, about)
//! are ranked by the stratum's median, high to low, ties sharing a rank; the jury's order is set beside the humans'. A
//! cell is `within_one_rank` when the two ranks differ by at most one. There is no pooled number: each stratum answers
//! for itself, and `calibrated` is the primary stratum's rule (the held-out detector's): every primary cell within one.
use super::model::{ScoreRow, Stake};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

type Key = (String, String, Option<String>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CellCal {
    pub lens: String,
    pub dimension: String,
    pub about: Option<String>,
    pub jury_median: i64,
    pub human_median: i64,
    pub jury_rank: i64,
    pub human_rank: i64,
    pub within_one_rank: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Uncompared {
    pub lens: String,
    pub dimension: String,
    pub about: Option<String>,
    /// which side lacks it: `human` (the jury judged it, the people did not) or `jury`
    pub missing: &'static str,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StratumCal {
    pub stake: Stake,
    pub cells: Vec<CellCal>,
    pub uncompared: Vec<Uncompared>,
}

/// The lower middle of the scores (the engine's median).
fn median(v: &mut [i64]) -> i64 {
    v.sort_unstable();
    v[(v.len() - 1) / 2]
}
fn medians(rows: &[ScoreRow], stakes: &BTreeMap<String, Stake>) -> BTreeMap<Stake, BTreeMap<Key, i64>> {
    let mut g: BTreeMap<Stake, BTreeMap<Key, Vec<i64>>> = BTreeMap::new();
    for r in rows {
        if let Some(s) = stakes.get(&r.persona) {
            g.entry(*s).or_default().entry((r.lens.clone(), r.dimension.clone(), r.about.clone())).or_default().push(r.score);
        }
    }
    g.into_iter().map(|(s, m)| (s, m.into_iter().map(|(k, mut v)| (k, median(&mut v))).collect())).collect()
}
/// Competition ranking, high median first: 1, 1, 3.
fn ranks(m: &BTreeMap<Key, i64>) -> BTreeMap<Key, i64> {
    m.iter().map(|(k, v)| (k.clone(), 1 + m.values().filter(|o| *o > v).count() as i64)).collect()
}

/// `human` rows carry the seat's persona id (the person speaks in that seat's stratum).
pub fn compare(
    jury: &[ScoreRow],
    jury_stakes: &BTreeMap<String, Stake>,
    human: &[ScoreRow],
    human_stakes: &BTreeMap<String, Stake>,
) -> Vec<StratumCal> {
    let (jm, hm) = (medians(jury, jury_stakes), medians(human, human_stakes));
    let mut out = Vec::new();
    for stake in [Stake::Primary, Stake::Secondary, Stake::Bystander] {
        let (j, h) = (jm.get(&stake), hm.get(&stake));
        if j.is_none() && h.is_none() {
            continue;
        }
        let empty = BTreeMap::new();
        let (j, h) = (j.unwrap_or(&empty), h.unwrap_or(&empty));
        let both: BTreeMap<Key, i64> = j.iter().filter(|(k, _)| h.contains_key(*k)).map(|(k, v)| (k.clone(), *v)).collect();
        let both_h: BTreeMap<Key, i64> = h.iter().filter(|(k, _)| j.contains_key(*k)).map(|(k, v)| (k.clone(), *v)).collect();
        let (jr, hr) = (ranks(&both), ranks(&both_h));
        let cells = both
            .iter()
            .map(|(k, jm)| CellCal {
                lens: k.0.clone(),
                dimension: k.1.clone(),
                about: k.2.clone(),
                jury_median: *jm,
                human_median: both_h[k],
                jury_rank: jr[k],
                human_rank: hr[k],
                within_one_rank: (jr[k] - hr[k]).abs() <= 1,
            })
            .collect();
        let mut uncompared: Vec<Uncompared> = Vec::new();
        for k in j.keys().filter(|k| !h.contains_key(*k)) {
            uncompared.push(Uncompared { lens: k.0.clone(), dimension: k.1.clone(), about: k.2.clone(), missing: "human" });
        }
        for k in h.keys().filter(|k| !j.contains_key(*k)) {
            uncompared.push(Uncompared { lens: k.0.clone(), dimension: k.1.clone(), about: k.2.clone(), missing: "jury" });
        }
        out.push(StratumCal { stake, cells, uncompared });
    }
    out
}
/// Every primary cell the jury judged is compared and within one rank; no primary stratum, or none compared, is not proven.
pub fn calibrated(strata: &[StratumCal]) -> bool {
    strata
        .iter()
        .find(|s| s.stake == Stake::Primary)
        .is_some_and(|s| !s.cells.is_empty() && s.cells.iter().all(|c| c.within_one_rank) && !s.uncompared.iter().any(|u| u.missing == "human"))
}


// ---- a trait slice of the calibration (D36): does the trait move the jury as it moves people? ----

/// The jurors (or the people) holding one band or option in one stratum, with their rows.
#[derive(Debug, Clone, Default)]
pub struct Side {
    pub members: Vec<String>,
    pub rows: Vec<ScoreRow>,
}
/// One band in a stratum: the two sides, and the D20 rank comparison of its cells.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SliceGroup {
    pub band: String,
    pub jury_n: usize,
    pub people_n: usize,
    pub jury: Vec<String>,
    pub people: Vec<String>,
    pub cells: Vec<CellCal>,
    pub uncompared: Vec<Uncompared>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Separates {
    pub cells: usize,
    pub total: usize,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SliceCal {
    #[serde(rename = "trait")]
    pub trait_id: String,
    pub stratum: Stake,
    pub groups: Vec<SliceGroup>,
    pub jury_unset: Vec<String>,
    pub people_unset: Vec<String>,
    pub separates_in_jury: Separates,
    pub separates_in_people: Separates,
    /// the cells on which both sides separate
    pub separates_in_both: usize,
    pub word: &'static str,
}

/// The cells on which two bands' `anchor_met` ranges do not overlap (the split rule of D13 applied to bands).
fn separated(sides: &[&Side]) -> BTreeSet<Key> {
    let mut per: BTreeMap<Key, Vec<(i64, i64)>> = BTreeMap::new();
    for side in sides.iter().filter(|s| !s.members.is_empty()) {
        let mut cell: BTreeMap<Key, (i64, i64)> = BTreeMap::new();
        for r in &side.rows {
            let e = cell.entry((r.lens.clone(), r.dimension.clone(), r.about.clone())).or_insert((r.anchor_met, r.anchor_met));
            e.0 = e.0.min(r.anchor_met);
            e.1 = e.1.max(r.anchor_met);
        }
        for (k, v) in cell {
            per.entry(k).or_default().push(v);
        }
    }
    per.into_iter().filter(|(_, rs)| rs.iter().enumerate().any(|(i, a)| rs[i + 1..].iter().any(|b| a.1 < b.0 || b.1 < a.0))).map(|(k, _)| k).collect()
}
fn cells_of(rows: &[ScoreRow]) -> usize {
    rows.iter().map(|r| (&r.lens, &r.dimension, &r.about)).collect::<BTreeSet<_>>().len()
}

/// One stratum of `calibrate --slice`: `bands` in the trait's order, each with the jury's and the people's holders;
/// `stakes` gives every member (a persona id, a `human:<name>:<seat>` id) its stratum, `jury_total` and `people_total` the
/// cells of the whole stratum on each side. The word: `thin` when any band has fewer than two on either side or fewer
/// than two bands are held at all (nothing can separate); else `predicts` when both sides separate on a common cell,
/// `missed` when the people separate and the jury does not, `inert` when neither does, and `played` otherwise (the jury
/// separates and the people do not, or on other cells than the people).
#[allow(clippy::too_many_arguments)]
pub fn slice(
    trait_id: &str,
    stake: Stake,
    bands: &[(String, Side, Side)],
    stakes: &BTreeMap<String, Stake>,
    jury_unset: Vec<String>,
    people_unset: Vec<String>,
    jury_total: usize,
    people_total: usize,
) -> SliceCal {
    let mut groups = Vec::new();
    let held: Vec<&(String, Side, Side)> = bands.iter().filter(|(_, j, p)| !j.members.is_empty() || !p.members.is_empty()).collect();
    for (band, j, p) in &held {
        let cal = compare(&j.rows, stakes, &p.rows, stakes).into_iter().find(|s| s.stake == stake);
        let (cells, uncompared) = cal.map(|c| (c.cells, c.uncompared)).unwrap_or_default();
        groups.push(SliceGroup {
            band: band.clone(),
            jury_n: j.members.len(),
            people_n: p.members.len(),
            jury: j.members.clone(),
            people: p.members.clone(),
            cells,
            uncompared,
        });
    }
    let sj = separated(&held.iter().map(|(_, j, _)| j).collect::<Vec<_>>());
    let sp = separated(&held.iter().map(|(_, _, p)| p).collect::<Vec<_>>());
    let both = sj.intersection(&sp).count();
    let thin = held.len() < 2 || held.iter().any(|(_, j, p)| j.members.len() < 2 || p.members.len() < 2);
    let word = if thin {
        "thin"
    } else if both > 0 {
        "predicts"
    } else if sp.is_empty() && sj.is_empty() {
        "inert"
    } else if sj.is_empty() {
        "missed"
    } else {
        "played"
    };
    SliceCal {
        trait_id: trait_id.to_string(),
        stratum: stake,
        groups,
        jury_unset,
        people_unset,
        separates_in_jury: Separates { cells: sj.len(), total: jury_total },
        separates_in_people: Separates { cells: sp.len(), total: people_total },
        separates_in_both: both,
        word,
    }
}
/// The cells of a stratum on one side, for `total`.
pub fn total_cells(rows: &[ScoreRow]) -> usize {
    cells_of(rows)
}

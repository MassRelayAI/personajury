//! Calibration (D20): does the jury rank things the way a person does? Per stratum, the cells (lens, dimension, about)
//! are ranked by the stratum's median, high to low, ties sharing a rank; the jury's order is set beside the humans'. A
//! cell is `within_one_rank` when the two ranks differ by at most one. There is no pooled number: each stratum answers
//! for itself, and `calibrated` is the primary stratum's rule (the held-out detector's): every primary cell within one.
use super::model::{ScoreRow, Stake};
use serde::Serialize;
use std::collections::BTreeMap;

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

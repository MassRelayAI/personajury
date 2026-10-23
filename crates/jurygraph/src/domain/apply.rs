//! The split rule and the apply verdict's vocabulary (D13). Pure: a split is two strata that disagree on a cell, found
//! by comparing the ranges of `anchor_met`; nothing here averages across strata.
use super::model::{ScoreRow, Stake};
use serde::Serialize;
use std::collections::BTreeMap;

/// One stratum's `anchor_met` range on a cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct StratumRange {
    pub stake: Stake,
    pub min: i64,
    pub max: i64,
}

/// A (lens, dimension, about) cell on which two strata's `anchor_met` ranges do not overlap.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Split {
    pub lens: String,
    pub dimension: String,
    pub about: Option<String>,
    /// the pair, in stake order
    pub strata: [StratumRange; 2],
}

type CellKey = (String, String, Option<String>);

/// Every cell on which two strata disagree. Ranges that touch (one's max equals the other's min) overlap: no split.
/// A persona with no stake in `stakes` counts nowhere. Cells are ordered by lens, dimension, about.
pub fn splits(rows: &[ScoreRow], stakes: &BTreeMap<String, Stake>) -> Vec<Split> {
    let mut cells: BTreeMap<CellKey, BTreeMap<Stake, (i64, i64)>> = BTreeMap::new();
    for r in rows {
        let Some(stake) = stakes.get(&r.persona) else { continue };
        let range = cells
            .entry((r.lens.clone(), r.dimension.clone(), r.about.clone()))
            .or_default()
            .entry(*stake)
            .or_insert((r.anchor_met, r.anchor_met));
        range.0 = range.0.min(r.anchor_met);
        range.1 = range.1.max(r.anchor_met);
    }
    let mut out = Vec::new();
    for ((lens, dimension, about), by_stake) in cells {
        let ranges: Vec<StratumRange> = by_stake
            .into_iter()
            .map(|(stake, (min, max))| StratumRange { stake, min, max })
            .collect();
        for (i, a) in ranges.iter().enumerate() {
            for b in &ranges[i + 1..] {
                if a.max < b.min || b.max < a.min {
                    out.push(Split {
                        lens: lens.clone(),
                        dimension: dimension.clone(),
                        about: about.clone(),
                        strata: [*a, *b],
                    });
                }
            }
        }
    }
    out
}

/// What `apply-check` answers (D13, D17, D18). The agent obeys it; the engine applies nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Answer {
    Apply,
    Escalate,
    Human,
    Advise,
}
impl Answer {
    /// `human` beats `escalate` beats `advise` beats `apply`: a human class stays human, the next quorum up is asked
    /// before a cheap or weak jury merely advises.
    pub fn from_facts(human: bool, escalate: bool, advise: bool) -> Answer {
        if human {
            Answer::Human
        } else if escalate {
            Answer::Escalate
        } else if advise {
            Answer::Advise
        } else {
            Answer::Apply
        }
    }
}

/// One level up: low to high, high to human; none above human (and none for a quorum that is not low or high).
pub fn next_quorum(quorum: &str) -> Option<&'static str> {
    match quorum {
        "low" => Some("high"),
        "high" => Some("human"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::Evidence;

    fn row(persona: &str, dim: &str, anchor: i64) -> ScoreRow {
        ScoreRow {
            persona: persona.into(),
            lens: "L".into(),
            dimension: dim.into(),
            about: None,
            score: anchor,
            anchor_met: anchor,
            evidence: Evidence::Seen,
            citations: vec![],
            blockers: vec![],
        }
    }
    fn st() -> BTreeMap<String, Stake> {
        BTreeMap::from([
            ("a".to_string(), Stake::Primary),
            ("b".to_string(), Stake::Primary),
            ("c".to_string(), Stake::Secondary),
            ("d".to_string(), Stake::Bystander),
        ])
    }
    #[test]
    fn disjoint_ranges_split_and_touching_ranges_do_not() {
        let rows = [row("a", "x", 1), row("c", "x", 3)];
        assert_eq!(splits(&rows, &st()).len(), 1);
        let rows = [row("a", "x", 1), row("b", "x", 3), row("c", "x", 3)];
        assert!(splits(&rows, &st()).is_empty());
    }
    #[test]
    fn three_strata_give_one_split_per_disagreeing_pair() {
        let rows = [row("a", "x", 1), row("c", "x", 3), row("d", "x", 5)];
        assert_eq!(splits(&rows, &st()).len(), 3);
    }
    #[test]
    fn the_answer_precedence_is_human_escalate_advise_apply() {
        assert_eq!(Answer::from_facts(true, true, true), Answer::Human);
        assert_eq!(Answer::from_facts(false, true, true), Answer::Escalate);
        assert_eq!(Answer::from_facts(false, false, true), Answer::Advise);
        assert_eq!(Answer::from_facts(false, false, false), Answer::Apply);
        assert_eq!(next_quorum("low"), Some("high"));
        assert_eq!(next_quorum("high"), Some("human"));
        assert_eq!(next_quorum("human"), None);
    }
}

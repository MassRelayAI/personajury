//! Rounds diff (report §2.6, §10(2)): blockers down and no dimension down by more than one anchor, or it did not land.
//! Blockers are counted per stratum (I1) and a cell of the earlier run the later run lacks means no landing (C2).
use super::model::{BlockerCount, CellKey, Delta, Move, ScoreRow, Stake};
use std::collections::{BTreeMap, BTreeSet};

/// (persona, lens, dimension, about): a cell of the matrix, per persona (D6 A3)
type Key = (String, String, String, Option<String>);

fn index(rows: &[ScoreRow]) -> BTreeMap<Key, i64> {
    rows.iter()
        .map(|r| {
            (
                (
                    r.persona.clone(),
                    r.lens.clone(),
                    r.dimension.clone(),
                    r.about.clone(),
                ),
                r.score,
            )
        })
        .collect()
}

/// Blockers per stratum of the persona in its own run; a persona with no stake in the run counts nowhere.
fn by_stratum(rows: &[ScoreRow], stakes: &BTreeMap<String, Stake>) -> BTreeMap<Stake, usize> {
    let mut out = BTreeMap::new();
    for r in rows {
        if let Some(s) = stakes.get(&r.persona) {
            *out.entry(*s).or_insert(0) += r.blockers.len();
        }
    }
    out
}

/// `stakes_before` and `stakes_after` are the stakes each run declared.
pub fn compute(
    before: &[ScoreRow],
    after: &[ScoreRow],
    step: i64,
    stakes_before: &BTreeMap<String, Stake>,
    stakes_after: &BTreeMap<String, Stake>,
) -> Delta {
    let (b, a) = (index(before), index(after));
    let keys: BTreeSet<&Key> = b.keys().chain(a.keys()).collect();
    let mut moves = Vec::new();
    let mut regressions = Vec::new();
    let mut incomplete = Vec::new();
    for k in keys {
        let (bs, as_) = (b.get(k).copied(), a.get(k).copied());
        let moved = match (bs, as_) {
            (Some(x), Some(y)) => Some(y - x),
            _ => None,
        };
        let m = Move {
            persona: k.0.clone(),
            lens: k.1.clone(),
            dimension: k.2.clone(),
            about: k.3.clone(),
            before: bs,
            after: as_,
            moved,
        };
        match (bs, as_) {
            (Some(x), Some(y)) if y < x - step => regressions.push(m.clone()),
            (Some(_), None) => incomplete.push(CellKey {
                persona: k.0.clone(),
                lens: k.1.clone(),
                dimension: k.2.clone(),
                about: k.3.clone(),
            }),
            _ => {}
        }
        moves.push(m);
    }
    let (sb, sa) = (
        by_stratum(before, stakes_before),
        by_stratum(after, stakes_after),
    );
    let mut blockers_by_stratum = BTreeMap::new();
    for s in sb.keys().chain(sa.keys()) {
        blockers_by_stratum.insert(
            *s,
            BlockerCount {
                before: sb.get(s).copied().unwrap_or(0),
                after: sa.get(s).copied().unwrap_or(0),
            },
        );
    }
    let blockers_before: usize = before.iter().map(|r| r.blockers.len()).sum();
    let blockers_after: usize = after.iter().map(|r| r.blockers.len()).sum();
    let no_stratum_rose = blockers_by_stratum.values().all(|c| c.after <= c.before);
    let landed = regressions.is_empty()
        && incomplete.is_empty()
        && no_stratum_rose
        && ((blockers_before == 0 && blockers_after == 0) || blockers_after < blockers_before);
    Delta {
        moves,
        blockers_before,
        blockers_after,
        blockers_by_stratum,
        regressions,
        incomplete,
        landed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::Evidence;

    fn st(v: &[(&str, Stake)]) -> BTreeMap<String, Stake> {
        v.iter().map(|(p, s)| (p.to_string(), *s)).collect()
    }
    /// every persona primary in both runs
    fn go(before: &[ScoreRow], after: &[ScoreRow], step: i64) -> Delta {
        let s = st(&[("a", Stake::Primary), ("b", Stake::Primary)]);
        compute(before, after, step, &s, &s)
    }

    fn row(persona: &str, dim: &str, score: i64, blockers: usize) -> ScoreRow {
        ScoreRow {
            persona: persona.into(),
            lens: "L".into(),
            dimension: dim.into(),
            about: None,
            score,
            anchor_met: 1,
            evidence: Evidence::Told,
            citations: vec![],
            blockers: (0..blockers).map(|i| format!("b{i}")).collect(),
        }
    }
    #[test]
    fn delta_landed_when_blockers_down() {
        let d = go(&[row("a", "x", 3, 2)], &[row("a", "x", 4, 1)], 2);
        assert_eq!((d.blockers_before, d.blockers_after), (2, 1));
        assert_eq!(d.moves[0].moved, Some(1));
        assert!(d.regressions.is_empty());
        assert!(d.landed);
    }
    #[test]
    fn delta_landed_when_no_blockers_either_side() {
        assert!(go(&[row("a", "x", 3, 0)], &[row("a", "x", 3, 0)], 2).landed);
    }
    #[test]
    fn delta_not_landed_when_blockers_not_down() {
        assert!(!go(&[row("a", "x", 3, 1)], &[row("a", "x", 3, 1)], 2).landed);
        assert!(!go(&[row("a", "x", 3, 0)], &[row("a", "x", 3, 1)], 2).landed);
    }
    #[test]
    fn delta_not_landed_when_dimension_drops_two_anchors() {
        let d = go(&[row("a", "x", 5, 2)], &[row("a", "x", 2, 0)], 2);
        assert_eq!(d.regressions.len(), 1);
        assert_eq!(d.regressions[0].moved, Some(-3));
        assert!(!d.landed);
    }
    #[test]
    fn drop_of_exactly_one_anchor_is_not_a_regression() {
        let d = go(&[row("a", "x", 5, 0)], &[row("a", "x", 3, 0)], 2);
        assert!(d.regressions.is_empty());
        assert!(d.landed);
    }
    #[test]
    fn delta_keys_on_about() {
        let about = |dim: &str, a: &str, score: i64| {
            let mut r = row("a", dim, score, 0);
            r.about = Some(a.into());
            r
        };
        let d = go(
            &[about("x", "a.png", 5), about("x", "b.png", 2)],
            &[about("x", "a.png", 5), about("x", "b.png", 4)],
            2,
        );
        let moves: Vec<(Option<&str>, Option<i64>)> = d
            .moves
            .iter()
            .map(|m| (m.about.as_deref(), m.moved))
            .collect();
        assert_eq!(
            moves,
            vec![(Some("a.png"), Some(0)), (Some("b.png"), Some(2))]
        );
        // one option dropping two anchors is a regression even when another rose
        let d = go(
            &[about("x", "a.png", 5), about("x", "b.png", 2)],
            &[about("x", "a.png", 1), about("x", "b.png", 5)],
            2,
        );
        assert_eq!(d.regressions.len(), 1);
        assert_eq!(d.regressions[0].about.as_deref(), Some("a.png"));
    }
    #[test]
    fn cells_missing_on_one_side_have_no_move() {
        let d = go(&[row("a", "x", 3, 0)], &[row("b", "x", 4, 0)], 2);
        assert_eq!(d.moves.len(), 2);
        assert_eq!(d.moves[0].after, None);
        assert_eq!(d.moves[0].moved, None);
        assert_eq!(d.moves[1].before, None);
        // a's cell is missing after: incomplete, never landed (C2)
        assert_eq!(d.incomplete.len(), 1);
        assert_eq!(d.incomplete[0].persona, "a");
        assert!(!d.landed);
    }
    #[test]
    fn an_empty_later_run_never_lands() {
        let d = go(&[row("a", "x", 3, 1), row("b", "x", 3, 0)], &[], 2);
        assert_eq!(d.blockers_after, 0);
        assert_eq!(d.incomplete.len(), 2);
        assert!(!d.landed);
    }
    #[test]
    fn a_stratum_whose_blockers_rise_does_not_land() {
        let s = st(&[("a", Stake::Primary), ("b", Stake::Bystander)]);
        let d = compute(
            &[row("a", "x", 3, 1), row("b", "x", 3, 3)],
            &[row("a", "x", 3, 3), row("b", "x", 3, 0)],
            2,
            &s,
            &s,
        );
        assert_eq!((d.blockers_before, d.blockers_after), (4, 3));
        assert_eq!(
            d.blockers_by_stratum[&Stake::Primary],
            BlockerCount {
                before: 1,
                after: 3
            }
        );
        assert_eq!(
            d.blockers_by_stratum[&Stake::Bystander],
            BlockerCount {
                before: 3,
                after: 0
            }
        );
        assert!(!d.landed);
        // the same persona moved to another stratum counts in its new one
        let later = st(&[("a", Stake::Secondary), ("b", Stake::Bystander)]);
        let d = compute(
            &[row("a", "x", 3, 1)],
            &[row("a", "x", 3, 0)],
            2,
            &s,
            &later,
        );
        assert_eq!(d.blockers_by_stratum.len(), 2);
        assert!(d.landed);
    }
}

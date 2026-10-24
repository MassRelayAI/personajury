//! The matrix: per stratum, side by side, never averaged across (report §2.3, §5).
use super::model::{
    Aggregate, Cell, Evidence, EvidenceCounts, Lens, Matrix, ScoreRow, Stake, Stratum,
};
use super::refuse::Refusal;
use std::collections::BTreeMap;

/// `lenses` gives the column order (lens order as passed, dimensions as each lens declares them).
pub fn build(
    rows: &[ScoreRow],
    stakes: &BTreeMap<String, Stake>,
    lenses: &[Lens],
    agg: Aggregate,
) -> Result<Matrix, Refusal> {
    if agg == Aggregate::AcrossStrata {
        return Err(Refusal::StrataAverageRefused);
    }
    let mut strata = Vec::new();
    for stake in [Stake::Primary, Stake::Secondary, Stake::Bystander] {
        let in_stratum: Vec<&ScoreRow> = rows
            .iter()
            .filter(|r| stakes.get(&r.persona) == Some(&stake))
            .collect();
        if in_stratum.is_empty() {
            continue;
        }
        let mut personas: Vec<String> = in_stratum.iter().map(|r| r.persona.clone()).collect();
        personas.sort();
        personas.dedup();
        let mut cells = Vec::new();
        for lens in lenses {
            for dim in &lens.dimensions {
                // keyed (lens, dimension, about): no about first, then about ascending (Option orders None first);
                // one row per persona in a cell, a later row replacing an earlier one
                let mut by_about: BTreeMap<Option<&str>, BTreeMap<&str, &ScoreRow>> =
                    BTreeMap::new();
                for r in in_stratum
                    .iter()
                    .filter(|r| r.lens == lens.name && r.dimension == dim.id)
                {
                    by_about
                        .entry(r.about.as_deref())
                        .or_default()
                        .insert(r.persona.as_str(), r);
                }
                for (about, by_persona) in by_about {
                    let mut vals: Vec<i64> = by_persona.values().map(|r| r.score).collect();
                    vals.sort();
                    let n = vals.len();
                    // the lower middle (D6 A6): an integer a score can be, never a mean
                    let median = vals[(n - 1) / 2];
                    let mut evidence = EvidenceCounts::default();
                    for r in by_persona.values() {
                        evidence.add(r.evidence);
                    }
                    cells.push(Cell {
                        lens: lens.name.clone(),
                        dimension: dim.id.clone(),
                        about: about.map(str::to_string),
                        scores: by_persona
                            .iter()
                            .map(|(p, r)| (p.to_string(), r.score))
                            .collect(),
                        n,
                        median,
                        min: vals[0],
                        max: vals[n - 1],
                        blockers: by_persona.values().map(|r| r.blockers.len()).sum(),
                        within_one: super::metrics::within_one(&vals, median),
                        evidence,
                        classes: by_persona
                            .iter()
                            .map(|(p, r)| (p.to_string(), r.evidence))
                            .collect(),
                    });
                }
            }
        }
        strata.push(Stratum {
            stake,
            personas,
            cells,
        });
    }
    Ok(Matrix { strata })
}
/// The legend line under every table (I4).
pub const LEGEND: &str = "scores: r = recalled, t = told, unmarked = seen";

/// Markdown: a row per persona per stratum (when the cells carry scores), then a `median` row per stratum, then the
/// legend. Columns are the union of `lens:dimension[@about]` across strata in first-seen order; `—` marks no score.
/// A persona's score is suffixed `r` when recalled and `t` when told (I4: §2.2 "shown as such"); a median never is.
pub fn render_table(m: &Matrix) -> String {
    render_table_with(m, &BTreeMap::new())
}

/// `render_table`, a persona with a credential (D19) marked ` (C)` in its row: the badge stays inside its stratum.
pub fn render_table_with(m: &Matrix, credentials: &BTreeMap<String, String>) -> String {
    type Col = (String, String, Option<String>);
    let mut cols: Vec<Col> = Vec::new();
    for s in &m.strata {
        for c in &s.cells {
            let k = (c.lens.clone(), c.dimension.clone(), c.about.clone());
            if !cols.contains(&k) {
                cols.push(k);
            }
        }
    }
    let mut out = String::from("| stratum | persona |");
    for (l, d, a) in &cols {
        match a {
            Some(a) => out.push_str(&format!(" {l}:{d}@{a} |")),
            None => out.push_str(&format!(" {l}:{d} |")),
        }
    }
    out.push('\n');
    out.push_str(&"|---".repeat(cols.len() + 2));
    out.push_str("|\n");
    for s in &m.strata {
        let cell = |col: &Col| {
            s.cells
                .iter()
                .find(|c| c.lens == col.0 && c.dimension == col.1 && c.about == col.2)
        };
        if s.cells.iter().any(|c| !c.scores.is_empty()) {
            for p in &s.personas {
                let badge = if credentials.contains_key(p) { " (C)" } else { "" };
                out.push_str(&format!("| {} | {}{} |", s.stake.name(), p, badge));
                for col in &cols {
                    match cell(col).and_then(|c| c.scores.get(p).map(|v| (v, c.classes.get(p)))) {
                        Some((v, e)) => {
                            let mark = match e {
                                Some(Evidence::Recalled) => "r",
                                Some(Evidence::Told) => "t",
                                _ => "",
                            };
                            out.push_str(&format!(" {v}{mark} |"))
                        }
                        None => out.push_str(" — |"),
                    }
                }
                out.push('\n');
            }
        }
        out.push_str(&format!("| {} | median |", s.stake.name()));
        for col in &cols {
            match cell(col) {
                Some(c) => out.push_str(&format!(" {} |", c.median)),
                None => out.push_str(" — |"),
            }
        }
        out.push('\n');
    }
    out.push('\n');
    out.push_str(LEGEND);
    out.push('\n');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::{Dimension, Evidence, Stratum};

    fn lens(name: &str, dims: &[&str]) -> Lens {
        Lens {
            name: name.into(),
            version: 1,
            scale: [1, 5],
            asks_pick: false,
            compares: false,
            evidence: vec![Evidence::Seen],
            standard: String::new(),
            dimensions: dims
                .iter()
                .map(|d| Dimension {
                    id: d.to_string(),
                    title: d.to_string(),
                    anchors: BTreeMap::new(),
                })
                .collect(),
        }
    }
    fn row(persona: &str, lens: &str, dim: &str, score: i64, blockers: usize) -> ScoreRow {
        ScoreRow {
            persona: persona.into(),
            lens: lens.into(),
            dimension: dim.into(),
            about: None,
            score,
            anchor_met: 1,
            evidence: Evidence::Seen,
            citations: vec![],
            blockers: (0..blockers).map(|i| format!("b{i}")).collect(),
        }
    }
    fn stakes() -> BTreeMap<String, Stake> {
        BTreeMap::from([
            ("a".to_string(), Stake::Primary),
            ("b".to_string(), Stake::Primary),
            ("c".to_string(), Stake::Bystander),
            ("d".to_string(), Stake::Secondary),
        ])
    }
    #[test]
    fn across_strata_aggregate_refused() {
        let e = build(&[], &stakes(), &[], Aggregate::AcrossStrata).unwrap_err();
        assert_eq!(e.code(), "strata_average_refused");
    }
    #[test]
    fn matrix_has_no_cross_strata_field() {
        // exhaustive destructuring: adding a field to Matrix breaks this test
        let Matrix { strata } = Matrix { strata: vec![] };
        let Stratum {
            stake: _,
            personas: _,
            cells: _,
        } = Stratum {
            stake: Stake::Primary,
            personas: vec![],
            cells: vec![],
        };
        assert!(strata.is_empty());
    }
    #[test]
    fn median_of_even_count_is_lower_middle() {
        let rows = [row("a", "L", "x", 2, 0), row("b", "L", "x", 5, 0)];
        let m = build(
            &rows,
            &stakes(),
            &[lens("L", &["x"])],
            Aggregate::PerStratum,
        )
        .unwrap();
        let c = &m.strata[0].cells[0];
        assert_eq!((c.n, c.median, c.min, c.max), (2, 2, 2, 5));
        assert_eq!(c.scores.get("a"), Some(&2));
        // four scores 1, 2, 4, 5: the lower middle is 2, not 3
        let st: BTreeMap<String, Stake> = ["a", "b", "c", "d"]
            .iter()
            .map(|p| (p.to_string(), Stake::Primary))
            .collect();
        let rows = [
            row("a", "L", "x", 5, 0),
            row("b", "L", "x", 1, 0),
            row("c", "L", "x", 4, 0),
            row("d", "L", "x", 2, 0),
        ];
        let m = build(&rows, &st, &[lens("L", &["x"])], Aggregate::PerStratum).unwrap();
        assert_eq!(m.strata[0].cells[0].median, 2);
    }
    #[test]
    fn cells_are_keyed_by_about() {
        let mut on_b = row("a", "L", "x", 4, 1);
        on_b.about = Some("b.png".into());
        let mut on_a = row("a", "L", "x", 2, 0);
        on_a.about = Some("a.png".into());
        let mut other_on_a = row("b", "L", "x", 5, 0);
        other_on_a.about = Some("a.png".into());
        let rows = [
            on_b,
            row("a", "L", "y", 3, 0),
            on_a,
            row("a", "L", "x", 3, 0),
            other_on_a,
        ];
        let m = build(
            &rows,
            &stakes(),
            &[lens("L", &["x", "y"])],
            Aggregate::PerStratum,
        )
        .unwrap();
        let cols: Vec<(&str, Option<&str>, usize, i64)> = m.strata[0]
            .cells
            .iter()
            .map(|c| (c.dimension.as_str(), c.about.as_deref(), c.n, c.median))
            .collect();
        assert_eq!(
            cols,
            vec![
                ("x", None, 1, 3),
                ("x", Some("a.png"), 2, 2),
                ("x", Some("b.png"), 1, 4),
                ("y", None, 1, 3)
            ]
        );
        let t = render_table(&m);
        assert!(
            t.starts_with("| stratum | persona | L:x | L:x@a.png | L:x@b.png | L:y |\n"),
            "{t}"
        );
        assert!(t.contains("| primary | median | 3 | 2 | 4 | 3 |\n"), "{t}");
    }
    #[test]
    fn median_of_odd_count() {
        let st: BTreeMap<String, Stake> = ["a", "b", "c"]
            .iter()
            .map(|p| (p.to_string(), Stake::Primary))
            .collect();
        let rows = [
            row("a", "L", "x", 5, 0),
            row("b", "L", "x", 1, 0),
            row("c", "L", "x", 3, 0),
        ];
        let m = build(&rows, &st, &[lens("L", &["x"])], Aggregate::PerStratum).unwrap();
        assert_eq!(m.strata[0].cells[0].median, 3);
    }
    #[test]
    fn strata_order_primary_secondary_bystander() {
        let rows = [
            row("c", "L", "x", 1, 0),
            row("d", "L", "x", 2, 0),
            row("a", "L", "x", 3, 0),
        ];
        let m = build(
            &rows,
            &stakes(),
            &[lens("L", &["x"])],
            Aggregate::PerStratum,
        )
        .unwrap();
        let order: Vec<Stake> = m.strata.iter().map(|s| s.stake).collect();
        assert_eq!(
            order,
            vec![Stake::Primary, Stake::Secondary, Stake::Bystander]
        );
    }
    #[test]
    fn empty_strata_and_unstaked_personas_are_skipped() {
        let rows = [row("a", "L", "x", 3, 0), row("ghost", "L", "x", 1, 0)];
        let m = build(
            &rows,
            &stakes(),
            &[lens("L", &["x"])],
            Aggregate::PerStratum,
        )
        .unwrap();
        assert_eq!(m.strata.len(), 1);
        assert_eq!(m.strata[0].personas, vec!["a"]);
        assert_eq!(m.strata[0].cells[0].n, 1);
    }
    #[test]
    fn columns_follow_lens_order_and_blockers_count() {
        let rows = [
            row("a", "L2", "z", 4, 2),
            row("a", "L1", "y", 3, 1),
            row("a", "L1", "x", 2, 0),
        ];
        let m = build(
            &rows,
            &stakes(),
            &[lens("L1", &["x", "y"]), lens("L2", &["z"])],
            Aggregate::PerStratum,
        )
        .unwrap();
        let cols: Vec<(&str, &str, usize)> = m.strata[0]
            .cells
            .iter()
            .map(|c| (c.lens.as_str(), c.dimension.as_str(), c.blockers))
            .collect();
        assert_eq!(cols, vec![("L1", "x", 0), ("L1", "y", 1), ("L2", "z", 2)]);
    }
    #[test]
    fn render_table_is_pinned_byte_for_byte() {
        let rows = [
            row("a", "L", "x", 2, 0),
            row("b", "L", "x", 5, 0),
            row("a", "L", "y", 4, 0),
            row("c", "L", "x", 1, 0),
        ];
        let m = build(
            &rows,
            &stakes(),
            &[lens("L", &["x", "y"])],
            Aggregate::PerStratum,
        )
        .unwrap();
        let want = "\
| stratum | persona | L:x | L:y |
|---|---|---|---|
| primary | a | 2 | 4 |
| primary | b | 5 | — |
| primary | median | 2 | 4 |
| bystander | c | 1 | — |
| bystander | median | 1 | — |

scores: r = recalled, t = told, unmarked = seen
";
        assert_eq!(render_table(&m), want);
    }
    #[test]
    fn table_marks_recalled_and_told_scores_and_counts_classes() {
        let mut told = row("a", "L", "x", 2, 0);
        told.evidence = Evidence::Told;
        let mut recalled = row("b", "L", "x", 5, 0);
        recalled.evidence = Evidence::Recalled;
        let rows = [told, recalled, row("c", "L", "x", 3, 0)];
        let m = build(
            &rows,
            &stakes(),
            &[lens("L", &["x"])],
            Aggregate::PerStratum,
        )
        .unwrap();
        let c = &m.strata[0].cells[0];
        assert_eq!(
            c.evidence,
            EvidenceCounts {
                seen: 0,
                told: 1,
                recalled: 1
            }
        );
        let want = "\
| stratum | persona | L:x |
|---|---|---|
| primary | a | 2t |
| primary | b | 5r |
| primary | median | 2 |
| bystander | c | 3 |
| bystander | median | 3 |

scores: r = recalled, t = told, unmarked = seen
";
        assert_eq!(render_table(&m), want);
        // the classes drive the table only; the JSON carries the counts
        let v = serde_json::to_value(c).unwrap();
        assert!(v.get("classes").is_none());
        assert_eq!(v["evidence"]["recalled"], 1);
    }
}

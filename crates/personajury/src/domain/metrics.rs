//! The honest metrics (D34): counts per stratum, never pooled, never a mean. Pure: it reads cells and rows the matrix
//! already holds and counts. No key it writes is a mean, an average, a share, a rate, a percent or a probability.
use super::model::{Lens, Matrix, ScoreRow, Stake};
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// How many scores sit within one point of the cell's median (an integer; the median is the lower middle).
pub fn within_one(scores: &[i64], median: i64) -> usize {
    scores.iter().filter(|s| (**s - median).abs() <= 1).count()
}

/// The persuasion shift of each stratum that has deliberated rows: per cell and persona `{before, after, step}` (`step` is
/// the change in anchor steps: the anchors at or below the score, counted), and the counts of persona-cells that moved
/// up, down or held, and that moved toward or away from the first median (the initial batch's median of the cell).
/// `initial` is the matrix of the jury's first word.
pub fn shift(initial: &Matrix, deliberated: &[ScoreRow], lenses: &[Lens]) -> BTreeMap<Stake, Value> {
    let anchors = |lens: &str, dim: &str| -> Vec<i64> {
        lenses
            .iter()
            .find(|l| l.name == lens)
            .and_then(|l| l.dimensions.iter().find(|d| d.id == dim))
            .map(|d| d.anchor_scores())
            .unwrap_or_default()
    };
    let mut out = BTreeMap::new();
    for st in &initial.strata {
        let (mut up, mut down, mut held, mut toward, mut away) = (0usize, 0usize, 0usize, 0usize, 0usize);
        let mut cells = Vec::new();
        for c in &st.cells {
            let a = anchors(&c.lens, &c.dimension);
            let at = |s: i64| a.iter().filter(|x| **x <= s).count() as i64;
            let mut people = Vec::new();
            for (p, before) in &c.scores {
                let Some(r) = deliberated
                    .iter()
                    .find(|r| &r.persona == p && r.lens == c.lens && r.dimension == c.dimension && r.about == c.about)
                else {
                    continue;
                };
                let after = r.score;
                match after.cmp(before) {
                    std::cmp::Ordering::Greater => up += 1,
                    std::cmp::Ordering::Less => down += 1,
                    std::cmp::Ordering::Equal => held += 1,
                }
                let (db, da) = ((before - c.median).abs(), (after - c.median).abs());
                if da < db {
                    toward += 1;
                } else if da > db {
                    away += 1;
                }
                people.push(json!({"persona": p, "before": before, "after": after, "step": at(after) - at(*before)}));
            }
            if !people.is_empty() {
                cells.push(json!({"lens": c.lens, "dimension": c.dimension, "about": c.about, "personas": people}));
            }
        }
        if !cells.is_empty() {
            out.insert(
                st.stake,
                json!({"moved_up": up, "moved_down": down, "held": held, "toward_first_median": toward,
                       "away_from_first_median": away, "cells": cells}),
            );
        }
    }
    out
}

/// The `panel:` footer lines of a matrix or summary: per run, the panel's terms as asked, seated and answered. Runs with
/// the same terms share a line.
pub fn panel_lines(panel_match: &Value) -> Vec<String> {
    let mut groups: Vec<(Vec<i64>, String)> = Vec::new();
    let mut runs: Vec<i64> = Vec::new();
    for t in panel_match.as_array().into_iter().flatten() {
        if let Some(r) = t["run"].as_i64() {
            if !runs.contains(&r) {
                runs.push(r);
            }
        }
    }
    for r in runs {
        let terms: Vec<String> = panel_match
            .as_array()
            .into_iter()
            .flatten()
            .filter(|t| t["run"].as_i64() == Some(r))
            .map(|t| {
                format!(
                    "asked {}, seated {}, answered {}",
                    t["term"].as_str().unwrap_or(""),
                    t["seated"],
                    t["answered"]
                )
            })
            .collect();
        let text = terms.join("; ");
        match groups.iter_mut().find(|(_, g)| *g == text) {
            Some((ids, _)) => ids.push(r),
            None => groups.push((vec![r], text)),
        }
    }
    let many = groups.len() > 1;
    groups
        .into_iter()
        .map(|(ids, text)| {
            if many {
                let ids: Vec<String> = ids.iter().map(i64::to_string).collect();
                format!("run {} panel: {text}", ids.join(", "))
            } else {
                format!("panel: {text}")
            }
        })
        .collect()
}

/// The `cost:` line: the tokens the agent reported over the jurors who reported, beside the project's estimate; a dash
/// when nothing was reported. Nothing is divided.
pub fn cost_line(cost: &Value) -> String {
    let est = cost["estimate"]["tokens"].as_u64().unwrap_or(0);
    let n = cost["jurors_reporting"].as_u64().unwrap_or(0);
    if n == 0 {
        return format!("cost: reported \u{2014} (estimate {est})");
    }
    format!(
        "cost: reported {} in, {} out over {} jurors (estimate {est})",
        cost["reported_total"]["input_tokens"], cost["reported_total"]["output_tokens"], n
    )
}

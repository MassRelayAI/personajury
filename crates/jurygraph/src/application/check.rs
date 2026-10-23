//! check: stale runs, unresolved findings, and fixes no later run has looked at (plan Task 5, D6 B3, final review C1,
//! C3, I5)
use super::verdict::{findings_of, moved_why, open};
use super::{AppError, Result};
use crate::domain::{Finding, RunRow};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// A finding as check sees it: the first run that raised it and every run that raised it, in run order.
struct Raised<'a> {
    finding: Finding,
    first: &'a RunRow,
    runs: Vec<i64>,
}

pub fn run(root: &Path, history: bool) -> Result<Value> {
    let store = open(root)?;
    let runs = store.runs()?;
    let mut stale = Vec::new();
    // personas with verdict lines in each run (any batch): what a later run has actually judged (I5)
    let mut judged: BTreeMap<i64, BTreeSet<String>> = BTreeMap::new();
    let mut order: Vec<String> = Vec::new();
    let mut raised: BTreeMap<String, Raised> = BTreeMap::new();
    for r in &runs {
        if let Some(why) = moved_why(&store, r)? {
            stale.push(json!({"run": r.id, "subject": r.subject_name, "why": why}));
        }
        // every batch (C3): a re-ingest without the blocker never clears it; a retracted point needs a decision
        let rows = store.verdict_rows(r.id, true)?;
        judged.insert(r.id, rows.iter().map(|v| v.persona.clone()).collect());
        for f in findings_of(&r.subject_name, &rows) {
            match raised.get_mut(&f.id) {
                Some(x) => {
                    if !x.runs.contains(&r.id) {
                        x.runs.push(r.id);
                    }
                }
                None => {
                    order.push(f.id.clone());
                    raised.insert(
                        f.id.clone(),
                        Raised {
                            finding: f,
                            first: r,
                            runs: vec![r.id],
                        },
                    );
                }
            }
        }
    }
    let mut unresolved = Vec::new();
    let mut decisions = Vec::new();
    let mut fixed_unverified = Vec::new();
    for id in &order {
        let x = &raised[id];
        let f = &x.finding;
        if history {
            for d in store.decisions(id)? {
                decisions.push(serde_json::to_value(d).expect("decision serialises"));
            }
        }
        let item = |run: i64, reason: &str| {
            json!({
                "finding": f.id, "run": run, "persona": f.persona, "dimension": f.dimension, "about": f.about,
                "text": f.text, "reason": reason,
            })
        };
        match store.latest_decision(id)? {
            None => unresolved.push(item(x.first.id, "no_decision")),
            Some(d) if d.outcome == "fixed" => {
                // a run created after the decision raises it again: the fix did not land (C1)
                if let Some(again) = x.runs.iter().find(|r| **r > d.after_run) {
                    unresolved.push(item(*again, "fix_not_landed"));
                    continue;
                }
                // verified only by a later run on the same subject and lens in which that persona judged (I5)
                let verified = runs.iter().any(|r| {
                    r.id > d.after_run
                        && r.subject_name == x.first.subject_name
                        && r.lens_name == x.first.lens_name
                        && judged.get(&r.id).is_some_and(|p| p.contains(&f.persona))
                });
                if !verified {
                    fixed_unverified.push(json!({
                        "finding": f.id, "run": x.first.id, "subject": x.first.subject_name, "persona": f.persona,
                        "dimension": f.dimension, "about": f.about, "text": f.text,
                    }));
                }
            }
            Some(_) => {}
        }
    }
    // listed, never failed: a fix claimed with no later run is visible as such (report §10(2))
    let mut data = json!({
        "runs": runs.len(), "stale": stale, "unresolved": unresolved, "fixed_unverified": fixed_unverified,
    });
    if !unresolved.is_empty() {
        let n = data["unresolved"].as_array().map_or(0, Vec::len);
        return Err(AppError::CheckFailed(
            format!("{n} finding(s) have no decision or a fix that did not land"),
            data,
        ));
    }
    if history {
        data["decisions"] = json!(decisions);
    }
    Ok(data)
}

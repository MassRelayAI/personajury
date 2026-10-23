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

/// What one run says about a finding in its rounds (D11): the life `check --history` lists and the graph draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum State {
    /// the run raised it
    Raised,
    /// the run raised it after a `fixed` decision: the fix did not land
    RaisedAgain,
    /// a later run in which the persona judged did not raise it
    Fixed,
    /// after a `fixed` decision, but the persona did not judge in this run
    Unjudged,
    /// not raised again, nothing claimed fixed
    Absent,
}
impl State {
    pub(crate) fn name(self) -> &'static str {
        match self {
            State::Raised => "raised",
            State::RaisedAgain => "raised_again",
            State::Fixed => "fixed",
            State::Unjudged => "unjudged",
            State::Absent => "absent",
        }
    }
}

/// A finding with its decision and its state in each round (a run of the same subject and lens, from the first that
/// raised it on), in run order.
pub(crate) struct Life {
    pub finding: Finding,
    /// open | fixed | dismissed | row; a fix a later run raised again is open
    pub status: &'static str,
    pub decision: Option<crate::domain::DecisionRow>,
    pub states: Vec<(i64, State)>,
}

/// The lives of every finding raised in `runs` (any batch), in order of first appearance.
pub(crate) fn lives(store: &crate::store::Store, runs: &[RunRow]) -> Result<Vec<Life>> {
    let mut judged: BTreeMap<i64, BTreeSet<String>> = BTreeMap::new();
    let mut order: Vec<(Finding, usize)> = Vec::new();
    let mut raised: BTreeMap<String, Vec<i64>> = BTreeMap::new();
    for (i, r) in runs.iter().enumerate() {
        let rows = store.verdict_rows(r.id, true)?;
        judged.insert(r.id, rows.iter().map(|v| v.persona.clone()).collect());
        for f in findings_of(&r.subject_name, &rows) {
            let ids = raised.entry(f.id.clone()).or_default();
            if ids.is_empty() {
                order.push((f, i));
            }
            if !ids.contains(&r.id) {
                ids.push(r.id);
            }
        }
    }
    let mut out = Vec::new();
    for (f, first) in order {
        let decision = store.latest_decision(&f.id)?;
        let fixed_after = decision.as_ref().filter(|d| d.outcome == "fixed").map(|d| d.after_run);
        // every `fixed` decision, in order (they are append-only): a run after the first of them is "after a fix",
        // so a second `fixed` never rewrites the step that did not land
        let first_fix = store
            .decisions(&f.id)?
            .iter()
            .filter(|d| d.outcome == "fixed")
            .map(|d| d.after_run)
            .min();
        let hits = &raised[&f.id];
        let status = match (&decision, fixed_after) {
            (None, _) => "open",
            (Some(_), Some(after)) if hits.iter().any(|r| *r > after) => "open",
            (Some(d), _) => match d.outcome.as_str() {
                "fixed" => "fixed",
                "dismissed" => "dismissed",
                _ => "row",
            },
        };
        let (subject, lens) = (&runs[first].subject_name, &runs[first].lens_name);
        let mut states = Vec::new();
        for r in runs.iter().skip(first) {
            if r.subject_name != *subject || r.lens_name != *lens {
                continue;
            }
            let after_fix = first_fix.is_some_and(|a| r.id > a);
            let state = if hits.contains(&r.id) {
                if after_fix {
                    State::RaisedAgain
                } else {
                    State::Raised
                }
            } else if after_fix {
                if judged.get(&r.id).is_some_and(|p| p.contains(&f.persona)) {
                    State::Fixed
                } else {
                    State::Unjudged
                }
            } else {
                State::Absent
            };
            states.push((r.id, state));
        }
        out.push(Life { finding: f, status, decision, states });
    }
    Ok(out)
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
    if history {
        data["lives"] = json!(lives(&store, &runs)?
            .iter()
            .map(|l| json!({
                "finding": l.finding.id, "persona": l.finding.persona, "dimension": l.finding.dimension,
                "about": l.finding.about, "text": l.finding.text, "status": l.status,
                "decision": l.decision.as_ref().map(|d| json!({"outcome": d.outcome, "after_run": d.after_run})),
                "life": l.states.iter().map(|(r, s)| json!({"run": r, "state": s.name()})).collect::<Vec<_>>(),
            }))
            .collect::<Vec<_>>());
    }
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

//! pick: the forced choices of a run, per stratum (plan Task 5)
use super::verdict::{load_run, open};
use super::Result;
use crate::domain::*;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::Path;

pub fn show(root: &Path, run: i64) -> Result<Value> {
    let store = open(root)?;
    let run = load_run(&store, run)?;
    let subject = store.subject_at(&run.subject_name, run.subject_version)?;
    if let Some(s) = subject.filter(|s| s.kind == SubjectKind::Decision) {
        return decision_picks(root, &store, &run, &s);
    }
    let mut by: BTreeMap<Stake, BTreeMap<String, Vec<(String, String)>>> = BTreeMap::new();
    for p in store.picks(run.id, false)? {
        if let Some(stake) = run.stakes.get(&p.persona) {
            by.entry(*stake)
                .or_default()
                .entry(p.choice)
                .or_default()
                .push((p.persona, p.reason));
        }
    }
    let strata: Vec<Value> = by
        .into_iter()
        .map(|(stake, choices)| {
            let mut list: Vec<(String, Vec<(String, String)>)> = choices.into_iter().collect();
            list.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then_with(|| a.0.cmp(&b.0)));
            let choices: Vec<Value> = list
                .into_iter()
                .map(|(choice, who)| {
                    json!({
                        "choice": choice,
                        "count": who.len(),
                        "personas": who.iter().map(|w| &w.0).collect::<Vec<_>>(),
                        "reasons": who.iter().map(|w| json!({"persona": w.0, "reason": w.1})).collect::<Vec<_>>(),
                    })
                })
                .collect();
            json!({ "stake": stake, "choices": choices })
        })
        .collect();
    Ok(json!({ "run": run.id, "strata": strata }))
}

/// A decision run's picks (D37): per question, per stratum, the choices with their counts and reasons.
fn decision_picks(root: &Path, store: &crate::store::Store, run: &RunRow, subject: &Subject) -> Result<Value> {
    let questions = super::decision::questions_of(root, subject)?;
    let picks = store.picks(run.id, false)?;
    let mut out = Vec::new();
    for q in &questions {
        let mut by: BTreeMap<Stake, BTreeMap<String, Vec<(String, String)>>> = BTreeMap::new();
        for p in picks.iter().filter(|p| p.about.as_deref() == Some(q.id.as_str())) {
            if let Some(stake) = run.stakes.get(&p.persona) {
                by.entry(*stake).or_default().entry(p.choice.clone()).or_default().push((p.persona.clone(), p.reason.clone()));
            }
        }
        let strata: Vec<Value> = by
            .into_iter()
            .map(|(stake, choices)| {
                let mut list: Vec<(String, Vec<(String, String)>)> = choices.into_iter().collect();
                list.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then_with(|| a.0.cmp(&b.0)));
                let choices: Vec<Value> = list
                    .into_iter()
                    .map(|(choice, who)| {
                        json!({
                            "choice": choice,
                            "count": who.len(),
                            "personas": who.iter().map(|w| &w.0).collect::<Vec<_>>(),
                            "reasons": who.iter().map(|w| json!({"persona": w.0, "reason": w.1})).collect::<Vec<_>>(),
                        })
                    })
                    .collect();
                json!({ "stake": stake, "choices": choices })
            })
            .collect();
        out.push(json!({"about": q.id, "question": q.text, "options": q.options, "strata": strata}));
    }
    Ok(json!({ "run": run.id, "strata": [], "questions": out }))
}

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

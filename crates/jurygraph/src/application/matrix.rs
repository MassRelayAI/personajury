//! matrix: per stratum, never averaged across (plan Task 5)
use super::verdict::{load_lens, load_run, open, score_rows};
use super::{AppError, Result};
use crate::domain::*;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::Path;

pub fn show(
    root: &Path,
    runs: &[i64],
    across_strata: bool,
    history: bool,
    table: bool,
    detail: Detail,
) -> Result<Value> {
    if across_strata {
        return Err(Refusal::StrataAverageRefused.into());
    }
    let store = open(root)?;
    let mut rows = Vec::new();
    let mut stakes: BTreeMap<String, Stake> = BTreeMap::new();
    let mut lenses: Vec<Lens> = Vec::new();
    let mut all = Vec::new();
    let mut subject: Option<(i64, String)> = None;
    for id in runs {
        let run = load_run(&store, *id)?;
        // one subject per matrix, and one version per lens name (I3): a cell never merges two of either
        match &subject {
            Some((first, name)) if *name != run.subject_name => {
                return Err(AppError::InvalidArguments(format!(
                    "a matrix is over one subject: run {first} is on {name}, run {id} is on {}",
                    run.subject_name
                )));
            }
            Some(_) => {}
            None => subject = Some((*id, run.subject_name.clone())),
        }
        if let Some(l) = lenses
            .iter()
            .find(|l| l.name == run.lens_name && l.version != run.lens_version)
        {
            return Err(AppError::InvalidArguments(format!(
                "a matrix holds one version of a lens: {} v{} and v{} (run {id})",
                l.name, l.version, run.lens_version
            )));
        }
        for p in &run.personas {
            if let Some(s) = run.stakes.get(p) {
                if stakes.insert(p.clone(), *s).is_some_and(|old| old != *s) {
                    return Err(AppError::InvalidArguments(format!(
                        "one stake per persona in a matrix: {p} has two stakes in runs {runs:?}"
                    )));
                }
            }
        }
        let lens = load_lens(&store, &run)?;
        if !lenses
            .iter()
            .any(|l| l.name == lens.name && l.version == lens.version)
        {
            lenses.push(lens);
        }
        rows.extend(score_rows(&run, &store.verdict_rows(*id, false)?));
        if history {
            all.extend(store.verdict_rows(*id, true)?);
        }
    }
    let mut m = matrix::build(&rows, &stakes, &lenses, Aggregate::PerStratum)?;
    let table_text = table.then(|| matrix::render_table(&m));
    if detail == Detail::Compact {
        for s in &mut m.strata {
            for c in &mut s.cells {
                c.scores.clear();
            }
        }
    }
    let mut out = json!({ "runs": runs, "strata": m.strata });
    if let Some(t) = table_text {
        out["table"] = json!(t);
    }
    if history {
        out["history"] = serde_json::to_value(all).expect("rows serialise");
    }
    Ok(out)
}

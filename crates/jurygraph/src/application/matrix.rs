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
    // runs whose caller departed from a rule: (override, the runs that carry it), in first-seen order
    let mut overrides: Vec<(Value, Vec<i64>)> = Vec::new();
    for id in runs {
        let run = load_run(&store, *id)?;
        if let Some(o) = &run.override_json {
            match overrides.iter_mut().find(|(v, _)| v == o) {
                Some((_, ids)) => ids.push(*id),
                None => overrides.push((o.clone(), vec![*id])),
            }
        }
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
    let table_text = table.then(|| override_header(&overrides, runs.len()) + &matrix::render_table(&m));
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
    if !overrides.is_empty() {
        let with_runs = |(o, ids): &(Value, Vec<i64>)| {
            let mut v = o.clone();
            v["runs"] = json!(ids);
            v
        };
        out["override"] = if overrides.len() == 1 {
            with_runs(&overrides[0])
        } else {
            Value::Array(overrides.iter().map(with_runs).collect())
        };
    }
    if history {
        out["history"] = serde_json::to_value(all).expect("rows serialise");
    }
    Ok(out)
}

/// One line per distinct override: `override (rule <class>): <field> <rule>→<given>, … — reason: <text>`, led by the
/// runs that carry it whenever the matrix holds runs that do not (or runs with another override). Newlines in the
/// reason are escaped: a reason can never add a line that reads like a table row.
fn override_header(overrides: &[(Value, Vec<i64>)], total_runs: usize) -> String {
    let show = |v: &Value| match v {
        Value::String(t) => t.clone(),
        other => other.to_string(),
    };
    let mut out = String::new();
    for (o, ids) in overrides {
        let fields: Vec<String> = o["fields"]
            .as_object()
            .map(|m| {
                m.iter()
                    .map(|(k, f)| format!("{k} {}\u{2192}{}", show(&f["rule"]), show(&f["given"])))
                    .collect()
            })
            .unwrap_or_default();
        let lead = if overrides.len() > 1 || ids.len() != total_runs {
            let ids: Vec<String> = ids.iter().map(i64::to_string).collect();
            format!("run {}: ", ids.join(", "))
        } else {
            String::new()
        };
        let reason = o["reason"].as_str().unwrap_or("").replace('\r', "\\r").replace('\n', "\\n");
        out.push_str(&format!(
            "{lead}override (rule {}): {} \u{2014} reason: {reason}\n",
            o["rule"].as_str().unwrap_or(""),
            fields.join(", ")
        ));
    }
    out
}

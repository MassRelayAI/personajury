//! matrix: per stratum, never averaged across (plan Task 5)
use super::verdict::{load_lens, load_run, open, score_rows, usage_line, usage_summary};
use super::{AppError, Result};
use crate::domain::apply;
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
    // the effort each run was planned under, with what it cut (D17): (effort, trimmed, the runs), first-seen order
    let mut efforts: Vec<(String, Vec<Value>, Vec<i64>)> = Vec::new();
    // each run's independence (D18), in run order
    let mut independence: Vec<(i64, String)> = Vec::new();
    // the models the briefs were for, per stratum (D15)
    let mut models: BTreeMap<Stake, std::collections::BTreeSet<String>> = BTreeMap::new();
    // who entered each human batch (D19)
    let mut human_batches: BTreeMap<String, String> = BTreeMap::new();
    for id in runs {
        let run = load_run(&store, *id)?;
        independence.push((*id, run.independence.clone()));
        for b in &run.briefs {
            models.entry(b.stake).or_default().insert(b.model.clone());
        }
        for (k, v) in store.human_batches(*id)? {
            human_batches.insert(k, v);
        }
        if let Some(e) = &run.effort {
            match efforts.iter_mut().find(|(x, t, _)| x == e && *t == run.trimmed) {
                Some((_, _, ids)) => ids.push(*id),
                None => efforts.push((e.clone(), run.trimmed.clone(), vec![*id])),
            }
        }
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
    // a persona with a credential is badged inside its stratum (D19)
    let mut credentials: BTreeMap<String, String> = BTreeMap::new();
    for id in stakes.keys() {
        if let Some(c) = store.persona(id)?.and_then(|p| super::persona::credential_of(&p)) {
            credentials.insert(id.clone(), c);
        }
    }
    // a lens that points at a standard names it in the header (D19)
    let standards: BTreeMap<String, String> = lenses
        .iter()
        .filter(|l| !l.standard.trim().is_empty())
        .map(|l| (l.name.clone(), l.standard.trim().to_string()))
        .collect();
    // what people entered by hand stands beside the jury's cells, never in them (D19)
    let humans = humans_of(&rows, &stakes, &human_batches)?;
    let usage = usage_summary(root, &store, runs)?;
    let table_text = table.then(|| {
        standard_header(&standards) + &independence_header(&independence) + &effort_header(&efforts, runs.len()) + &override_header(&overrides, runs.len())
            + &matrix::render_table_with(&m, &credentials)
            + &usage_line(&usage).unwrap_or_default()
            + &human_block(&humans)
    });
    if detail == Detail::Compact {
        for s in &mut m.strata {
            for c in &mut s.cells {
                c.scores.clear();
            }
        }
    }
    // strata that disagree on a cell (D13): beside the strata, never inside one
    let splits = apply::splits(&rows, &stakes);
    let mut out = json!({ "runs": runs, "strata": m.strata, "splits": splits, "usage": usage });
    out["models"] = json!(models.iter().map(|(s, m)| (s.name(), m)).collect::<BTreeMap<_, _>>());
    out["models_distinct"] = json!(models.values().flatten().collect::<std::collections::BTreeSet<_>>().len());
    if !credentials.is_empty() {
        out["credentials"] = json!(credentials);
    }
    if !standards.is_empty() {
        out["standards"] = json!(standards);
    }
    if !humans.is_empty() {
        out["human"] = json!(humans.iter().map(|h| h.json()).collect::<Vec<_>>());
    }
    out["independence"] = json!(match independence.iter().map(|(_, w)| w.as_str()).collect::<std::collections::BTreeSet<_>>() {
        words if words.len() == 1 => words.into_iter().next().unwrap_or("independent"),
        _ => "mixed",
    });
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
    if !efforts.is_empty() {
        let mut words: Vec<&str> = Vec::new();
        for (e, _, _) in &efforts {
            if !words.contains(&e.as_str()) {
                words.push(e);
            }
        }
        out["effort"] = json!(words);
        out["trimmed"] = json!(efforts
            .iter()
            .flat_map(|(_, t, ids)| t.iter().map(move |x| {
                let mut x = x.clone();
                x["runs"] = json!(ids);
                x
            }))
            .collect::<Vec<_>>());
    }
    if history {
        out["history"] = serde_json::to_value(all).expect("rows serialise");
    }
    Ok(out)
}

/// A person's verdict, beside the jury (D19).
struct Human {
    persona: String,
    seat: String,
    stake: Stake,
    by: String,
    rows: Vec<ScoreRow>,
}
impl Human {
    fn json(&self) -> Value {
        json!({
            "persona": self.persona, "seat": self.seat, "stake": self.stake, "by": self.by,
            "lines": self.rows.iter().map(|r| json!({
                "lens": r.lens, "dimension": r.dimension, "about": r.about, "score": r.score,
                "anchor_met": r.anchor_met, "evidence": r.evidence, "citations": r.citations, "blockers": r.blockers,
            })).collect::<Vec<_>>(),
        })
    }
}
fn humans_of(
    rows: &[ScoreRow],
    stakes: &BTreeMap<String, Stake>,
    by: &BTreeMap<String, String>,
) -> Result<Vec<Human>> {
    let mut out: Vec<Human> = Vec::new();
    for r in rows.iter().filter(|r| super::verdict::is_human(&r.persona)) {
        if let Some(h) = out.iter_mut().find(|h| h.persona == r.persona) {
            h.rows.push(r.clone());
            continue;
        }
        let seat = r.persona.rsplit(':').next().unwrap_or("").to_string();
        let Some(stake) = stakes.get(&seat).copied() else { continue };
        out.push(Human {
            persona: r.persona.clone(),
            seat,
            stake,
            by: by.get(&r.persona).cloned().unwrap_or_default(),
            rows: vec![r.clone()],
        });
    }
    out.sort_by(|a, b| (a.stake, &a.persona).cmp(&(b.stake, &b.persona)));
    Ok(out)
}
/// The `H` rows under the table: one row per person, the columns the people's own lines make.
fn human_block(humans: &[Human]) -> String {
    if humans.is_empty() {
        return String::new();
    }
    type Col = (String, String, Option<String>);
    let mut cols: Vec<Col> = Vec::new();
    for h in humans {
        for r in &h.rows {
            let k = (r.lens.clone(), r.dimension.clone(), r.about.clone());
            if !cols.contains(&k) {
                cols.push(k);
            }
        }
    }
    let mut out = String::from("\nH = a person's verdict, entered with --by-human: beside the jury's rows, never in a cell of them\n\n| stratum | H persona |");
    for (l, d, a) in &cols {
        match a {
            Some(a) => out.push_str(&format!(" {l}:{d}@{a} |")),
            None => out.push_str(&format!(" {l}:{d} |")),
        }
    }
    out.push('\n');
    out.push_str(&"|---".repeat(cols.len() + 2));
    out.push_str("|\n");
    for h in humans {
        let by = h.by.replace('\r', "\\r").replace('\n', "\\n");
        out.push_str(&format!("| {} | H {} as {} ({by}) |", h.stake.name(), h.persona, h.seat));
        for c in &cols {
            match h.rows.iter().find(|r| (&r.lens, &r.dimension, &r.about) == (&c.0, &c.1, &c.2)) {
                Some(r) => {
                    let mark = match r.evidence {
                        Evidence::Recalled => "r",
                        Evidence::Told => "t",
                        Evidence::Seen => "",
                    };
                    out.push_str(&format!(" {}{mark} |", r.score));
                }
                None => out.push_str(" \u{2014} |"),
            }
        }
        out.push('\n');
    }
    out
}
/// `standard: <lens> — <first line>` per lens that points at a standard (D19); nothing for a lens without one.
fn standard_header(standards: &BTreeMap<String, String>) -> String {
    let mut out = String::new();
    for (lens, text) in standards {
        let first: String = text.lines().next().unwrap_or("").chars().take(200).collect();
        out.push_str(&format!("standard: {lens} \u{2014} {first}\n"));
    }
    out
}

/// `independence: sequential` when every run is, or `independence: run 1 sequential, run 2 independent` when they differ;
/// nothing for a matrix of independent runs (the recorded table is unchanged).
fn independence_header(runs: &[(i64, String)]) -> String {
    if runs.iter().all(|(_, w)| w == "independent") {
        return String::new();
    }
    if runs.iter().all(|(_, w)| *w == runs[0].1) {
        return format!("independence: {}\n", runs[0].1);
    }
    let parts: Vec<String> = runs.iter().map(|(id, w)| format!("run {id} {w}")).collect();
    format!("independence: {}\n", parts.join(", "))
}

/// One line per distinct effort: `effort: low (seats 7→3, lenses 3→1 — effort low)`, led by the runs when the matrix
/// holds runs of another effort or none. Nothing for a matrix of runs made with no effort (the recorded table is unchanged).
fn effort_header(efforts: &[(String, Vec<Value>, Vec<i64>)], total_runs: usize) -> String {
    let mut out = String::new();
    for (e, trimmed, ids) in efforts {
        // the default under a rule, nothing cut: nothing to say
        if e == "medium" && trimmed.is_empty() {
            continue;
        }
        let lead = if efforts.len() > 1 || ids.len() != total_runs {
            let ids: Vec<String> = ids.iter().map(i64::to_string).collect();
            format!("run {}: ", ids.join(", "))
        } else {
            String::new()
        };
        let show = |v: &Value| match v {
            Value::Array(a) => a.iter().map(|x| x.as_str().unwrap_or("").to_string()).collect::<Vec<_>>().join("+"),
            Value::String(t) => t.clone(),
            other => other.to_string(),
        };
        let cuts: Vec<String> = trimmed
            .iter()
            .map(|t| {
                format!(
                    "{} {}\u{2192}{} ({})",
                    t["what"].as_str().unwrap_or(""),
                    show(&t["from"]),
                    show(&t["to"]),
                    t["why"].as_str().unwrap_or("")
                )
            })
            .collect();
        if cuts.is_empty() {
            out.push_str(&format!("{lead}effort: {e}\n"));
        } else {
            out.push_str(&format!("{lead}effort: {e}, trimmed: {}\n", cuts.join("; ")));
        }
    }
    out
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
        // no rule: a brief above 12 seats says why without naming one (D12)
        let what = match o["rule"].as_str() {
            Some(class) => format!("override (rule {class})"),
            None => "override".to_string(),
        };
        out.push_str(&format!(
            "{lead}{what}: {} \u{2014} reason: {reason}\n",
            fields.join(", ")
        ));
    }
    out
}

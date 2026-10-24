//! matrix: per stratum, never averaged across (plan Task 5)
use super::verdict::{load_lens, load_run, open, score_rows, usage_line, usage_summary};
use super::{AppError, Result};
use crate::domain::apply;
use crate::domain::metrics;
use crate::domain::traits;
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
    slice: Option<&str>,
    phase: &str,
) -> Result<Value> {
    if across_strata {
        return Err(Refusal::StrataAverageRefused.into());
    }
    if !matches!(phase, "initial" | "deliberated" | "both") {
        return Err(AppError::InvalidArguments(format!("--phase {phase:?}: the words are initial, deliberated and both")));
    }
    let store = open(root)?;
    let mut rows_i = Vec::new();
    let mut rows_d = Vec::new();
    // per run: the personas with an accepted initial verdict (D34, panel match)
    let mut answered: BTreeMap<i64, std::collections::BTreeSet<String>> = BTreeMap::new();
    let mut panel_match: Vec<Value> = Vec::new();
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
    // the persona version each run briefed: a slice reads the traits the juror was briefed with
    let mut versions: BTreeMap<String, i64> = BTreeMap::new();
    for id in runs {
        let run = load_run(&store, *id)?;
        for b in &run.briefs {
            versions.insert(b.persona.clone(), b.persona_version);
        }
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
        let initial = store.verdict_rows(*id, false)?;
        answered.insert(*id, initial.iter().filter(|r| !super::verdict::is_human(&r.persona)).map(|r| r.persona.clone()).collect());
        if let Some(terms) = store.run_panel_terms(*id)? {
            for t in terms.as_array().into_iter().flatten() {
                let seated: Vec<&str> = t["personas"].as_array().into_iter().flatten().filter_map(Value::as_str).collect();
                let got = seated.iter().filter(|p| answered[id].contains(**p)).count();
                panel_match.push(json!({"run": id, "term": t["term"], "tag": t["tag"], "stake": t["stake"],
                                        "asked": t["asked"], "seated": t["seated"], "answered": got}));
            }
        }
        rows_i.extend(score_rows(&run, &initial));
        rows_d.extend(score_rows(&run, &store.deliberated_rows(*id)?));
        if history {
            all.extend(store.verdict_rows(*id, true)?);
        }
    }
    // D33: the default reads the jury's first word; the advise-only round is read only when asked
    let rows = if phase == "deliberated" { rows_d.clone() } else { rows_i.clone() };
    let mut m = matrix::build(&rows, &stakes, &lenses, Aggregate::PerStratum)?;
    let mut both = if phase == "both" { Some(matrix::build(&rows_d, &stakes, &lenses, Aggregate::PerStratum)?) } else { None };
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
    // cost (D34): what each juror's verdict reported, listed; nothing is divided
    let cost = {
        let rows = store.usage_rows(runs)?;
        let (i, o) = rows.iter().fold((0u64, 0u64), |(i, o), r| (i.saturating_add(r.3), o.saturating_add(r.4)));
        let jurors: std::collections::BTreeSet<(i64, &String)> = rows.iter().map(|r| (r.0, &r.1)).collect();
        json!({
            "per_juror": rows.iter().map(|r| json!({"run": r.0, "persona": r.1, "batch": r.2, "input_tokens": r.3, "output_tokens": r.4})).collect::<Vec<_>>(),
            "reported_total": {"input_tokens": i, "output_tokens": o},
            "jurors_reporting": jurors.len(),
            "estimate": usage["estimate"].clone(),
        })
    };
    // the persuasion shift needs the first word's cells; an initial matrix is built again when this one is another phase
    let shifts = if rows_d.is_empty() {
        BTreeMap::new()
    } else if phase == "initial" {
        metrics::shift(&m, &rows_d, &lenses)
    } else {
        metrics::shift(&matrix::build(&rows_i, &stakes, &lenses, Aggregate::PerStratum)?, &rows_d, &lenses)
    };
    let sliced = match slice {
        Some(t) => Some(slices(&store, t, &rows, &stakes, &lenses, &versions, &m)?),
        None => None,
    };
    let table_text = table.then(|| {
        standard_header(&standards) + &independence_header(&independence) + &effort_header(&efforts, runs.len()) + &override_header(&overrides, runs.len())
            + &matrix::render_table_with(&m, &credentials)
            + &usage_line(&usage).unwrap_or_default()
            + &human_block(&humans)
            + &sliced.as_ref().map(|(_, t)| format!("\n{t}")).unwrap_or_default()
    });
    if detail == Detail::Compact {
        for s in m.strata.iter_mut().chain(both.iter_mut().flat_map(|b| b.strata.iter_mut())) {
            for c in &mut s.cells {
                c.scores.clear();
            }
        }
    }
    // strata that disagree on a cell (D13): beside the strata, never inside one
    let splits = apply::splits(&rows, &stakes);
    let mut out = json!({ "runs": runs, "strata": strata_json(&m, &shifts), "splits": splits, "usage": usage, "cost": cost });
    if !panel_match.is_empty() {
        out["panel_match"] = json!(panel_match);
    }
    if let Some((v, _)) = sliced {
        out["slices"] = Value::Array(v);
    }
    if let Some(d) = both {
        out["phases"] = json!({"initial": strata_json(&m, &shifts), "deliberated": strata_json(&d, &shifts)});
    }
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

/// The strata as data, each with its counts (D34): agreement, evidence totals and, once a deliberation was answered, the shift.
fn strata_json(m: &Matrix, shifts: &BTreeMap<Stake, Value>) -> Value {
    let mut v = serde_json::to_value(&m.strata).expect("strata serialise");
    for (s, o) in m.strata.iter().zip(v.as_array_mut().into_iter().flatten()) {
        o["agreement"] = json!({
            "cells": s.cells.len(),
            "cells_all_within_one": s.cells.iter().filter(|c| c.within_one == c.n).count(),
        });
        let sum = |f: fn(&EvidenceCounts) -> usize| s.cells.iter().map(|c| f(&c.evidence)).sum::<usize>();
        o["evidence"] = json!({"seen": sum(|e| e.seen), "told": sum(|e| e.told), "recalled": sum(|e| e.recalled)});
        if let Some(sh) = shifts.get(&s.stake) {
            o["shift"] = sh.clone();
        }
    }
    v
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

type CellKey = (String, String, Option<String>);

/// How the slice's trait sorts the personas of one stratum into groups, as a key per persona: the bands it holds, or `unset`.
fn partition_of(def: Option<&traits::TraitDef>, v: Option<&traits::TraitValue>) -> String {
    match (def, v) {
        (_, None) => "unset".into(),
        (Some(d), Some(v)) => {
            let h = d.bands_held(v);
            if h.is_empty() {
                "unset".into()
            } else {
                h.join("+")
            }
        }
        (None, Some(v)) => v.show(),
    }
}

/// A trait's groups in each stratum of the matrix (D31): the personas holding each band, their cells, who has none, and the
/// cells on which two groups' `anchor_met` ranges do not overlap. Counts and the lower middle only: no mean, no share, and
/// no group is ever pooled with another stratum's.
fn slices(
    store: &crate::store::Store,
    trait_id: &str,
    rows: &[ScoreRow],
    stakes: &BTreeMap<String, Stake>,
    lenses: &[Lens],
    versions: &BTreeMap<String, i64>,
    m: &Matrix,
) -> Result<(Vec<Value>, String)> {
    let defs = store.trait_map()?;
    let def = defs.get(trait_id).ok_or_else(|| Refusal::UnknownTrait { id: trait_id.to_string() })?;
    if def.shape == "words" {
        return Err(AppError::InvalidArguments(format!("{trait_id} holds words; a slice groups by band or option")));
    }
    let mut held: BTreeMap<String, BTreeMap<String, traits::TraitValue>> = BTreeMap::new();
    for id in stakes.keys() {
        let p = match versions.get(id) {
            Some(v) => store.persona_at(id, *v)?,
            None => store.persona(id)?,
        };
        if let Some(t) = p.and_then(|p| p.traits) {
            held.insert(id.clone(), t);
        }
    }
    let mut out = Vec::new();
    let mut table = String::new();
    for st in &m.strata {
        let mine = |p: &String| held.get(p).and_then(|t| t.get(trait_id));
        let mut groups = Vec::new();
        let mut unset: Vec<String> = Vec::new();
        let mut table_blocks = String::new();
        for band in def.bands() {
            let members: Vec<String> =
                st.personas.iter().filter(|p| mine(p).is_some_and(|v| def.bands_held(v).contains(&band))).cloned().collect();
            if members.is_empty() {
                table_blocks.push_str(&format!("slice {trait_id}: {}, {band}: \u{2014}\n\n", st.stake.name()));
                continue;
            }
            let gr: Vec<ScoreRow> = rows.iter().filter(|r| members.contains(&r.persona)).cloned().collect();
            let gm = matrix::build(&gr, stakes, lenses, Aggregate::PerStratum)?;
            let gstratum = gm.strata.into_iter().find(|s| s.stake == st.stake).unwrap_or(Stratum {
                stake: st.stake,
                personas: members.clone(),
                cells: vec![],
            });
            table_blocks.push_str(&format!(
                "slice {trait_id}: {}, {band} (n {}: {})\n\n{}\n",
                st.stake.name(),
                members.len(),
                members.join(", "),
                matrix::render_table(&Matrix { strata: vec![gstratum.clone()] })
            ));
            groups.push((band, members, gstratum));
        }
        for p in &st.personas {
            if mine(p).is_none_or(|v| def.bands_held(v).is_empty()) {
                unset.push(p.clone());
            }
        }
        // separates: per cell, each group's own range of anchor_met; two disjoint ranges are a cell the trait separates
        let mut per_group: BTreeMap<CellKey, Vec<(i64, i64)>> = BTreeMap::new();
        for (_, members, _) in &groups {
            let mut cell: BTreeMap<CellKey, (i64, i64)> = BTreeMap::new();
            for r in rows.iter().filter(|r| members.contains(&r.persona)) {
                let e = cell.entry((r.lens.clone(), r.dimension.clone(), r.about.clone())).or_insert((r.anchor_met, r.anchor_met));
                e.0 = e.0.min(r.anchor_met);
                e.1 = e.1.max(r.anchor_met);
            }
            for (k, v) in cell {
                per_group.entry(k).or_default().push(v);
            }
        }
        let separated = per_group
            .values()
            .filter(|rs| rs.iter().enumerate().any(|(i, a)| rs[i + 1..].iter().any(|b| a.1 < b.0 || b.1 < a.0)))
            .count();
        // collinear: another trait the stratum's personas hold that sorts them exactly as this one does
        let mut warnings: Vec<String> = Vec::new();
        let mut others: Vec<&String> = held.values().flat_map(|t| t.keys()).filter(|k| *k != trait_id).collect();
        others.sort();
        others.dedup();
        for o in others {
            let a: Vec<String> = st.personas.iter().map(|p| partition_of(Some(def), mine(p))).collect();
            let b: Vec<String> = st.personas.iter().map(|p| partition_of(defs.get(o), held.get(p).and_then(|t| t.get(o)))).collect();
            let blocks = a.iter().collect::<std::collections::BTreeSet<_>>().len();
            let same = (0..a.len()).all(|i| (0..a.len()).all(|j| (a[i] == a[j]) == (b[i] == b[j])));
            if blocks >= 2 && same {
                warnings.push(format!("collinear with {o}"));
            }
        }
        out.push(json!({
            "trait": trait_id,
            "shape": def.shape,
            "stratum": st.stake.name(),
            "groups": groups.iter().map(|(band, members, g)| json!({
                "band": band, "personas": members, "n": members.len(), "thin": members.len() < 2, "cells": g.cells,
            })).collect::<Vec<_>>(),
            "unset": unset,
            "separates": {"cells": separated, "total": st.cells.len()},
            "warnings": warnings,
        }));
        table.push_str(&table_blocks);
        if !unset.is_empty() {
            table.push_str(&format!("slice {trait_id}: {}, unset: {}\n\n", st.stake.name(), unset.join(", ")));
        }
    }
    Ok((out, table))
}

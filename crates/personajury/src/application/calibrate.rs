//! calibrate: set the jury's rank order beside the humans' (D20). A human run is an ordinary run whose verdict batches
//! were entered with `--by-human`; each person speaks from a seat, so their rows count in that seat's stratum.
use super::verdict::{is_human, load_run, open, score_rows};
use super::{AppError, Result};
use crate::domain::calibrate as cal;
use crate::domain::traits::TraitValue;
use std::collections::BTreeMap;
use crate::domain::*;
use serde_json::{json, Value};
use std::path::Path;

pub fn show(root: &Path, jury: i64, human: i64, out: Option<&Path>, phase: &str, slice: Option<&str>) -> Result<Value> {
    if !matches!(phase, "initial" | "deliberated") {
        return Err(AppError::InvalidArguments(format!("--phase {phase:?}: the words are initial and deliberated")));
    }
    let store = open(root)?;
    let (jr, hr) = (load_run(&store, jury)?, load_run(&store, human)?);
    if store.human_batches(human)?.is_empty() {
        return Err(Refusal::NotHumanRun { run: human }.into());
    }
    if jr.subject_name != hr.subject_name || jr.lens_name != hr.lens_name || jr.lens_version != hr.lens_version {
        return Err(AppError::InvalidArguments(format!(
            "calibrate needs the same subject and lens version: run {jury} is {} / {} v{}, run {human} is {} / {} v{}",
            jr.subject_name, jr.lens_name, jr.lens_version, hr.subject_name, hr.lens_name, hr.lens_version
        )));
    }
    let jury_rows: Vec<ScoreRow> = score_rows(&jr, &super::verdict::rows_in(&store, jury, phase)?.into_iter().filter(|r| !is_human(&r.persona)).collect::<Vec<_>>());
    // a person speaks in the stratum of the seat they sat in: `human:<name>:<seat>` counts as the seat
    let mut human_rows: Vec<ScoreRow> = Vec::new();
    // the same rows under the person's own id, for a trait slice (D36)
    let mut human_full: Vec<ScoreRow> = Vec::new();
    for r in score_rows(&hr, &store.verdict_rows(human, false)?) {
        if is_human(&r.persona) {
            let seat = r.persona.rsplit(':').next().unwrap_or("").to_string();
            human_full.push(r.clone());
            human_rows.push(ScoreRow { persona: seat, ..r });
        }
    }
    let strata = cal::compare(&jury_rows, &jr.stakes, &human_rows, &hr.stakes);
    let calibrated = cal::calibrated(&strata);
    let outside: Vec<Value> = strata
        .iter()
        .filter(|s| s.stake != Stake::Primary)
        .flat_map(|s| s.cells.iter().filter(|c| !c.within_one_rank).map(move |c| {
            json!({"stake": s.stake, "lens": c.lens, "dimension": c.dimension, "about": c.about, "jury_rank": c.jury_rank, "human_rank": c.human_rank})
        }))
        .collect();
    let data = json!({
        "jury_run": jury, "human_run": human, "subject": jr.subject_name, "lens": jr.lens_name, "lens_version": jr.lens_version,
        "calibrated": calibrated, "strata": strata, "outside_primary": outside,
        "rule": "calibrated when every primary-stratum cell is within one rank; bystander and secondary differences are reported, never averaged in",
    });
    let mut data = data;
    if let Some(t) = slice {
        data["slices"] = json!(slices(&store, t, &jr, &hr, &jury_rows, &human_full)?);
    }
    if let Some(p) = out {
        let text = serde_json::to_string_pretty(&data).expect("serialises") + "\n";
        std::fs::write(p, text).map_err(|e| AppError::Io(p.to_path_buf(), e))?;
    }
    Ok(data)
}

/// The trait's bands in each stratum, on the jury's side (the personas the run briefed, with the traits they were briefed
/// with) and on the people's (the traits each person declared), and what the trait does to the cells on each side (D36).
/// A side without the trait is `unset` and left out of every group; nothing crosses a stratum.
fn slices(
    store: &crate::store::Store,
    trait_id: &str,
    jr: &RunRow,
    hr: &RunRow,
    jury_rows: &[ScoreRow],
    human_rows: &[ScoreRow],
) -> Result<Vec<cal::SliceCal>> {
    let defs = store.trait_map()?;
    let def = defs.get(trait_id).ok_or_else(|| Refusal::UnknownTrait { id: trait_id.to_string() })?;
    if def.shape == "words" {
        return Err(AppError::InvalidArguments(format!("{trait_id} holds words; a slice groups by band or option")));
    }
    let mut jury_traits: BTreeMap<String, TraitValue> = BTreeMap::new();
    for b in &jr.briefs {
        let p = store.persona_at(&b.persona, b.persona_version)?;
        if let Some(v) = p.and_then(|p| p.traits).and_then(|t| t.get(trait_id).cloned()) {
            jury_traits.insert(b.persona.clone(), v);
        }
    }
    let people_traits: BTreeMap<String, TraitValue> = store
        .human_traits(hr.id)?
        .into_iter()
        .filter_map(|(id, t)| t.get(trait_id).cloned().map(|v| (id, v)))
        .collect();
    let seat_of = |id: &str| id.rsplit(':').next().unwrap_or("").to_string();
    let mut out = Vec::new();
    for stake in [Stake::Primary, Stake::Secondary, Stake::Bystander] {
        let rows_j: Vec<ScoreRow> = jury_rows.iter().filter(|r| jr.stakes.get(&r.persona) == Some(&stake)).cloned().collect();
        let rows_p: Vec<ScoreRow> = human_rows.iter().filter(|r| hr.stakes.get(&seat_of(&r.persona)) == Some(&stake)).cloned().collect();
        let ids = |rows: &[ScoreRow]| -> Vec<String> {
            let mut v: Vec<String> = rows.iter().map(|r| r.persona.clone()).collect();
            v.sort();
            v.dedup();
            v
        };
        let (jm, pm) = (ids(&rows_j), ids(&rows_p));
        if jm.is_empty() && pm.is_empty() {
            continue;
        }
        let side = |members: &[String], rows: &[ScoreRow], traits: &BTreeMap<String, TraitValue>, band: &str| {
            let m: Vec<String> = members.iter().filter(|p| traits.get(*p).is_some_and(|v| def.bands_held(v).iter().any(|b| b == band))).cloned().collect();
            cal::Side { rows: rows.iter().filter(|r| m.contains(&r.persona)).cloned().collect(), members: m }
        };
        let bands: Vec<(String, cal::Side, cal::Side)> = def
            .bands()
            .into_iter()
            .map(|b| (b.clone(), side(&jm, &rows_j, &jury_traits, &b), side(&pm, &rows_p, &people_traits, &b)))
            .collect();
        let unset = |members: &[String], traits: &BTreeMap<String, TraitValue>| -> Vec<String> {
            members.iter().filter(|p| traits.get(*p).is_none_or(|v| def.bands_held(v).is_empty())).cloned().collect()
        };
        let mut stakes: BTreeMap<String, Stake> = jm.iter().map(|p| (p.clone(), stake)).collect();
        stakes.extend(pm.iter().map(|p| (p.clone(), stake)));
        out.push(cal::slice(
            trait_id,
            stake,
            &bands,
            &stakes,
            unset(&jm, &jury_traits),
            unset(&pm, &people_traits),
            cal::total_cells(&rows_j),
            cal::total_cells(&rows_p),
        ));
    }
    Ok(out)
}

/// `trait report --calibration F [F ...]` (D36): per trait and stratum, how many of the files' slices said each word. A
/// stratum is never added to another, and the files are never averaged: a count of words, with the number of files that
/// sliced that trait in that stratum.
pub fn report(files: &[std::path::PathBuf]) -> Result<Value> {
    #[derive(Default)]
    struct Row {
        words: BTreeMap<String, usize>,
        files: usize,
    }
    let mut rows: BTreeMap<(String, String), Row> = BTreeMap::new();
    for f in files {
        let not = || AppError::from(Refusal::NotACalibration { path: f.display().to_string() });
        let text = super::read_text(f)?;
        let v: Value = serde_json::from_str(&text).map_err(|_| not())?;
        if v.get("calibrated").and_then(Value::as_bool).is_none() || v.get("strata").and_then(Value::as_array).is_none() {
            return Err(not());
        }
        for s in v.get("slices").and_then(Value::as_array).into_iter().flatten() {
            let (Some(t), Some(st), Some(w)) = (s["trait"].as_str(), s["stratum"].as_str(), s["word"].as_str()) else { continue };
            let row = rows.entry((t.to_string(), st.to_string())).or_default();
            *row.words.entry(w.to_string()).or_default() += 1;
            row.files += 1;
        }
    }
    let report: Vec<Value> = rows
        .iter()
        .map(|((t, st), r)| {
            let n = |w: &str| r.words.get(w).copied().unwrap_or(0);
            json!({"trait": t, "stratum": st, "predicts": n("predicts"), "played": n("played"), "missed": n("missed"),
                   "inert": n("inert"), "thin": n("thin"), "files": r.files})
        })
        .collect();
    let mut table = String::from("| trait | stratum | predicts | played | missed | inert | thin | files |\n|---|---|---|---|---|---|---|---|\n");
    for r in &report {
        table.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
            r["trait"].as_str().unwrap_or(""), r["stratum"].as_str().unwrap_or(""), r["predicts"], r["played"], r["missed"], r["inert"], r["thin"], r["files"]
        ));
    }
    Ok(json!({"files": files.len(), "report": report, "table": table}))
}

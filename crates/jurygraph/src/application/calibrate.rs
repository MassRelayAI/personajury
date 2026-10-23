//! calibrate: set the jury's rank order beside the humans' (D20). A human run is an ordinary run whose verdict batches
//! were entered with `--by-human`; each person speaks from a seat, so their rows count in that seat's stratum.
use super::verdict::{is_human, load_run, open, score_rows};
use super::{AppError, Result};
use crate::domain::calibrate as cal;
use crate::domain::*;
use serde_json::{json, Value};
use std::path::Path;

pub fn show(root: &Path, jury: i64, human: i64, out: Option<&Path>) -> Result<Value> {
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
    let jury_rows: Vec<ScoreRow> = score_rows(&jr, &store.verdict_rows(jury, false)?.into_iter().filter(|r| !is_human(&r.persona)).collect::<Vec<_>>());
    // a person speaks in the stratum of the seat they sat in: `human:<name>:<seat>` counts as the seat
    let mut human_rows: Vec<ScoreRow> = Vec::new();
    for r in score_rows(&hr, &store.verdict_rows(human, false)?) {
        if is_human(&r.persona) {
            let seat = r.persona.rsplit(':').next().unwrap_or("").to_string();
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
    if let Some(p) = out {
        let text = serde_json::to_string_pretty(&data).expect("serialises") + "\n";
        std::fs::write(p, text).map_err(|e| AppError::Io(p.to_path_buf(), e))?;
    }
    Ok(data)
}

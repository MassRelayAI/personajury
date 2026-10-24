//! export (D40): the verdict rows of runs, raw, as ndjson or csv. Reads only.
use super::verdict::{is_human, load_run, open};
use super::{AppError, Result};
use crate::domain::export::{self, ExportRow};
use crate::domain::traits::{held_sensitive, TraitValue};
use crate::domain::VerdictRow;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::Path;

/// The rows of these runs (each once), in the fixed order. `phase`: `initial` (the jury's first word), `deliberated`, `all`.
pub fn rows(root: &Path, runs: &[i64], phase: &str) -> Result<Vec<ExportRow>> {
    if !matches!(phase, "initial" | "deliberated" | "all") {
        return Err(AppError::InvalidArguments(format!("--phase {phase:?}: the words are initial, deliberated and all")));
    }
    let store = open(root)?;
    let defs = store.trait_map()?;
    let mut ids = runs.to_vec();
    ids.sort_unstable();
    ids.dedup();
    let mut out = Vec::new();
    for id in ids {
        let run = load_run(&store, id)?;
        let humans = store.human_batches(id)?;
        let declared = store.human_traits(id)?;
        let mut batches: Vec<(&'static str, Vec<VerdictRow>)> = Vec::new();
        if phase != "deliberated" {
            batches.push(("initial", store.verdict_rows(id, false)?));
        }
        if phase != "initial" {
            batches.push(("deliberated", store.deliberated_rows(id)?));
        }
        for (name, vrows) in batches {
            for r in vrows {
                let human = is_human(&r.persona);
                // a person speaks from a seat of the run: its stratum
                let seat = if human { r.persona.rsplit(':').next().unwrap_or("") } else { r.persona.as_str() };
                let Some(stratum) = run.stakes.get(seat).copied() else { continue };
                let (model, traits, sensitive): (Option<String>, BTreeMap<String, TraitValue>, Vec<String>) = if human {
                    let t = declared.get(&r.persona).cloned().unwrap_or_default();
                    let s = held_sensitive(&t, &defs).into_iter().map(String::from).collect();
                    (None, t, s)
                } else {
                    let model = run.briefs.iter().find(|b| b.persona == r.persona).map(|b| b.model.clone());
                    match store.persona_at(&r.persona, r.persona_version)? {
                        Some(p) => {
                            let s = super::persona::sensitive_of(&p, &defs).into_iter().map(String::from).collect();
                            (model, p.traits.clone().unwrap_or_default(), s)
                        }
                        None => (model, BTreeMap::new(), vec![]),
                    }
                };
                out.push(ExportRow {
                    run: id,
                    subject: run.subject_name.clone(),
                    subject_version: run.subject_version,
                    lens: run.lens_name.clone(),
                    lens_version: run.lens_version,
                    stratum,
                    model,
                    dimension: r.dimension,
                    about: r.about,
                    score: r.score,
                    anchor_met: r.anchor_met,
                    evidence: r.evidence.name().to_string(),
                    citations: r.citations,
                    blockers: r.blockers.len(),
                    phase: name,
                    by_human: human.then(|| humans.get(&r.persona).cloned()).flatten(),
                    traits,
                    sensitive,
                    persona: r.persona,
                });
            }
        }
    }
    export::sort(&mut out);
    Ok(out)
}

fn render(rows: &[ExportRow], format: &str) -> Result<String> {
    match format {
        "ndjson" => Ok(export::ndjson(rows)),
        "csv" => Ok(export::csv(rows)),
        other => Err(AppError::InvalidArguments(format!("--format {other:?}: the words are ndjson and csv"))),
    }
}

/// The text the CLI prints to stdout: the rows alone, no envelope.
pub fn raw(root: &Path, runs: &[i64], format: &str, phase: &str) -> Result<String> {
    render(&rows(root, runs, phase)?, format)
}

/// The envelope's data. With `out` the file is written whole and the answer says where; without it the text rides in
/// `data.<format>`, cut to `cap` rows with `truncated: true` (`rows` is always the total).
pub fn show(root: &Path, runs: &[i64], format: &str, phase: &str, out: Option<&Path>, cap: Option<usize>) -> Result<Value> {
    let mut all = rows(root, runs, phase)?;
    let total = all.len();
    if let Some(f) = out {
        let text = render(&all, format)?;
        std::fs::write(f, text.as_bytes()).map_err(|e| AppError::Io(f.to_path_buf(), e))?;
        return Ok(json!({"format": format, "rows": total, "out": f.to_string_lossy()}));
    }
    let truncated = cap.is_some_and(|c| total > c);
    if let Some(c) = cap {
        all.truncate(c);
    }
    Ok(json!({"format": format, "rows": total, "truncated": truncated, format: render(&all, format)?}))
}

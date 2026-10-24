//! deliberate: one advise-only round after the jury's first word (D33). Each juror reads what the others saw and may
//! change its verdict. It breaks independence by design, so it only advises: it measures persuasion, it decides nothing.
use super::brief::{persona_section, stake_section, TraitCtx};
use super::verdict::{load_lens, load_run, open};
use super::{AppError, Result};
use crate::domain::hash::sha256_hex;
use crate::domain::*;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::Path;

fn line(r: &VerdictRow, own: bool) -> String {
    let mut s = format!("- {}", r.dimension);
    if let Some(a) = &r.about {
        s.push_str(&format!(" about {a}"));
    }
    s.push_str(&format!(": score {}, anchor_met {}", r.score, r.anchor_met));
    if own {
        s.push_str(&format!(", evidence {}", r.evidence.name()));
    }
    if !r.blockers.is_empty() {
        s.push_str(&format!(", blockers: {}", r.blockers.join("; ")));
    }
    if let Some(n) = r.note.as_deref().filter(|n| !n.trim().is_empty()) {
        s.push_str(&format!(", note: {}", n.split_whitespace().collect::<Vec<_>>().join(" ")));
    }
    s.push('\n');
    s
}

pub fn run(root: &Path, run_id: i64, wave: Option<usize>) -> Result<Value> {
    if wave == Some(0) {
        return Err(AppError::InvalidArguments("--wave must be at least 1".into()));
    }
    let store = open(root)?;
    let run = load_run(&store, run_id)?;
    if store.deliberation(run.id)?.is_some() {
        return Err(Refusal::AlreadyDeliberated { run: run.id }.into());
    }
    let rows = store.verdict_rows(run.id, false)?;
    let mut by: BTreeMap<String, Vec<&VerdictRow>> = BTreeMap::new();
    for r in rows.iter().filter(|r| !super::verdict::is_human(&r.persona)) {
        by.entry(r.persona.clone()).or_default().push(r);
    }
    let missing: Vec<String> = run.personas.iter().filter(|p| !by.contains_key(*p)).cloned().collect();
    if !missing.is_empty() {
        return Err(Refusal::IncompleteJuryForDeliberation { missing }.into());
    }
    let lens = load_lens(&store, &run)?;
    let traits = TraitCtx::for_run(&store, store.run_traits(run.id)?.0.as_ref())?;
    // the other jurors, anonymised, in persona-id order
    let mut ids: Vec<&String> = run.personas.iter().collect();
    ids.sort();
    let dir_rel = format!(".personajury/runs/{}", run.id);
    let dir = root.join(&dir_rel);
    std::fs::create_dir_all(&dir).map_err(|e| AppError::Io(dir.clone(), e))?;
    let mut refs: Vec<Value> = Vec::new();
    let mut files: Vec<(String, String)> = Vec::new();
    for id in &run.personas {
        let brief = run.briefs.iter().find(|b| &b.persona == id);
        let persona = match brief {
            Some(b) => store.persona_at(id, b.persona_version)?,
            None => store.persona(id)?,
        }
        .ok_or_else(|| AppError::NotFound(format!("no persona {id}")))?;
        let stake = run.stakes.get(id).copied().ok_or_else(|| Refusal::PersonaWithoutStake { persona: id.clone() })?;
        let mut t = String::from("personajury-deliberation 1\n\n");
        t.push_str(&format!("# {} reconsiders {} under the {} lens\n\n", persona.name, run.subject_name, lens.name));
        let block = traits.as_ref().and_then(|c| c.block_for(&persona));
        t.push_str(&persona_section(&persona, block.as_deref()));
        t.push_str(&stake_section(&persona, stake));
        t.push_str("## Your first verdict\n\n");
        for r in &by[id] {
            t.push_str(&line(r, true));
        }
        t.push_str("\n## What the other jurors said\n");
        let mut n = 0;
        for other in ids.iter().filter(|o| ***o != *id) {
            n += 1;
            let st = run.stakes.get(*other).map_or("", |s| s.name());
            t.push_str(&format!("\nJuror {n} ({st})\n\n"));
            for r in &by[*other] {
                t.push_str(&line(r, false));
            }
        }
        t.push_str("\n## What to do\n\nRead what they saw. Change a line only where their evidence persuades you, and say so in its note. A score you changed because of another juror is `told` evidence unless you cite what you saw yourself. Answer with the full verdict in the same shape as before, with \"phase\": \"deliberated\" beside \"persona\".\n");
        files.push((id.clone(), t));
    }
    for (id, t) in &files {
        let rel = format!("{dir_rel}/deliberate-{id}--{}.md", lens.name);
        let path = root.join(&rel);
        std::fs::write(&path, t.as_bytes()).map_err(|e| AppError::Io(path.clone(), e))?;
        refs.push(json!({"persona": id, "path": rel, "hash": sha256_hex(t.as_bytes()), "bytes": t.len()}));
    }
    store.insert_deliberation(run.id, &json!(refs))?;
    let wave = wave.unwrap_or(super::brief::DEFAULT_WAVE);
    Ok(json!({
        "run": run.id,
        "briefs": refs,
        "bytes_total": files.iter().map(|(_, t)| t.len()).sum::<usize>(),
        "waves": run.personas.chunks(wave).map(|c| c.to_vec()).collect::<Vec<_>>(),
        "note": "advise-only: a deliberated verdict measures persuasion; the initial batch stays the jury's independent word",
    }))
}
